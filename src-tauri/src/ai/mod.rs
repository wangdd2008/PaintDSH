//! AI 网关（Harness / Direct 双引擎 + Token 计量 + paint-ops 协议）。
//!
//! - `gateway`：引擎选择、并发闸门、`ai://delta` / `ai://done` / `ai://error`；
//! - `engines`：DSH headless 与 DeepSeek 直连两个真实实现；
//! - `context`：画布状态 → DSH 上下文；
//! - `ops`：解析模型答复里的 ```paint-ops 并走 Command Dispatcher；
//! - `repository`：`ai_sessions` / `ai_messages` / `ai_calls` 的 SQL。
//!
//! 对外契约：所有 AI 事件经 `EventSink` 发出，错误单独走 `EVENT_AI_ERROR`，不得塞进 `ai://done`。

pub mod context;
pub mod engines;
pub mod gateway;
pub mod ops;
pub mod repository;
