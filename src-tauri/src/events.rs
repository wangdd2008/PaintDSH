//! 事件出口：业务逻辑只依赖 [`EventSink`]，不直接摸 `AppHandle`（engineering-contract 第 8 节）。
//!
//! 事件名与载荷结构见 engineering-contract 第 6 节；各载荷结构体随对应工单落到本模块。

use std::sync::Mutex;

use serde_json::Value;

/// 画布文档变更（载荷 = `CanvasDocument`）。
pub const EVENT_CANVAS_CHANGED: &str = "canvas://changed";
/// AI 流式增量（载荷 = `{ sessionId, content, kind }`）。
pub const EVENT_AI_DELTA: &str = "ai://delta";
/// AI 调用完成（载荷 = `{ sessionId, engine, model, tokensInput, tokensOutput, latencyMs, success }`）。
pub const EVENT_AI_DONE: &str = "ai://done";
/// AI 调用失败（载荷 = `{ sessionId, kind, message }`）。错误不走 `ai://done`。
pub const EVENT_AI_ERROR: &str = "ai://error";
/// AI 提议的高风险操作等待用户确认（载荷 = `{ canvasId, action, input, reason }`）。
pub const EVENT_AI_PENDING_OP: &str = "ai://pending-op";
/// DSH Web UI 状态（载荷 = `{ running, url?, port? }`）。
pub const EVENT_DSH_WEB_STATUS: &str = "dsh://web-status";

/// 事件发射器抽象。生产用 [`AppHandleSink`]，测试用 [`RecordingSink`]。
pub trait EventSink: Send + Sync {
    fn emit(&self, event: &str, payload: Value);
}

/// 生产实现：把事件交给 Tauri 转发到窗口。
pub struct AppHandleSink {
    app: tauri::AppHandle,
}

impl AppHandleSink {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for AppHandleSink {
    fn emit(&self, event: &str, payload: Value) {
        use tauri::Emitter;
        // 窗口已关闭时 emit 会失败；事件是尽力而为，不因为没人监听就中断业务流程。
        let _ = self.app.emit(event, payload);
    }
}

/// 测试实现：记录每次 emit 的 `(event, payload)`。
#[derive(Debug, Default)]
pub struct RecordingSink {
    events: Mutex<Vec<(String, Value)>>,
}

impl RecordingSink {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<(String, Value)>> {
        self.events.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 已记录事件的快照。
    pub fn events(&self) -> Vec<(String, Value)> {
        self.lock().clone()
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// 某个事件被 emit 的次数。
    pub fn count(&self, event: &str) -> usize {
        self.lock().iter().filter(|(name, _)| name == event).count()
    }

    /// 取走全部记录（保留 sink 可继续使用）。
    pub fn take(&self) -> Vec<(String, Value)> {
        std::mem::take(&mut *self.lock())
    }

    /// 最后一条事件的载荷。
    pub fn last_payload(&self) -> Option<Value> {
        self.lock().last().map(|(_, payload)| payload.clone())
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: &str, payload: Value) {
        self.lock().push((event.to_string(), payload));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recording_sink_keeps_event_order_and_payload() {
        let sink = RecordingSink::new();
        assert!(sink.is_empty());

        sink.emit(EVENT_CANVAS_CHANGED, json!({ "revision": 1 }));
        sink.emit(EVENT_CANVAS_CHANGED, json!({ "revision": 2 }));
        sink.emit(EVENT_AI_ERROR, json!({ "kind": "timeout" }));

        assert_eq!(sink.len(), 3);
        assert_eq!(sink.count(EVENT_CANVAS_CHANGED), 2);
        assert_eq!(sink.count(EVENT_AI_ERROR), 1);
        assert_eq!(sink.last_payload(), Some(json!({ "kind": "timeout" })));

        let events = sink.events();
        assert_eq!(events[0].0, "canvas://changed");
        assert_eq!(events[0].1, json!({ "revision": 1 }));
    }

    #[test]
    fn take_drains_recorded_events() {
        let sink = RecordingSink::new();
        sink.emit(EVENT_AI_DONE, json!({ "success": true }));

        let drained = sink.take();

        assert_eq!(drained.len(), 1);
        assert!(sink.is_empty());
    }
}
