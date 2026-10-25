//! AI 网关：所有 AI 调用的唯一入口。
//!
//! 职责（engineering-contract 第 6 节、`learning-records/0003`）：
//! 上下文装配 → 引擎选择 → 并发控制 → 流式推送 → Token 计量 → 审计落库。
//!
//! 引擎实现通过 [`ChatEngine`] 注入，测试用假引擎，不碰网络与真实 DSH。

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ai::context::{build_canvas_context, compose_prompt};
use crate::ai::repository::{self as ai_repository, NewAiCall};
pub use crate::ai::repository::{AiCall, AiUsageRow};
use crate::error::{AppError, ErrorKind};
use crate::events::{EventSink, EVENT_AI_DELTA, EVENT_AI_DONE, EVENT_AI_ERROR};
use crate::paint::document::now_ms;
use crate::paint::service;
use crate::store::Store;

/// Direct 引擎的最大在途请求数（网络/费用约束）。
pub const MAX_CONCURRENT_DIRECT: usize = 2;

/// 单次 prompt 的字符上限。
pub const MAX_PROMPT_CHARS: usize = 8000;

/// Direct 引擎拿不到并发额度时的等待上限。
pub const DIRECT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(15);

/// Direct 引擎默认模型。
pub const DEFAULT_DIRECT_MODEL: &str = "deepseek-chat";

/// AI 引擎。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiEngine {
    Harness,
    Direct,
}

impl AiEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            AiEngine::Harness => "harness",
            AiEngine::Direct => "direct",
        }
    }
}

impl std::str::FromStr for AiEngine {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "harness" => Ok(AiEngine::Harness),
            "direct" => Ok(AiEngine::Direct),
            other => Err(AppError::invalid_action(format!("未知的 AI 引擎：{other}"))),
        }
    }
}

/// 流式增量的种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaKind {
    Content,
    Reasoning,
}

impl DeltaKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            DeltaKind::Content => "content",
            DeltaKind::Reasoning => "reasoning",
        }
    }
}

/// 前端发起的对话请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatRequest {
    #[serde(default)]
    pub canvas_id: Option<String>,
    #[serde(default)]
    pub employee_id: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub engine: Option<AiEngine>,
    #[serde(default)]
    pub module: Option<String>,
}

/// 交给引擎的请求（已装配好上下文）。
#[derive(Debug, Clone, PartialEq)]
pub struct EngineRequest {
    pub prompt: String,
    pub canvas_id: Option<String>,
    pub employee_id: Option<String>,
    pub persona: Option<String>,
    pub canvas_context: Option<String>,
}

/// 引擎返回的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct EngineOutcome {
    pub success: bool,
    pub text: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub model: String,
    pub error: Option<String>,
    pub tool_calls: Vec<String>,
    pub harness_session_id: Option<String>,
}

impl Default for EngineOutcome {
    fn default() -> Self {
        Self {
            success: true,
            text: String::new(),
            tokens_input: 0,
            tokens_output: 0,
            model: DEFAULT_DIRECT_MODEL.to_string(),
            error: None,
            tool_calls: Vec::new(),
            harness_session_id: None,
        }
    }
}

/// 可插拔的引擎实现（真实实现见 `ai::engines`）。
pub trait ChatEngine: Send + Sync {
    fn engine(&self) -> AiEngine;
    fn model(&self) -> &str;
    fn run(
        &self,
        request: &EngineRequest,
        on_delta: &mut dyn FnMut(DeltaKind, &str),
    ) -> Result<EngineOutcome, AppError>;
}

/// 一次对话的摘要（命令返回值 / 审计用）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatSummary {
    pub session_id: String,
    pub engine: AiEngine,
    pub model: String,
    pub success: bool,
    pub text: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub latency_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 计数型并发闸门（`acquire` 阻塞直到有空位或超时）。
#[derive(Debug)]
pub struct CountingGate {
    limit: usize,
    active: Mutex<usize>,
    condvar: Condvar,
}

