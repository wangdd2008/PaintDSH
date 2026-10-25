//! DSH Web UI 集成：启动本地 Web 服务、读取带 token 的启动 URL、品牌注入脚本。
//!
//! 关键事实（DSH 实测）：启动就绪信号是 stdout 上的一行
//! `dsh web: <authenticatedUrl>`；URL 里带进程级 token，不能写日志。
//! 鉴权用 `SameSite=Strict` cookie，所以**不要用 iframe**（第三方上下文拿不到 cookie），
//! 由 Tauri 开独立的 WebviewWindow 承载（首方上下文）。

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorKind};
use crate::harness::HarnessPaths;

/// 启动超时：等 `dsh web:` 就绪行的上限。
pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(45);

/// stdout 就绪行前缀。
pub const READY_PREFIX: &str = "dsh web:";

/// 默认监听地址（DSH 只接受具体本地 IP）。
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// 端口 0 = 让操作系统分配空闲端口。
pub const EPHEMERAL_PORT: u16 = 0;

/// Web UI 的运行信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DshWebInfo {
    /// 带 token 的启动 URL（敏感，不要写日志）。
    pub url: String,
    pub port: u16,
    pub pid: u32,
}

/// 启动命令（纯数据，便于断言）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl WebCommand {
    pub fn build(paths: &HarnessPaths, host: &str, port: u16) -> Self {
        Self {
            program: paths.node_executable.clone(),
            args: vec![
                paths.dsh_entry.to_string_lossy().into_owned(),
                "--profile".to_string(),
                "web".to_string(),
                "--no-open".to_string(),
                "--host".to_string(),
                host.to_string(),
                "--port".to_string(),
                port.to_string(),
            ],
            env: vec![
                (
                    "DSH_HOME".to_string(),
                    paths.dsh_home.to_string_lossy().into_owned(),
                ),
                ("DSH_TELEMETRY_MODE".to_string(), "DISABLED".to_string()),
            ],
        }
    }
}

/// 解析就绪行，返回 (url, port)。
pub fn parse_ready_line(line: &str) -> Option<(String, u16)> {
    let trimmed = line.trim();
    let url = trimmed.strip_prefix(READY_PREFIX)?.trim();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    let port = extract_port(url)?;
    Some((url.to_string(), port))
}

fn extract_port(url: &str) -> Option<u16> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme.split(['/', '?']).next()?;
    let (_, port) = authority.rsplit_once(':')?;
    port.parse::<u16>().ok()
}

/// 品牌注入脚本：只做"外观遮盖"，失败即静默。
///
/// 注入点由 Tauri 侧决定（`WebviewWindowBuilder::initialization_script` 或
/// `webview.eval`）。DSH 没有品牌配置面，脚本只能改 DOM/CSS 变量，跨版本可能失效。
pub fn brand_script() -> String {
    r##"(function () {
  if (window.__paintBrandApplied) return;
  window.__paintBrandApplied = true;

  var REPLACEMENTS = [
    ["DeepSeek Harness", "PAINT"],
    ["DeepSeek", "PAINT"],
    ["deepseek", "paint"]
  ];

  function rewriteText(node) {
    if (!node || !node.nodeValue) return;
    var text = node.nodeValue;
    for (var i = 0; i < REPLACEMENTS.length; i += 1) {
      text = text.split(REPLACEMENTS[i][0]).join(REPLACEMENTS[i][1]);
    }
    if (text !== node.nodeValue) node.nodeValue = text;
  }

  function walk(root) {
    if (!root) return;
    if (root.nodeType === 3) {
      rewriteText(root);
      return;
    }
    if (root.nodeType !== 1) return;
    var tag = root.tagName;
    if (tag === "SCRIPT" || tag === "STYLE") return;
    for (var child = root.firstChild; child; child = child.nextSibling) walk(child);
  }

  function apply() {
    try {
      walk(document.body);
      var root = document.documentElement;
      root.style.setProperty("--paint-accent", "#e5484d");
      document.title = document.title
        .split("DeepSeek").join("PAINT")
        .split("deepseek").join("paint");
    } catch (error) {
      /* 选择器或 DOM 结构变化时静默失败 */
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", apply, { once: true });
  } else {
    apply();
  }

  try {
    var observer = new MutationObserver(function () { apply(); });
    observer.observe(document.documentElement, { childList: true, subtree: true });
    setTimeout(function () { observer.disconnect(); }, 15000);
  } catch (error) {
    /* ignore */
  }
})();"##
        .to_string()
}

/// DSH Web UI 进程管理（单例式：同一时间只跑一个）。
#[derive(Default)]
pub struct WebUi {
    child: Arc<Mutex<Option<Child>>>,
    info: Arc<Mutex<Option<DshWebInfo>>>,
}

