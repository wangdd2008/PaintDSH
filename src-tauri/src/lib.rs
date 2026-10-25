//! PAINT 后端库入口。
//!
//! 本工单只建立骨架：模块声明、`ping` 探活命令、Tauri Builder 装配。
//! 业务命令随各工单加入 `invoke_handler`（engineering-contract 第 6 节）。

pub mod ai;
pub mod error;
pub mod events;
pub mod paint;
pub mod paths;
pub mod store;

use serde::{Deserialize, Serialize};

pub use error::{AppError, ErrorKind};

/// `ping` 命令的返回体，与 `src/lib/types.ts` 的 `PingResponse` 对应。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub pong: bool,
    pub version: String,
}

/// 构造 ping 响应的纯函数（便于单测，不依赖 Tauri 运行时）。
pub fn ping_response() -> PingResponse {
    PingResponse {
        pong: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// 探活命令：前端启动时调用，用来确认 IPC 是否通。
///
/// 注意：命令放在子模块里。`#[tauri::command]` 会 `#[macro_export]` 一个
/// `__cmd__<name>` 宏；定义在 crate 根时，`#[macro_export]` 的落点与 `pub use`
/// 会撞在同一个宏命名空间（E0255），放进子模块即可避开。
pub mod commands {
    use super::PingResponse;

    #[tauri::command]
    pub fn ping() -> PingResponse {
        crate::ping_response()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("PAINT 启动失败");
}

#[cfg(test)]
mod tests {
    // 注意：不要写 `use super::*;`——`#[tauri::command]` 会生成 `__cmd__ping` 宏，
    // glob 导入会把它重复导入同一个模块，触发 E0255。
    use super::ping_response;

    #[test]
    fn ping_returns_pong_with_crate_version() {
        let response = ping_response();

        assert!(response.pong);
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert!(!response.version.is_empty());
    }

    #[test]
    fn ping_response_serializes_with_camel_case_fields() {
        let value = serde_json::to_value(ping_response()).expect("序列化 PingResponse");

        assert_eq!(value["pong"], serde_json::json!(true));
        assert_eq!(value["version"], serde_json::json!(env!("CARGO_PKG_VERSION")));
    }
}
