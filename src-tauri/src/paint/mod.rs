//! 画布核心域（Command Dispatcher 的落点）。
//!
//! 本工单只建立模块骨架，避免提前锁定后续工单的内部结构：
//! - `document.rs` / `state_machine.rs` / `service.rs` / `command_dispatcher.rs` 随 ticket 02、03 落地。
//!
//! 对外契约（不得改动）：
//! - 所有写操作经 `dispatch(store, sink, canvas_id, request)`（engineering-contract 第 5 节）；
//! - 成功后 `sink.emit(EVENT_CANVAS_CHANGED, document)`；
//! - Rust 是画布状态的唯一真相源，前端只渲染它下发的 `CanvasDocument`。
