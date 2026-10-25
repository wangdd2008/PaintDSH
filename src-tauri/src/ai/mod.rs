//! AI 网关（Harness / Direct 双引擎 + Token 计量）。
//!
//! 本工单只建立模块骨架：
//! - `gateway.rs`（引擎选择、并发信号量、`ai://delta` / `ai://done` / `ai://error`）
//!   与 `context.rs`（画布状态 → DSH 上下文）随 ticket 07、08 落地。
//!
//! 对外契约：所有 AI 事件经 `EventSink` 发出，错误单独走 `EVENT_AI_ERROR`，不得塞进 `ai://done`。
