//! 全应用统一错误类型。
//!
//! 所有 Tauri 命令失败时都返回 `AppError`，序列化为 `{ kind, message }`（kind 为 camelCase），
//! 前端由 `src/lib/errors.ts` 解析。后续工单不得另造错误类型——见 engineering-contract 第 3、6 节。

use serde::{Deserialize, Serialize};

/// 错误分类。**命名固定**，前端与工单都以它做分支判断。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    /// 乐观锁失败：`expectedRevision` 与当前 revision 不符。
    RevisionConflict,
    /// 资源不存在（画布、图层、会话……）。
    NotFound,
    /// Action 非法（词表外的 action、参数校验失败、状态机拒绝）。
    InvalidAction,
    /// 文件系统错误。
    Io,
    /// SQLite 错误。
    Db,
    /// AI 网关 / Direct 引擎错误。
    Ai,
    /// DSH 子进程错误。
    Harness,
    /// 超时。
    Timeout,
    /// 全局锁被占用（同一时间只允许一个 harness 任务）。
    Busy,
}

impl ErrorKind {
    /// 与 serde 输出一致的字符串形式。
    pub const fn as_str(self) -> &'static str {
        match self {
            ErrorKind::RevisionConflict => "revisionConflict",
            ErrorKind::NotFound => "notFound",
            ErrorKind::InvalidAction => "invalidAction",
            ErrorKind::Io => "io",
            ErrorKind::Db => "db",
            ErrorKind::Ai => "ai",
            ErrorKind::Harness => "harness",
            ErrorKind::Timeout => "timeout",
            ErrorKind::Busy => "busy",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 应用错误。`Serialize` 后即前端拿到的 `{ kind, message }`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{kind}: {message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn revision_conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RevisionConflict, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn invalid_action(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidAction, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message)
    }

    pub fn db(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Db, message)
    }

    pub fn ai(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Ai, message)
    }

    pub fn harness(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Harness, message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, message)
    }

    pub fn busy(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Busy, message)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, error.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::new(ErrorKind::Db, error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(ErrorKind::Io, format!("JSON 序列化失败：{error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_kind_and_message_with_camel_case_kind() {
        let error = AppError::revision_conflict("revision 5 != 6");
        let value = serde_json::to_value(&error).expect("序列化 AppError");

        assert_eq!(value["kind"], serde_json::json!("revisionConflict"));
        assert_eq!(value["message"], serde_json::json!("revision 5 != 6"));
        assert_eq!(value.as_object().map(|o| o.len()), Some(2));
    }

    #[test]
    fn every_error_kind_has_a_stable_serialized_name() {
        let cases = [
            (ErrorKind::RevisionConflict, "revisionConflict"),
            (ErrorKind::NotFound, "notFound"),
            (ErrorKind::InvalidAction, "invalidAction"),
            (ErrorKind::Io, "io"),
            (ErrorKind::Db, "db"),
            (ErrorKind::Ai, "ai"),
            (ErrorKind::Harness, "harness"),
            (ErrorKind::Timeout, "timeout"),
            (ErrorKind::Busy, "busy"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected);
            assert_eq!(serde_json::to_value(kind).expect("序列化 kind"), expected);
        }
    }

    #[test]
    fn display_includes_kind_and_message() {
        let error = AppError::not_found("画布不存在");
        assert_eq!(error.to_string(), "notFound: 画布不存在");
    }

    #[test]
    fn converts_io_and_json_errors() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "拒绝访问");
        let from_io: AppError = io_error.into();
        assert_eq!(from_io.kind, ErrorKind::Io);
        assert!(from_io.message.contains("拒绝访问"));

        let json_error = serde_json::from_str::<serde_json::Value>("{").expect_err("非法 JSON");
        let from_json: AppError = json_error.into();
        assert_eq!(from_json.kind, ErrorKind::Io);
        assert!(from_json.message.contains("JSON"));
    }
}
