//! 画布核心域（Command Dispatcher 的落点）。
//!
//! 分层：`document`（数据模型）→ `state_machine`（纯 reducer）→ `service`（持久化 + 历史）
//! → `command_dispatcher`（唯一写入口 + 事件广播）；`repository` 收全部 SQL，`files` 管 `.paint`
//! 与导出落盘。
//!
//! 对外契约（不得改动）：
//! - 所有写操作经 `dispatch(store, sink, canvas_id, request)`（engineering-contract 第 5 节）；
//! - 成功后 `sink.emit(EVENT_CANVAS_CHANGED, document)`；
//! - Rust 是画布状态的唯一真相源，前端只渲染它下发的 `CanvasDocument`。

pub mod command_dispatcher;
pub mod document;
pub mod files;
pub mod repository;
pub mod service;
pub mod state_machine;