impl CountingGate {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            active: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    /// 获取一个额度；超时返回 `busy`。
    pub fn acquire(&self, timeout: Duration) -> Result<GateGuard<'_>, AppError> {
        let deadline = Instant::now() + timeout;
        let mut active = self.active.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        while *active >= self.limit {
            let now = Instant::now();
            if now >= deadline {
                return Err(AppError::new(
                    ErrorKind::Busy,
                    format!("AI 并发已满（上限 {}），请稍后再试", self.limit),
                ));
            }
            let (next, _) = self
                .condvar
                .wait_timeout(active, deadline - now)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            active = next;
        }

        *active += 1;
        Ok(GateGuard { gate: self })
    }

    pub fn active(&self) -> usize {
        *self.active.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// 额度守卫，Drop 时归还。
#[derive(Debug)]
pub struct GateGuard<'a> {
    gate: &'a CountingGate,
}

impl Drop for GateGuard<'_> {
    fn drop(&mut self) {
        let mut active = self
            .gate
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *active = active.saturating_sub(1);
        self.gate.condvar.notify_one();
    }
}

/// AI 网关。
pub struct Gateway {
    harness: Option<Arc<dyn ChatEngine>>,
    direct: Option<Arc<dyn ChatEngine>>,
    direct_gate: CountingGate,
}

/// 生成一次对话的会话 id。
pub fn new_session_id() -> String {
    format!("paint-{}", uuid::Uuid::new_v4())
}

impl Gateway {
    pub fn new(harness: Option<Arc<dyn ChatEngine>>, direct: Option<Arc<dyn ChatEngine>>) -> Self {
        Self {
            harness,
            direct,
            direct_gate: CountingGate::new(MAX_CONCURRENT_DIRECT),
        }
    }

    /// 只装 Direct（DSH 不可用时的降级形态）。
    pub fn direct_only(direct: Arc<dyn ChatEngine>) -> Self {
        Self::new(None, Some(direct))
    }

    pub fn has_engine(&self, engine: AiEngine) -> bool {
        match engine {
            AiEngine::Harness => self.harness.is_some(),
            AiEngine::Direct => self.direct.is_some(),
        }
    }

    pub fn available_engines(&self) -> Vec<AiEngine> {
        [AiEngine::Harness, AiEngine::Direct]
            .into_iter()
            .filter(|engine| self.has_engine(*engine))
            .collect()
    }

    /// 选择引擎：显式指定优先；否则按任务性质推断；再不可用就降级。
    pub fn select_engine(&self, request: &AiChatRequest) -> Result<AiEngine, AppError> {
        let preferred = match request.engine {
            Some(engine) => {
                if !self.has_engine(engine) {
                    return Err(AppError::new(
                        ErrorKind::Ai,
                        format!("{} 引擎当前不可用", engine.as_str()),
                    ));
                }
                return Ok(engine);
            }
            None => infer_engine(request),
        };

        if self.has_engine(preferred) {
            return Ok(preferred);
        }
        // 降级：harness 不可用就用 direct，反之亦然。
        let fallback = match preferred {
            AiEngine::Harness => AiEngine::Direct,
            AiEngine::Direct => AiEngine::Harness,
        };
        if self.has_engine(fallback) {
            return Ok(fallback);
        }

        Err(AppError::new(ErrorKind::Ai, "没有可用的 AI 引擎"))
    }

    /// 执行一次对话：装配上下文 → 选引擎 → 并发控制 → 流式事件 → 计量 → 审计。
    pub fn chat(
        &self,
        store: &Store,
        sink: &dyn EventSink,
        request: AiChatRequest,
    ) -> Result<AiChatSummary, AppError> {
        self.chat_with_session(store, sink, request, new_session_id())
    }

