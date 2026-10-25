//! 两个真实引擎实现：DSH headless（`HarnessEngine`）与 DeepSeek 直连（`DirectEngine`）。
//!
//! 二者都实现 [`ChatEngine`]，网关只依赖 trait，所以测试可以换假引擎。

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};

use crate::ai::gateway::{ChatEngine, DeltaKind, EngineOutcome, EngineRequest};
use crate::ai::gateway::{AiEngine, DEFAULT_DIRECT_MODEL};
use crate::error::{AppError, ErrorKind};
use crate::harness::{self, HarnessCommand, HarnessPaths, HarnessOutcome, SandboxPolicy};

/// 环境变量：DeepSeek API Key。
pub const ENV_API_KEY: &str = "DEEPSEEK_API_KEY";

/// DSH 默认模型（`dsh-base` 的 `agent-default-model`）。
pub const HARNESS_DEFAULT_MODEL: &str = "deepseek-flash";

/// 直连默认 base URL。
pub const DIRECT_DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

/// DSH headless 引擎。
#[derive(Debug, Clone)]
pub struct HarnessEngine {
    paths: HarnessPaths,
    workspace_root: PathBuf,
    patch_dir: PathBuf,
    policy: SandboxPolicy,
    api_key: String,
    timeout: Duration,
}

