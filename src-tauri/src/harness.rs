//! DSH headless 集成：命令拼装、`--json` 事件解析、安全沙箱 patch、超时看门狗。
//!
//! 依据实测结论（见 `docs` 里的 DSH 参考笔记）：
//! - 入口是 `node <dsh>/lib/bin.js`，不要 spawn `.cmd` 垫片；
//! - `--patch` 是唯一可靠的策略注入点，且在 task 之前；
//! - 成败看退出码，token 计量取 `status/step_end.usage`；
//! - Windows 没有 SIGKILL，超时要杀进程树。

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::fsutil::write_atomic;
use crate::paths::PaintHome;

/// 默认超时（秒）——与 DSH Platform 的看门狗口径一致。
pub const DEFAULT_TIMEOUT_SECS: u64 = 180;

/// 环境变量覆盖：DSH 入口文件。
pub const ENV_DSH_ENTRY: &str = "PAINT_DSH_ENTRY";
/// 环境变量覆盖：node 可执行文件。
pub const ENV_NODE_EXE: &str = "PAINT_NODE_EXE";
/// 环境变量覆盖：沙箱档位（`canvas-write` | `read-only`）。
pub const ENV_SANDBOX: &str = "PAINT_HARNESS_SANDBOX";

/// Windows 下隐藏控制台窗口（`CREATE_NO_WINDOW`）。
pub const CREATE_NO_WINDOW_FLAG: u32 = 0x0800_0000;

/// 全局串行锁：headless 一次只跑一个任务（同步 Mutex，跨运行时有效）。
static RUN_LOCK: Mutex<()> = Mutex::new(());

/// 运行期取消标志（`ai_cancel` 置位，看门狗下一轮轮询即杀进程）。
static CANCEL_FLAG: OnceLock<AtomicBool> = OnceLock::new();

fn cancel_flag() -> &'static AtomicBool {
    CANCEL_FLAG.get_or_init(|| AtomicBool::new(false))
}

/// 请求取消当前 headless 任务（幂等）。
pub fn request_cancel() {
    cancel_flag().store(true, Ordering::SeqCst);
}

/// DSH 沙箱档位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxPolicy {
    /// `canvas-write`：允许在工作区内写入（画布文件、导出产物）。
    CanvasWrite,
    /// `read-only`：完全只读；AI 的画布操作随答复返回。
    ReadOnly,
}

impl SandboxPolicy {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "canvas-write" | "canvaswrite" => Ok(SandboxPolicy::CanvasWrite),
            "read-only" | "readonly" => Ok(SandboxPolicy::ReadOnly),
            other => Err(AppError::invalid_action(format!(
                "未知的沙箱档位：{other}"
            ))),
        }
    }

    pub const fn mode(self) -> &'static str {
        match self {
            SandboxPolicy::CanvasWrite => "workspace-write",
            SandboxPolicy::ReadOnly => "read-only",
        }
    }

    pub const fn preset_id(self) -> &'static str {
        match self {
            SandboxPolicy::CanvasWrite => "canvas-write",
            SandboxPolicy::ReadOnly => "canvas-read-only",
        }
    }

    /// 生成注入 DSH 的 patch 文件内容。
    ///
    /// 说明：DSH 的沙箱是文件写边界，不是命令黑名单；`rm` 在工作区内是允许的。
    /// 真正的操作白名单在 PAINT 的 Command Dispatcher（见 `ai::ops`）。
    pub fn patch_yaml(self) -> String {
        format!(
            "# PAINT 沙箱策略（自动生成，请勿手改）\n\
             - id: sandbox-policy\n\
             \x20 config:\n\
             \x20   mode: {mode}\n\
             \x20   workspaceRoot: !!js process.cwd()\n\
             - id: approval\n\
             \x20 config:\n\
             \x20   policy: never\n\
             - id: permission\n\
             \x20 config:\n\
             \x20   presets:\n\
             \x20     {preset}:\n\
             \x20       sandbox: {mode}\n\
             \x20       approval: never\n\
             \x20   defaultPreset: {preset}\n\
             - id: tool-web\n\
             \x20 disabled: true\n",
            mode = self.mode(),
            preset = self.preset_id(),
        )
    }
}

/// DSH 运行所需的路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessPaths {
    pub node_executable: PathBuf,
    pub dsh_entry: PathBuf,
    pub dsh_home: PathBuf,
}