    /// 同上，但由调用方指定会话 id（命令层要先把 id 返回给前端）。
    pub fn chat_with_session(
        &self,
        store: &Store,
        sink: &dyn EventSink,
        request: AiChatRequest,
        session_id: String,
    ) -> Result<AiChatSummary, AppError> {
        let prompt = request.prompt.trim().to_string();
        if prompt.is_empty() {
            return Err(AppError::invalid_action("对话内容不能为空"));
        }
        if prompt.chars().count() > MAX_PROMPT_CHARS {
            return Err(AppError::invalid_action(format!(
                "对话内容超出 {MAX_PROMPT_CHARS} 字符上限"
            )));
        }

        let engine = self.select_engine(&request)?;
        let engine_impl = match engine {
            AiEngine::Harness => self.harness.clone(),
            AiEngine::Direct => self.direct.clone(),
        }
        .ok_or_else(|| AppError::new(ErrorKind::Ai, "AI 引擎未就绪"))?;

        let model = engine_impl.model().to_string();
        let created_at = now_ms();

        ai_repository::insert_session(
            store,
            &session_id,
            request.canvas_id.as_deref(),
            request.employee_id.as_deref(),
            engine.as_str(),
            created_at,
        )?;
        ai_repository::insert_message(store, &session_id, "user", &prompt, created_at)?;

        let canvas_context = match request.canvas_id.as_deref() {
            Some(canvas_id) => Some(build_canvas_context(&service::load_canvas(
                store, canvas_id,
            )?)),
            None => None,
        };
        // 数字员工的人设从库里读，注入提示词（headless 没有 preset registry）。
        let persona = match request.employee_id.as_deref() {
            Some(employee_id) => crate::employees::persona_of(store, employee_id)?,
            None => None,
        };
        let engine_request = EngineRequest {
            prompt: compose_prompt(&prompt, canvas_context.as_deref(), persona.as_deref()),
            canvas_id: request.canvas_id.clone(),
            employee_id: request.employee_id.clone(),
            persona,
            canvas_context,
        };

        let started = Instant::now();
        let emit_session_id = session_id.clone();
        let mut on_delta = |kind: DeltaKind, text: &str| {
            sink.emit(
                EVENT_AI_DELTA,
                json!({
                    "sessionId": emit_session_id,
                    "content": text,
                    "kind": kind.as_str(),
                }),
            );
        };

        let outcome = if engine == AiEngine::Direct {
            let guard = self.direct_gate.acquire(DIRECT_ACQUIRE_TIMEOUT)?;
            let result = engine_impl.run(&engine_request, &mut on_delta);
            drop(guard);
            result
        } else {
            engine_impl.run(&engine_request, &mut on_delta)
        };

        let latency_ms = started.elapsed().as_millis() as i64;

        let summary = match outcome {
            Ok(outcome) => {
                ai_repository::insert_message(
                    store,
                    &session_id,
                    "assistant",
                    &outcome.text,
                    now_ms(),
                )?;
                record_call(
                    store,
                    &request,
                    &session_id,
                    engine,
                    &outcome.model,
                    &prompt,
                    &outcome,
                    latency_ms,
                )?;

                if outcome.success {
                    sink.emit(
                        EVENT_AI_DONE,
                        json!({
                            "sessionId": session_id,
                            "engine": engine.as_str(),
                            "model": outcome.model,
                            "tokensInput": outcome.tokens_input,
                            "tokensOutput": outcome.tokens_output,
                            "latencyMs": latency_ms,
                            "success": true,
                        }),
                    );
                } else {
                    let message = outcome
                        .error
                        .clone()
                        .unwrap_or_else(|| "AI 调用失败".to_string());
                    sink.emit(
                        EVENT_AI_ERROR,
                        json!({
                            "sessionId": session_id,
                            "kind": "ai",
                            "message": message,
                        }),
                    );
                }

                AiChatSummary {
                    session_id,
                    engine,
                    model: outcome.model.clone(),
                    success: outcome.success,
                    text: outcome.text.clone(),
                    tokens_input: outcome.tokens_input,
                    tokens_output: outcome.tokens_output,
                    latency_ms,
                    error: outcome.error.clone(),
                }
            }
            Err(error) => {
                let failed = EngineOutcome {
                    success: false,
                    model: model.clone(),
                    error: Some(error.message.clone()),
                    ..Default::default()
                };
                record_call(
                    store, &request, &session_id, engine, &model, &prompt, &failed, latency_ms,
                )?;
                sink.emit(
                    EVENT_AI_ERROR,
                    json!({
                        "sessionId": session_id,
                        "kind": error.kind.as_str(),
                        "message": error.message,
                    }),
                );
                AiChatSummary {
                    session_id,
                    engine,
                    model,
                    success: false,
                    text: String::new(),
                    tokens_input: 0,
                    tokens_output: 0,
                    latency_ms,
                    error: Some(error.message),
                }
            }
        };

        Ok(summary)
    }
}