impl HarnessEngine {
    pub fn new(
        paths: HarnessPaths,
        paint_home: &Path,
        policy: SandboxPolicy,
        api_key: String,
    ) -> Self {
        Self {
            paths,
            workspace_root: paint_home.join("workspaces"),
            patch_dir: paint_home.join("harness").join("patches"),
            policy,
            api_key,
            timeout: Duration::from_secs(harness::DEFAULT_TIMEOUT_SECS),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 每个画布一个独立工作目录（沙箱边界）。
    fn workspace_for(&self, canvas_id: Option<&str>) -> PathBuf {
        let name = canvas_id.unwrap_or("default");
        self.workspace_root.join(sanitize_component(name))
    }
}

impl ChatEngine for HarnessEngine {
    fn engine(&self) -> AiEngine {
        AiEngine::Harness
    }

    fn model(&self) -> &str {
        HARNESS_DEFAULT_MODEL
    }

    fn run(
        &self,
        request: &EngineRequest,
        on_delta: &mut dyn FnMut(DeltaKind, &str),
    ) -> Result<EngineOutcome, AppError> {
        if self.api_key.trim().is_empty() {
            return Err(AppError::new(
                ErrorKind::Ai,
                "未配置 DEEPSEEK_API_KEY，无法调用 Harness 引擎",
            ));
        }

        let workspace = self.workspace_for(request.canvas_id.as_deref());
        std::fs::create_dir_all(&workspace).map_err(|error| {
            AppError::io(format!("创建 DSH 工作目录 {} 失败：{error}", workspace.display()))
        })?;

        let patch = write_run_patch(&self.patch_dir, self.policy, request.persona.as_deref())?;
        let command = HarnessCommand::headless(
            &self.paths,
            &patch,
            &workspace,
            &request.prompt,
            &self.api_key,
        );

        let mut collected = String::new();
        let outcome = harness::run(&command, self.timeout, &mut |event| match event {
            harness::RunEvent::Text { text } => {
                collected.push_str(&text);
                on_delta(DeltaKind::Content, &text);
            }
            harness::RunEvent::Thinking { text } => on_delta(DeltaKind::Reasoning, &text),
            _ => {}
        })?;

        Ok(outcome_to_engine_outcome(outcome, collected))
    }
}

/// 把沙箱策略与员工人设合成一个 patch 文件（`--patch` 可叠加，同一个文件里更省事）。
fn write_run_patch(
    dir: &Path,
    policy: SandboxPolicy,
    persona: Option<&str>,
) -> Result<PathBuf, AppError> {
    let mut yaml = policy.patch_yaml();

    if let Some(persona) = persona.map(str::trim).filter(|value| !value.is_empty()) {
        yaml.push_str("\n- id: system-prompt\n");
        yaml.push_str("  config:\n");
        yaml.push_str("    personaPrefix: |-\n");
        for line in persona.lines() {
            yaml.push_str("      ");
            yaml.push_str(line);
            yaml.push('\n');
        }
    }

    let path = dir.join("paint-run.patch.yml");
    crate::fsutil::write_if_changed(&path, yaml.as_bytes())?;
    Ok(path)
}

fn outcome_to_engine_outcome(outcome: HarnessOutcome, collected: String) -> EngineOutcome {
    let text = if outcome.final_text.trim().is_empty() {
        collected
    } else {
        outcome.final_text.clone()
    };

    EngineOutcome {
        success: outcome.is_success(),
        text,
        tokens_input: outcome.tokens_input,
        tokens_output: outcome.tokens_output,
        model: HARNESS_DEFAULT_MODEL.to_string(),
        error: (!outcome.is_success()).then(|| outcome.failure_message()),
        tool_calls: outcome.tool_calls,
        harness_session_id: outcome.session_id,
    }
}

/// DeepSeek 直连引擎（SSE 流式）。
#[derive(Debug, Clone)]
pub struct DirectEngine {
    api_key: String,
    base_url: String,
    model: String,
    timeout: Duration,
}

impl DirectEngine {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DIRECT_DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_DIRECT_MODEL.to_string(),
            timeout: Duration::from_secs(180),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// 从环境变量构造；没有 key 时返回 `None`（网关降级为 harness-only）。
    pub fn from_env() -> Option<Self> {
        let key = std::env::var(ENV_API_KEY).ok()?;
        if key.trim().is_empty() {
            return None;
        }
        Some(Self::new(key))
    }
}

impl ChatEngine for DirectEngine {
    fn engine(&self) -> AiEngine {
        AiEngine::Direct
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn run(
        &self,
        request: &EngineRequest,
        on_delta: &mut dyn FnMut(DeltaKind, &str),
    ) -> Result<EngineOutcome, AppError> {
        if self.api_key.trim().is_empty() {
            return Err(AppError::new(
                ErrorKind::Ai,
                "未配置 DEEPSEEK_API_KEY，无法调用 Direct 引擎",
            ));
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|error| AppError::new(ErrorKind::Ai, format!("创建 HTTP 客户端失败：{error}")))?;

        let body = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": request.prompt }],
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        let response = client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|error| AppError::new(ErrorKind::Ai, format!("请求 DeepSeek 失败：{error}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(AppError::new(
                ErrorKind::Ai,
                format!("DeepSeek 返回 {status}：{}", truncate(&body, 300)),
            ));
        }

        let mut text = String::new();
        let mut tokens_input = 0;
        let mut tokens_output = 0;

        for line in std::io::BufReader::new(response).lines() {
            let line = line.map_err(|error| {
                AppError::new(ErrorKind::Ai, format!("读取 SSE 流失败：{error}"))
            })?;
            let Some(event) = parse_sse_line(&line) else {
                continue;
            };
            if event.done {
                break;
            }
            if let Some(reasoning) = event.reasoning {
                on_delta(DeltaKind::Reasoning, &reasoning);
            }
            if let Some(content) = event.content {
                text.push_str(&content);
                on_delta(DeltaKind::Content, &content);
            }
            if let Some((input, output)) = event.usage {
                tokens_input = input;
                tokens_output = output;
            }
        }

        Ok(EngineOutcome {
            success: true,
            text,
            tokens_input,
            tokens_output,
            model: self.model.clone(),
            error: None,
            tool_calls: Vec::new(),
            harness_session_id: None,
        })
    }
}

/// 一条 SSE 事件里我们关心的内容。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SseEvent {
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub usage: Option<(i64, i64)>,
    pub done: bool,
}

/// 解析 `data: {...}` 行；注释行、空行、非 data 行返回 `None`。
pub fn parse_sse_line(line: &str) -> Option<SseEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return None;
    }
    let payload = trimmed.strip_prefix("data:")?.trim();
    if payload == "[DONE]" {
        return Some(SseEvent {
            done: true,
            ..Default::default()
        });
    }

    let value: Value = serde_json::from_str(payload).ok()?;
    let choice = value.get("choices").and_then(|choices| choices.get(0));
    let delta = choice.and_then(|choice| choice.get("delta"));

    let usage = value.get("usage").and_then(|usage| {
        let input = usage.get("prompt_tokens")?.as_i64()?;
        let output = usage.get("completion_tokens")?.as_i64()?;
        Some((input, output))
    });

    Some(SseEvent {
        content: delta
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        reasoning: delta
            .and_then(|delta| delta.get("reasoning_content"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        usage,
        done: false,
    })
}

fn sanitize_component(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "default".to_string()
    } else {
        cleaned
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    value.chars().take(limit).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_content_and_reasoning_are_split() {
        let content = parse_sse_line(
            r#"data: {"choices":[{"delta":{"content":"你好"}}]}"#,
        )
        .expect("解析内容");
        assert_eq!(content.content.as_deref(), Some("你好"));
        assert!(content.reasoning.is_none());

        let reasoning = parse_sse_line(
            r#"data: {"choices":[{"delta":{"reasoning_content":"想一想"}}]}"#,
        )
        .expect("解析推理");
        assert_eq!(reasoning.reasoning.as_deref(), Some("想一想"));
        assert!(reasoning.content.is_none());
    }

    #[test]
    fn sse_usage_only_appears_in_final_chunk() {
        let event = parse_sse_line(
            r#"data: {"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":7}}"#,
        )
        .expect("解析 usage");

        assert_eq!(event.usage, Some((12, 7)));
        assert!(event.content.is_none());
    }

    #[test]
    fn sse_done_comment_and_garbage_are_handled() {
        assert!(parse_sse_line("data: [DONE]").expect("done").done);
        assert!(parse_sse_line(": keep-alive").is_none());
        assert!(parse_sse_line("event: ping").is_none());
        assert!(parse_sse_line("data: {oops").is_none());
        assert!(parse_sse_line("").is_none());
    }

    #[test]
    fn empty_content_chunks_are_dropped() {
        let event = parse_sse_line(r#"data: {"choices":[{"delta":{"content":""}}]}"#).expect("解析");
        assert!(event.content.is_none());
    }

    #[test]
    fn harness_outcome_prefers_final_text_then_collected() {
        let with_final = HarnessOutcome {
            exit_code: Some(0),
            final_text: "最终答案".to_string(),
            turn_reason: Some("completed".to_string()),
            tokens_input: 10,
            tokens_output: 2,
            ..Default::default()
        };
        let mapped = outcome_to_engine_outcome(with_final, "中间输出".to_string());
        assert_eq!(mapped.text, "最终答案");
        assert!(mapped.success);
        assert_eq!(mapped.tokens_input, 10);

        let without_final = HarnessOutcome {
            exit_code: Some(0),
            final_text: String::new(),
            ..Default::default()
        };
        let mapped = outcome_to_engine_outcome(without_final, "流式片段".to_string());
        assert_eq!(mapped.text, "流式片段");
    }

    #[test]
    fn failed_harness_run_carries_error_message() {
        let failed = HarnessOutcome {
            exit_code: Some(1),
            timed_out: true,
            ..Default::default()
        };
        let mapped = outcome_to_engine_outcome(failed, String::new());

        assert!(!mapped.success);
        assert!(mapped.error.unwrap_or_default().contains("超时"));
    }

    #[test]
    fn workspace_component_is_sanitized() {
        assert_eq!(sanitize_component("canvas-1"), "canvas-1");
        assert_eq!(sanitize_component("../evil path"), ".._evil_path");
        assert_eq!(sanitize_component(""), "default");
    }
}