impl HarnessPaths {
    /// 解析路径：环境变量优先，其次 npm 全局安装的默认位置。
    pub fn resolve(paint_home: &PaintHome) -> Result<Self, AppError> {
        Self::resolve_with(paint_home, |key| std::env::var(key).ok())
    }

    /// 可注入的解析（测试用）。
    pub fn resolve_with<F>(paint_home: &PaintHome, lookup: F) -> Result<Self, AppError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let node_executable = match lookup(ENV_NODE_EXE) {
            Some(value) => PathBuf::from(value),
            None => default_node_executable().ok_or_else(|| {
                AppError::harness("找不到 node 可执行文件，请安装 Node.js 或设置 PAINT_NODE_EXE")
            })?,
        };

        let dsh_entry = match lookup(ENV_DSH_ENTRY) {
            Some(value) => PathBuf::from(value),
            None => default_dsh_entry().ok_or_else(|| {
                AppError::harness(
                    "找不到 DSH 入口（@deepseek-ai/dsh/lib/bin.js），请安装 deepseek-desktop 或设置 PAINT_DSH_ENTRY",
                )
            })?,
        };

        Ok(Self {
            node_executable,
            dsh_entry,
            dsh_home: paint_home.harness_home(),
        })
    }

    pub fn is_available(&self) -> bool {
        self.node_executable.exists() && self.dsh_entry.exists()
    }
}

fn default_node_executable() -> Option<PathBuf> {
    if let Some(path) = find_on_path("node") {
        return Some(path);
    }
    let fallback = PathBuf::from(r"C:\Program Files\nodejs\node.exe");
    fallback.exists().then_some(fallback)
}

fn default_dsh_entry() -> Option<PathBuf> {
    let candidate = if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata)
            .join("npm")
            .join("node_modules")
            .join("@deepseek-ai")
            .join("dsh")
            .join("lib")
            .join("bin.js")
    } else {
        PathBuf::from("@deepseek-ai/dsh/lib/bin.js")
    };
    candidate.exists().then_some(candidate)
}