#[allow(clippy::too_many_arguments)]
fn record_call(
    store: &Store,
    request: &AiChatRequest,
    session_id: &str,
    engine: AiEngine,
    model: &str,
    prompt: &str,
    outcome: &EngineOutcome,
    latency_ms: i64,
) -> Result<(), AppError> {
    let call = NewAiCall {
        session_id: Some(session_id.to_string()),
        canvas_id: request.canvas_id.clone(),
        module: request.module.clone().unwrap_or_else(|| "chat".to_string()),
        employee: request.employee_id.clone(),
        task: prompt.chars().take(200).collect(),
        engine: engine.as_str().to_string(),
        model: model.to_string(),
        tokens_input: outcome.tokens_input,
        tokens_output: outcome.tokens_output,
        latency_ms,
        success: outcome.success,
        error: outcome.error.clone(),
    };
    ai_repository::insert_call(store, &call, now_ms())?;
    Ok(())
}

/// 引擎推断（请求未显式指定时）。
pub fn infer_engine(request: &AiChatRequest) -> AiEngine {
    if request.module.as_deref() == Some("employee") {
        return AiEngine::Harness;
    }
    if looks_like_canvas_task(&request.prompt) {
        return AiEngine::Harness;
    }
    AiEngine::Direct
}

/// 判断一句话是否在要求改动画布（→ 需要工具链，走 harness）。
pub fn looks_like_canvas_task(prompt: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "画", "绘制", "生成", "添加", "新建", "删除", "修改", "调整", "合并", "排列", "图层",
        "图形", "圆形", "矩形", "圆形", "线条", "文字", "颜色", "配色", "滤镜", "模糊", "锐化",
        "导出", "缩放", "旋转", "透明度", "draw", "create", "add", "remove", "delete", "layer",
        "shape", "circle", "rect", "filter", "blur", "sharpen", "color", "export", "merge",
    ];

    let lowered = prompt.to_ascii_lowercase();
    KEYWORDS
        .iter()
        .any(|keyword| lowered.contains(&keyword.to_ascii_lowercase()))
}

/// 审计与计量查询。
pub fn list_calls(store: &Store, limit: u32) -> Result<Vec<AiCall>, AppError> {
    ai_repository::list_calls(store, limit)
}