impl WebUi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(&self) -> Option<DshWebInfo> {
        self.info.lock().ok().and_then(|info| info.clone())
    }

    pub fn is_running(&self) -> bool {
        self.status().is_some()
    }

    /// 启动 Web 服务并等到就绪；已运行则直接返回现有信息。
    pub fn start(
        &self,
        paths: &HarnessPaths,
        host: &str,
        port: u16,
    ) -> Result<DshWebInfo, AppError> {
        if let Some(info) = self.status() {
            return Ok(info);
        }

        let command = WebCommand::build(paths, host, port);
        let mut builder = Command::new(&command.program);
        builder
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &command.env {
            builder.env(key, value);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            builder.creation_flags(crate::harness::CREATE_NO_WINDOW_FLAG);
        }

        let mut child = builder
            .spawn()
            .map_err(|error| AppError::harness(format!("启动 DSH Web 失败：{error}")))?;
        let pid = child.id();

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::harness("无法读取 DSH Web stdout"))?;
        let stderr = child.stderr.take();

        let (sender, receiver) = std::sync::mpsc::channel::<(String, Option<u16>)>();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                match parse_ready_line(&line) {
                    Some((url, port)) => {
                        let _ = sender.send((url, Some(port)));
                    }
                    None => {
                        let _ = sender.send((line, None));
                    }
                }
            }
        });

        // stderr 单独消费，避免管道写满。
        if let Some(stderr) = stderr {
            std::thread::spawn(move || {
                for _ in BufReader::new(stderr).lines().map_while(Result::ok) {}
            });
        }

        let deadline = Instant::now() + STARTUP_TIMEOUT;
        let mut last_lines: Vec<String> = Vec::new();

        loop {
            let now = Instant::now();
            if now >= deadline {
                kill_child(&mut child);
                return Err(AppError::new(
                    ErrorKind::Timeout,
                    format!(
                        "DSH Web 在 {} 秒内未就绪{}",
                        STARTUP_TIMEOUT.as_secs(),
                        if last_lines.is_empty() {
                            String::new()
                        } else {
                            format!("，最近输出：{}", last_lines.join(" | "))
                        }
                    ),
                ));
            }

            match receiver.recv_timeout(deadline - now) {
                Ok((line, Some(port))) => {
                    let info = DshWebInfo { url: line, port, pid };
                    *self.child.lock().unwrap_or_else(|p| p.into_inner()) = Some(child);
                    *self.info.lock().unwrap_or_else(|p| p.into_inner()) = Some(info.clone());
                    return Ok(info);
                }
                Ok((line, None)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if last_lines.len() >= 5 {
                        last_lines.remove(0);
                    }
                    // 注意：不把整行写日志，可能含 token。
                    last_lines.push(redact(&line));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    kill_child(&mut child);
                    return Err(AppError::harness("DSH Web 进程提前退出"));
                }
            }
        }
    }

    /// 停止 Web 服务（杀进程树）。
    pub fn stop(&self) -> Result<(), AppError> {
        let mut guard = self.child.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(mut child) = guard.take() {
            kill_child(&mut child);
        }
        *self.info.lock().unwrap_or_else(|p| p.into_inner()) = None;
        Ok(())
    }
}

impl Drop for WebUi {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn redact(line: &str) -> String {
    match line.split_once("token=") {
        Some((prefix, _)) => format!("{prefix}token=***"),
        None => line.to_string(),
    }
}

fn kill_child(child: &mut Child) {
    #[cfg(windows)]
    {
        let mut command = Command::new("taskkill");
        command
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        use std::os::windows::process::CommandExt;
        command.creation_flags(crate::harness::CREATE_NO_WINDOW_FLAG);
        let _ = command.status();
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> HarnessPaths {
        HarnessPaths {
            node_executable: PathBuf::from("node"),
            dsh_entry: PathBuf::from("C:/dsh/lib/bin.js"),
            dsh_home: PathBuf::from("C:/paint/harness"),
        }
    }

    #[test]
    fn web_command_uses_profile_web_and_no_open() {
        let command = WebCommand::build(&paths(), "127.0.0.1", 0);

        assert_eq!(command.program, PathBuf::from("node"));
        assert_eq!(
            command.args,
            vec![
                "C:/dsh/lib/bin.js",
                "--profile",
                "web",
                "--no-open",
                "--host",
                "127.0.0.1",
                "--port",
                "0",
            ]
        );
        assert!(command
            .env
            .contains(&("DSH_HOME".to_string(), "C:/paint/harness".to_string())));
    }

    #[test]
    fn ready_line_is_parsed_with_token_url() {
        let line = "dsh web: http://127.0.0.1:3080/?token=secret";

        let (url, port) = parse_ready_line(line).expect("解析");

        assert_eq!(port, 3080);
        assert!(url.contains("token=secret"));
    }

    #[test]
    fn non_ready_lines_are_ignored() {
        assert!(parse_ready_line("boot: loading plugins").is_none());
        assert!(parse_ready_line("dsh web: not-a-url").is_none());
        assert!(parse_ready_line("dsh web: http://127.0.0.1/").is_none());
    }

    #[test]
    fn port_is_extracted_from_authority() {
        assert_eq!(extract_port("http://127.0.0.1:51234/?token=x"), Some(51234));
        assert_eq!(extract_port("http://127.0.0.1:1/"), Some(1));
        assert_eq!(extract_port("http://127.0.0.1/"), None);
    }

    #[test]
    fn redact_hides_token() {
        assert_eq!(
            redact("dsh web: http://127.0.0.1:3080/?token=abc"),
            "dsh web: http://127.0.0.1:3080/?token=***"
        );
        assert_eq!(redact("plain"), "plain");
    }

    #[test]
    fn brand_script_replaces_deepseek_and_is_idempotent() {
        let script = brand_script();

        assert!(script.contains("PAINT"));
        assert!(script.contains("DeepSeek"));
        assert!(script.contains("__paintBrandApplied"));
        assert!(script.contains("MutationObserver"));
        assert!(script.trim_start().starts_with("(function"));
    }

    #[test]
    fn status_is_none_before_start() {
        let web = WebUi::new();
        assert!(web.status().is_none());
        assert!(!web.is_running());
    }
}