fn find_on_path(executable: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let names = if cfg!(windows) {
        vec![format!("{executable}.exe"), executable.to_string()]
    } else {
        vec![executable.to_string()]
    };

    for dir in std::env::split_paths(&path_var) {
        for name in &names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 待执行的 DSH 命令（纯数据，便于断言参数顺序）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: PathBuf,
}

impl HarnessCommand {
    /// 拼装 headless 运行命令。
    ///
    /// 顺序要点：`--profile`/`--patch` 属启动器参数，必须在应用参数（`--json`、task）之前。
    pub fn headless(
        paths: &HarnessPaths,
        patch_path: &Path,
        workspace: &Path,
        task: &str,
        api_key: &str,
    ) -> Self {
        Self {
            program: paths.node_executable.clone(),
            args: vec![
                paths.dsh_entry.to_string_lossy().into_owned(),
                "--profile".to_string(),
                "headless".to_string(),
                "--patch".to_string(),
                patch_path.to_string_lossy().into_owned(),
                "--json".to_string(),
                task.to_string(),
            ],
            env: vec![
                (
                    "DSH_HOME".to_string(),
                    paths.dsh_home.to_string_lossy().into_owned(),
                ),
                ("DEEPSEEK_API_KEY".to_string(), api_key.to_string()),
                ("DSH_TELEMETRY_MODE".to_string(), "DISABLED".to_string()),
            ],
            cwd: workspace.to_path_buf(),
        }
    }

    /// 写沙箱 patch 到 `dir/paint-<policy>.patch.yml`，返回路径。
    pub fn write_patch(dir: &Path, policy: SandboxPolicy) -> Result<PathBuf, AppError> {
        let path = dir.join(format!("paint-{}.patch.yml", policy.preset_id()));
        write_atomic(&path, policy.patch_yaml().as_bytes())?;
        Ok(path)
    }
}

/// `--json` 事件流里我们关心的部分。
#[derive(Debug, Clone, PartialEq)]
pub enum RunEvent {
    Session {
        session_id: String,
        cwd: Option<String>,
    },
    Thinking {
        text: String,
    },
    Text {
        text: String,
    },
    StepUsage {
        input_tokens: i64,
        output_tokens: i64,
    },
    TurnEnd {
        reason: String,
    },
    ToolCall {
        tool: String,
    },
    Final {
        text: String,
    },
    Error {
        message: String,
    },
    Ignored,
}

/// 解析一行 `--json` 输出；无法解析时返回 `Ignored`（未来版本可能新增事件类型）。
pub fn parse_event_line(line: &str) -> RunEvent {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return RunEvent::Ignored;
    }
    let value: Value = match serde_json::from_str(trimmed) {
        Ok(value) => value,
        Err(_) => return RunEvent::Ignored,
    };

    match value.get("type").and_then(Value::as_str) {
        Some("session") => RunEvent::Session {
            session_id: value
                .get("sessionId")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            cwd: value.get("cwd").and_then(Value::as_str).map(str::to_string),
        },
        Some("thinking") => RunEvent::Thinking {
            text: value
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        Some("text") => RunEvent::Text {
            text: value
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        Some("status") => match value.get("phase").and_then(Value::as_str) {
            Some("step_end") => match value.get("usage") {
                Some(usage) => RunEvent::StepUsage {
                    input_tokens: usage
                        .get("inputTokens")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                    output_tokens: usage
                        .get("outputTokens")
                        .and_then(Value::as_i64)
                        .unwrap_or(0),
                },
                None => RunEvent::Ignored,
            },
            Some("turn_end") => RunEvent::TurnEnd {
                reason: value
                    .get("reason")
                    .and_then(|reason| reason.get("kind"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
            },
            _ => RunEvent::Ignored,
        },
        Some("tool_call") => RunEvent::ToolCall {
            tool: value
                .get("tool")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        Some("final") => RunEvent::Final {
            text: value
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        Some("error") => RunEvent::Error {
            message: value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        },
        _ => RunEvent::Ignored,
    }
}

/// 一次 headless 运行的解析结果。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HarnessOutcome {
    pub exit_code: Option<i32>,
    pub session_id: Option<String>,
    pub final_text: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub tool_calls: Vec<String>,
    pub turn_reason: Option<String>,
    pub stderr_tail: String,
    pub timed_out: bool,
    pub cancelled: bool,
    pub latency_ms: i64,
}

impl HarnessOutcome {
    /// 成功判定：退出码 0 且（若出现过）turn_end 的 reason 为 completed。
    pub fn is_success(&self) -> bool {
        if self.timed_out || self.cancelled {
            return false;
        }
        if self.exit_code != Some(0) {
            return false;
        }
        match &self.turn_reason {
            Some(reason) => reason == "completed",
            None => true,
        }
    }

    /// 失败时的可读原因（喂给前端 `ai://error`）。
    pub fn failure_message(&self) -> String {
        if self.timed_out {
            return format!("DSH 运行超时（{} 秒）", DEFAULT_TIMEOUT_SECS);
        }
        if self.cancelled {
            return "DSH 运行已取消".to_string();
        }
        if !self.stderr_tail.is_empty() {
            return self.stderr_tail.clone();
        }
        match &self.turn_reason {
            Some(reason) => format!("DSH 运行未完成：{reason}"),
            None => format!("DSH 退出码 {:?}", self.exit_code),
        }
    }
}

/// 逐行回调。
pub type LineHandler<'a> = &'a mut dyn FnMut(RunEvent);

/// 启动 DSH headless 并逐行解析输出。
pub fn run(
    command: &HarnessCommand,
    timeout: Duration,
    on_event: LineHandler<'_>,
) -> Result<HarnessOutcome, AppError> {
    let _guard = RUN_LOCK
        .try_lock()
        .map_err(|_| AppError::busy("已有 DSH 任务在运行，请稍后再试"))?;
    cancel_flag().store(false, Ordering::SeqCst);

    let started = Instant::now();
    let mut child = {
        let mut builder = Command::new(&command.program);
        builder
            .args(&command.args)
            .current_dir(&command.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &command.env {
            builder.env(key, value);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            builder.creation_flags(CREATE_NO_WINDOW_FLAG);
        }
        builder
            .spawn()
            .map_err(|error| AppError::harness(format!("启动 DSH 失败：{error}")))?
    };

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::harness("无法读取 DSH stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::harness("无法读取 DSH stderr"))?;

    // stdout 交给独立线程逐行读取：主线程必须能用 100ms 轮询超时与取消，
    // 否则子进程静默时看门狗永远不会触发。
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });

    // stderr 必须单独消费，否则管道写满会死锁。
    let stderr_handle = std::thread::spawn(move || {
        let mut tail = String::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            tail.push_str(&line);
            tail.push('\n');
            if tail.len() > 8 * 1024 {
                let cut = tail.len() - 8 * 1024;
                tail = tail[cut..].to_string();
            }
        }
        tail
    });

    let mut outcome = HarnessOutcome::default();
    let mut watchdog_triggered = false;
    let poll_slice = Duration::from_millis(100);

    loop {
        if cancel_flag().swap(false, Ordering::SeqCst) {
            outcome.cancelled = true;
            kill_tree(child.id());
            break;
        }

        let elapsed = started.elapsed();
        if elapsed > timeout {
            watchdog_triggered = true;
            kill_tree(child.id());
            break;
        }

        let slice = poll_slice.min(timeout - elapsed);
        match receiver.recv_timeout(slice) {
            Ok(line) => {
                let event = parse_event_line(&line);
                match &event {
                    RunEvent::Session { session_id, .. } => {
                        outcome.session_id = Some(session_id.clone());
                    }
                    RunEvent::StepUsage {
                        input_tokens,
                        output_tokens,
                    } => {
                        outcome.tokens_input += input_tokens;
                        outcome.tokens_output += output_tokens;
                    }
                    RunEvent::TurnEnd { reason } => {
                        outcome.turn_reason = Some(reason.clone());
                    }
                    RunEvent::ToolCall { tool } => outcome.tool_calls.push(tool.clone()),
                    RunEvent::Final { text } => outcome.final_text = text.clone(),
                    RunEvent::Error { message } => {
                        outcome.stderr_tail = message.clone();
                    }
                    _ => {}
                }
                on_event(event);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let status = child.wait().ok();
    outcome.exit_code = status.and_then(|status| status.code());
    outcome.timed_out = watchdog_triggered;
    outcome.latency_ms = started.elapsed().as_millis() as i64;
    outcome.stderr_tail = stderr_handle.join().unwrap_or_default();

    Ok(outcome)
}

/// 杀掉整个进程树（Windows 没有 SIGKILL；只 kill node 会留下 pwsh 孤儿）。
fn kill_tree(pid: u32) {
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill");
        command
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW_FLAG);
        let _ = command.status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status();
    }
}

/// 探测本机 DSH 版本（`node <entry> --version`），失败返回 `None`。
pub fn probe_version(paths: &HarnessPaths) -> Option<String> {
    let mut builder = Command::new(&paths.node_executable);
    builder
        .arg(&paths.dsh_entry)
        .arg("--version")
        .stdin(Stdio::null())
        .envs(paths_env(paths));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        builder.creation_flags(CREATE_NO_WINDOW_FLAG);
    }

    let output = builder.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!version.is_empty()).then_some(version)
}

fn paths_env(paths: &HarnessPaths) -> Vec<(String, String)> {
    vec![(
        "DSH_HOME".to_string(),
        paths.dsh_home.to_string_lossy().into_owned(),
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 进程类用例会争用全局 RUN_LOCK，串行执行避免互相干扰。
    static TEST_SERIAL: Mutex<()> = Mutex::new(());

    fn paths() -> HarnessPaths {
        HarnessPaths {
            node_executable: PathBuf::from("node"),
            dsh_entry: PathBuf::from("C:/dsh/lib/bin.js"),
            dsh_home: PathBuf::from("C:/paint/harness"),
        }
    }

    #[test]
    fn headless_command_puts_launcher_flags_before_app_flags() {
        let command = HarnessCommand::headless(
            &paths(),
            Path::new("C:/tmp/paint-canvas-write.patch.yml"),
            Path::new("C:/paint/workspaces/canvas-1"),
            "画一个红色的圆",
            "sk-test",
        );

        assert_eq!(command.program, PathBuf::from("node"));
        assert_eq!(
            command.args,
            vec![
                "C:/dsh/lib/bin.js",
                "--profile",
                "headless",
                "--patch",
                "C:/tmp/paint-canvas-write.patch.yml",
                "--json",
                "画一个红色的圆",
            ]
        );
        assert_eq!(command.cwd, PathBuf::from("C:/paint/workspaces/canvas-1"));
        assert!(command
            .env
            .contains(&("DEEPSEEK_API_KEY".to_string(), "sk-test".to_string())));
        assert!(command
            .env
            .contains(&("DSH_HOME".to_string(), "C:/paint/harness".to_string())));
        assert!(command
            .env
            .contains(&("DSH_TELEMETRY_MODE".to_string(), "DISABLED".to_string())));
    }

    #[test]
    fn canvas_write_patch_declares_policy_and_disables_web_tools() {
        let yaml = SandboxPolicy::CanvasWrite.patch_yaml();

        assert!(yaml.contains("mode: workspace-write"));
        assert!(yaml.contains("policy: never"));
        assert!(yaml.contains("canvas-write:"));
        assert!(yaml.contains("defaultPreset: canvas-write"));
        assert!(yaml.contains("id: tool-web"));
        assert!(yaml.contains("disabled: true"));
        assert!(yaml.contains("workspaceRoot: !!js process.cwd()"));
    }

    #[test]
    fn read_only_patch_switches_mode_and_preset() {
        let yaml = SandboxPolicy::ReadOnly.patch_yaml();

        assert!(yaml.contains("mode: read-only"));
        assert!(yaml.contains("canvas-read-only:"));
        assert!(yaml.contains("defaultPreset: canvas-read-only"));
        assert!(!yaml.contains("workspace-write"));
    }

    #[test]
    fn write_patch_is_idempotent_on_disk() {
        let dir = std::env::temp_dir().join(format!("paint-harness-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("创建目录");

        let first = HarnessCommand::write_patch(&dir, SandboxPolicy::CanvasWrite).expect("写 patch");
        let second = HarnessCommand::write_patch(&dir, SandboxPolicy::CanvasWrite).expect("再写");

        assert_eq!(first, second);
        let content = std::fs::read_to_string(&first).expect("读取");
        assert_eq!(content, SandboxPolicy::CanvasWrite.patch_yaml());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn json_events_are_parsed_and_tokens_accumulate() {
        let lines = [
            r#"{"type":"session","sessionId":"session-1","cwd":"C:/paint/workspaces/x"}"#,
            r#"{"type":"status","phase":"turn_start","turn":1}"#,
            r#"{"type":"thinking","text":"想一想"}"#,
            r#"{"type":"text","text":"好的"}"#,
            r#"{"type":"status","phase":"step_end","turn":1,"step":1,"usage":{"inputTokens":120,"outputTokens":30,"totalTokens":150}}"#,
            r#"{"type":"tool_call","callId":"c1","tool":"pwsh","input":{}}"#,
            r#"{"type":"status","phase":"step_end","turn":1,"step":2}"#,
            r#"{"type":"status","phase":"turn_end","turn":1,"reason":{"kind":"completed"}}"#,
            r#"{"type":"final","text":"画好了"}"#,
            "not json at all",
            r#"{"type":"future-event","payload":1}"#,
        ];
        let mut tokens = (0, 0);
        let mut session = None;
        let mut final_text = String::new();
        let mut reason = None;
        let mut tools = Vec::new();

        for line in lines {
            match parse_event_line(line) {
                RunEvent::Session { session_id, .. } => session = Some(session_id),
                RunEvent::StepUsage {
                    input_tokens,
                    output_tokens,
                } => {
                    tokens.0 += input_tokens;
                    tokens.1 += output_tokens;
                }
                RunEvent::TurnEnd { reason: kind } => reason = Some(kind),
                RunEvent::ToolCall { tool } => tools.push(tool),
                RunEvent::Final { text } => final_text = text,
                _ => {}
            }
        }

        assert_eq!(session.as_deref(), Some("session-1"));
        assert_eq!(tokens, (120, 30));
        assert_eq!(reason.as_deref(), Some("completed"));
        assert_eq!(final_text, "画好了");
        assert_eq!(tools, vec!["pwsh".to_string()]);
    }

    #[test]
    fn missing_usage_is_not_counted_as_zero() {
        assert_eq!(
            parse_event_line(r#"{"type":"status","phase":"step_end","turn":1,"step":1}"#),
            RunEvent::Ignored
        );
    }

    #[test]
    fn success_requires_zero_exit_and_completed_turn() {
        let ok = HarnessOutcome {
            exit_code: Some(0),
            turn_reason: Some("completed".to_string()),
            ..Default::default()
        };
        assert!(ok.is_success());

        let failed_turn = HarnessOutcome {
            exit_code: Some(0),
            turn_reason: Some("aborted".to_string()),
            ..Default::default()
        };
        assert!(!failed_turn.is_success());

        let nonzero = HarnessOutcome {
            exit_code: Some(1),
            ..Default::default()
        };
        assert!(!nonzero.is_success());

        let timed_out = HarnessOutcome {
            exit_code: Some(0),
            timed_out: true,
            ..Default::default()
        };
        assert!(!timed_out.is_success());
        assert!(timed_out.failure_message().contains("超时"));
    }

    #[test]
    fn sandbox_policy_parses_known_values() {
        assert_eq!(
            SandboxPolicy::parse("canvas-write").expect("解析"),
            SandboxPolicy::CanvasWrite
        );
        assert_eq!(
            SandboxPolicy::parse("READ-ONLY").expect("解析"),
            SandboxPolicy::ReadOnly
        );
        assert!(SandboxPolicy::parse("nope").is_err());
    }

    #[test]
    fn resolve_prefers_explicit_env_overrides() {
        let home = PaintHome::new("C:/paint");

        let resolved = HarnessPaths::resolve_with(&home, |key| match key {
            ENV_NODE_EXE => Some("C:/node/node.exe".to_string()),
            ENV_DSH_ENTRY => Some("C:/dsh/lib/bin.js".to_string()),
            _ => None,
        })
        .expect("解析路径");

        assert_eq!(resolved.node_executable, PathBuf::from("C:/node/node.exe"));
        assert_eq!(resolved.dsh_entry, PathBuf::from("C:/dsh/lib/bin.js"));
        assert_eq!(resolved.dsh_home, PathBuf::from("C:/paint/harness"));
    }

    /// 真实子进程：用 `type` 回放一段 NDJSON 模拟 DSH，验证逐行回调与退出码。
    #[cfg(windows)]
    #[test]
    fn run_streams_stdout_lines_and_reports_exit_code() {
        let _serial = TEST_SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let dir = std::env::temp_dir().join(format!("paint-run-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("建目录");
        let script = dir.join("events.ndjson");
        std::fs::write(&script, "{\"type\":\"final\",\"text\":\"ok\"}\r\n").expect("写脚本");

        let command = HarnessCommand {
            program: PathBuf::from("cmd"),
            // cmd /c 下不要给路径加引号：外层已有引号，嵌套会把命令拆坏。
            args: vec!["/c".to_string(), format!("type {}", script.display())],
            env: Vec::new(),
            cwd: dir.clone(),
        };

        let mut seen = Vec::new();
        let outcome = run(&command, Duration::from_secs(30), &mut |event| seen.push(event))
            .expect("运行命令");

        assert_eq!(outcome.exit_code, Some(0));
        assert!(outcome.is_success());
        assert_eq!(outcome.final_text, "ok");
        assert!(seen
            .iter()
            .any(|event| matches!(event, RunEvent::Final { .. })));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 看门狗：故意超时并杀进程树。
    #[cfg(windows)]
    #[test]
    fn watchdog_kills_long_running_child() {
        let _serial = TEST_SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        let command = HarnessCommand {
            program: PathBuf::from("cmd"),
            // stdout 重定向到 nul：让子进程长时间静默，验证看门狗不依赖输出。
            args: vec!["/c".to_string(), "ping -n 60 127.0.0.1 > nul".to_string()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
        };

        let started = Instant::now();
        let outcome = run(&command, Duration::from_millis(500), &mut |_| {}).expect("运行命令");

        assert!(outcome.timed_out);
        assert!(!outcome.is_success());
        assert!(
            started.elapsed() < Duration::from_secs(8),
            "看门狗应及时返回"
        );
    }

    /// 全局锁：第二个并发任务直接 busy。
    #[test]
    fn concurrent_runs_are_rejected_while_lock_held() {
        let _serial = TEST_SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _guard = RUN_LOCK.lock().expect("拿锁");

        let command = HarnessCommand {
            program: PathBuf::from("cmd"),
            args: vec!["/c".to_string(), "echo hi".to_string()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
        };

        let error = run(&command, Duration::from_secs(5), &mut |_| {}).expect_err("应拒绝并发");
        assert_eq!(error.kind, crate::error::ErrorKind::Busy);
    }
}