pub fn usage_summary(store: &Store, days: u32) -> Result<Vec<AiUsageRow>, AppError> {
    ai_repository::usage_summary(store, days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::RecordingSink;
    use crate::paint::service;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeEngine {
        engine: AiEngine,
        deltas: Vec<(DeltaKind, String)>,
        outcome: EngineOutcome,
        calls: AtomicUsize,
    }

    impl FakeEngine {
        fn ok(engine: AiEngine) -> Self {
            Self {
                engine,
                deltas: vec![
                    (DeltaKind::Reasoning, "想一想".to_string()),
                    (DeltaKind::Content, "画好了".to_string()),
                ],
                outcome: EngineOutcome {
                    success: true,
                    text: "画好了".to_string(),
                    tokens_input: 100,
                    tokens_output: 20,
                    model: "fake-model".to_string(),
                    ..Default::default()
                },
                calls: AtomicUsize::new(0),
            }
        }

        fn failing(engine: AiEngine) -> Self {
            Self {
                outcome: EngineOutcome {
                    success: false,
                    error: Some("模型拒绝".to_string()),
                    model: "fake-model".to_string(),
                    ..Default::default()
                },
                ..Self::ok(engine)
            }
        }
    }

    impl ChatEngine for FakeEngine {
        fn engine(&self) -> AiEngine {
            self.engine
        }

        fn model(&self) -> &str {
            "fake-model"
        }

        fn run(
            &self,
            _request: &EngineRequest,
            on_delta: &mut dyn FnMut(DeltaKind, &str),
        ) -> Result<EngineOutcome, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            for (kind, text) in &self.deltas {
                on_delta(*kind, text);
            }
            Ok(self.outcome.clone())
        }
    }

    fn store_with_canvas() -> (Store, String) {
        let store = Store::open_in_memory().expect("内存库");
        let document = service::create_canvas(&store, "草稿", 200, 100).expect("新建画布");
        (store, document.id)
    }

    fn request(prompt: &str, canvas_id: Option<&str>) -> AiChatRequest {
        AiChatRequest {
            canvas_id: canvas_id.map(str::to_string),
            employee_id: None,
            prompt: prompt.to_string(),
            engine: None,
            module: None,
        }
    }

    #[test]
    fn canvas_tasks_prefer_harness_and_small_talk_prefers_direct() {
        assert_eq!(infer_engine(&request("帮我画一个红色的圆", None)), AiEngine::Harness);
        assert_eq!(infer_engine(&request("给图层加个模糊滤镜", None)), AiEngine::Harness);
        assert_eq!(infer_engine(&request("你好，你能做什么", None)), AiEngine::Direct);
        assert_eq!(infer_engine(&request("解释一下什么是图层", None)), AiEngine::Harness);

        let mut employee = request("你好", None);
        employee.module = Some("employee".to_string());
        assert_eq!(infer_engine(&employee), AiEngine::Harness);
    }

    #[test]
    fn explicit_engine_wins_and_unavailable_engine_errors() {
        let gateway = Gateway::new(
            Some(Arc::new(FakeEngine::ok(AiEngine::Harness))),
            Some(Arc::new(FakeEngine::ok(AiEngine::Direct))),
        );

        let mut explicit = request("你好", None);
        explicit.engine = Some(AiEngine::Harness);
        assert_eq!(gateway.select_engine(&explicit).expect("选择"), AiEngine::Harness);

        let only_harness = Gateway::new(
            Some(Arc::new(FakeEngine::ok(AiEngine::Harness))),
            None,
        );
        explicit.engine = Some(AiEngine::Direct);
        let error = only_harness.select_engine(&explicit).expect_err("不可用");
        assert_eq!(error.kind, ErrorKind::Ai);
    }

    #[test]
    fn missing_preferred_engine_falls_back() {
        let gateway = Gateway::new(None, Some(Arc::new(FakeEngine::ok(AiEngine::Direct))));

        let selected = gateway
            .select_engine(&request("画一个圆", None))
            .expect("降级到 direct");

        assert_eq!(selected, AiEngine::Direct);
        assert_eq!(gateway.available_engines(), vec![AiEngine::Direct]);
    }

    #[test]
    fn chat_streams_deltas_and_records_usage() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();
        let gateway = Gateway::new(Some(Arc::new(FakeEngine::ok(AiEngine::Harness))), None);

        let summary = gateway
            .chat(&store, &sink, request("画一个红色的圆", Some(&canvas_id)))
            .expect("对话");

        assert!(summary.success);
        assert_eq!(summary.engine, AiEngine::Harness);
        assert_eq!(summary.tokens_input, 100);
        assert_eq!(summary.tokens_output, 20);
        assert_eq!(sink.count(EVENT_AI_DELTA), 2);
        assert_eq!(sink.count(EVENT_AI_DONE), 1);
        assert_eq!(sink.count(EVENT_AI_ERROR), 0);

        let deltas: Vec<_> = sink
            .events()
            .into_iter()
            .filter(|(event, _)| event == EVENT_AI_DELTA)
            .map(|(_, payload)| payload)
            .collect();
        assert_eq!(deltas[0]["kind"], json!("reasoning"));
        assert_eq!(deltas[1]["kind"], json!("content"));
        assert_eq!(deltas[1]["sessionId"], json!(summary.session_id));

        let calls = list_calls(&store, 10).expect("审计");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].engine, "harness");
        assert_eq!(calls[0].tokens_input, 100);
        assert!(calls[0].success);

        let usage = usage_summary(&store, 1).expect("汇总");
        assert_eq!(usage[0].tokens_output, 20);
    }

    #[test]
    fn failed_turn_emits_error_event_and_audits_failure() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();
        let gateway = Gateway::new(Some(Arc::new(FakeEngine::failing(AiEngine::Harness))), None);

        let summary = gateway
            .chat(&store, &sink, request("画一个圆", Some(&canvas_id)))
            .expect("对话返回摘要");

        assert!(!summary.success);
        assert_eq!(sink.count(EVENT_AI_ERROR), 1);
        assert_eq!(sink.count(EVENT_AI_DONE), 0);

        let calls = list_calls(&store, 10).expect("审计");
        assert!(!calls[0].success);
        assert_eq!(calls[0].error.as_deref(), Some("模型拒绝"));
    }

    #[test]
    fn engine_panic_free_error_path_still_audits() {
        struct Boom;
        impl ChatEngine for Boom {
            fn engine(&self) -> AiEngine {
                AiEngine::Direct
            }
            fn model(&self) -> &str {
                "boom"
            }
            fn run(
                &self,
                _request: &EngineRequest,
                _on_delta: &mut dyn FnMut(DeltaKind, &str),
            ) -> Result<EngineOutcome, AppError> {
                Err(AppError::new(ErrorKind::Timeout, "超时"))
            }
        }

        let (store, _) = store_with_canvas();
        let sink = RecordingSink::new();
        let gateway = Gateway::new(None, Some(Arc::new(Boom)));

        let summary = gateway
            .chat(&store, &sink, request("你好", None))
            .expect("摘要");

        assert!(!summary.success);
        assert_eq!(sink.count(EVENT_AI_ERROR), 1);
        assert_eq!(sink.last_payload().unwrap()["kind"], json!("timeout"));
        assert!(!list_calls(&store, 5).expect("审计")[0].success);
    }

    #[test]
    fn empty_or_oversized_prompts_are_rejected() {
        let (store, _) = store_with_canvas();
        let sink = RecordingSink::new();
        let gateway = Gateway::new(None, Some(Arc::new(FakeEngine::ok(AiEngine::Direct))));

        assert!(gateway.chat(&store, &sink, request("   ", None)).is_err());
        let too_long = "字".repeat(MAX_PROMPT_CHARS + 1);
        assert!(gateway.chat(&store, &sink, request(&too_long, None)).is_err());
    }

    #[test]
    fn canvas_context_is_attached_when_canvas_id_present() {
        struct Capture(Arc<Mutex<Option<EngineRequest>>>);
        impl ChatEngine for Capture {
            fn engine(&self) -> AiEngine {
                AiEngine::Harness
            }
            fn model(&self) -> &str {
                "capture"
            }
            fn run(
                &self,
                request: &EngineRequest,
                _on_delta: &mut dyn FnMut(DeltaKind, &str),
            ) -> Result<EngineOutcome, AppError> {
                *self.0.lock().unwrap() = Some(request.clone());
                Ok(EngineOutcome {
                    model: "capture".to_string(),
                    ..Default::default()
                })
            }
        }

        let (store, canvas_id) = store_with_canvas();
        let capture = Arc::new(Mutex::new(None));
        let gateway = Gateway::new(Some(Arc::new(Capture(capture.clone()))), None);

        gateway
            .chat(
                &store,
                &RecordingSink::new(),
                request("画个方块", Some(&canvas_id)),
            )
            .expect("对话");

        let captured = capture.lock().unwrap().clone().expect("捕获请求");
        assert!(captured.canvas_context.is_some());
        assert!(captured.prompt.contains("当前画布状态"));
        assert_eq!(captured.canvas_id.as_deref(), Some(canvas_id.as_str()));
    }

    #[test]
    fn counting_gate_enforces_limit_and_times_out() {
        let gate = CountingGate::new(2);
        let first = gate.acquire(Duration::from_millis(50)).expect("第一个");
        let second = gate.acquire(Duration::from_millis(50)).expect("第二个");
        assert_eq!(gate.active(), 2);

        let error = gate
            .acquire(Duration::from_millis(50))
            .expect_err("第三个应超时");
        assert_eq!(error.kind, ErrorKind::Busy);

        drop(first);
        assert!(gate.acquire(Duration::from_millis(50)).is_ok());
        drop(second);
    }

    #[test]
    fn gate_releases_permits_from_other_threads() {
        let gate = Arc::new(CountingGate::new(1));
        let held = gate.acquire(Duration::from_millis(10)).expect("占用");
        let gate_clone = gate.clone();

        let waiter = std::thread::spawn(move || {
            gate_clone.acquire(Duration::from_secs(2)).is_ok()
        });

        std::thread::sleep(Duration::from_millis(50));
        drop(held);
        assert!(waiter.join().expect("线程"), "释放后应能拿到额度");
    }
}
