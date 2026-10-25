//! Command Dispatcher：人类与 AI 的唯一写入口。
//!
//! 职责：路由 undo/redo、调用服务层、广播 `canvas://changed`。
//! 不区分 actor——同一套状态机、同一套事件（engineering-contract 第 5 节）。

use crate::error::AppError;
use crate::events::{EventSink, EVENT_CANVAS_CHANGED};
use crate::paint::document::CanvasDocument;
use crate::paint::service::{self, HistoryStatus};
use crate::paint::state_machine::ActionRequest;
use crate::store::Store;

/// 执行一次画布操作并把新状态推给前端。
pub fn dispatch(
    store: &Store,
    sink: &dyn EventSink,
    canvas_id: &str,
    request: ActionRequest,
) -> Result<CanvasDocument, AppError> {
    let document = match request.action.as_str() {
        "undo" => service::undo(store, canvas_id)?,
        "redo" => service::redo(store, canvas_id)?,
        _ => service::apply_action(store, canvas_id, &request)?,
    };

    emit_document(sink, &document)?;
    Ok(document)
}

/// 新建画布并广播（新建本身也是一次状态变更）。
pub fn create_canvas(
    store: &Store,
    sink: &dyn EventSink,
    name: impl Into<String>,
    width: u32,
    height: u32,
) -> Result<CanvasDocument, AppError> {
    let document = service::create_canvas(store, name, width, height)?;
    emit_document(sink, &document)?;
    Ok(document)
}

pub fn history_status(store: &Store, canvas_id: &str) -> Result<HistoryStatus, AppError> {
    service::history_status(store, canvas_id)
}

fn emit_document(sink: &dyn EventSink, document: &CanvasDocument) -> Result<(), AppError> {
    let payload = serde_json::to_value(document)?;
    sink.emit(EVENT_CANVAS_CHANGED, payload);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use crate::events::RecordingSink;
    use crate::paint::document::Actor;
    use crate::paint::state_machine::ActionRequest;
    use serde_json::json;

    fn setup() -> (Store, RecordingSink, CanvasDocument) {
        let store = Store::open_in_memory().expect("内存库");
        let sink = RecordingSink::new();
        let document = create_canvas(&store, &sink, "草稿", 400, 300).expect("新建画布");
        sink.take();
        (store, sink, document)
    }

    #[test]
    fn dispatch_emits_canvas_changed_with_latest_document() {
        let (store, sink, document) = setup();

        let next = dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new("create_layer", json!({ "name": "图层 A" }), Actor::User),
        )
        .expect("创建图层");

        assert_eq!(sink.count(EVENT_CANVAS_CHANGED), 1);
        let payload = sink.last_payload().expect("载荷");
        assert_eq!(payload["id"], json!(next.id));
        assert_eq!(payload["layers"].as_array().map(|value| value.len()), Some(2));
        assert_eq!(payload["revision"], json!(1));
    }

    #[test]
    fn agent_actions_go_through_the_same_dispatcher() {
        let (store, sink, document) = setup();

        dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new(
                "create_layer",
                json!({
                    "name": "AI 图形",
                    "type": "aiGenerated",
                    "content": { "type": "aiCode", "code": "<circle r=\"3\" />" }
                }),
                Actor::Agent,
            ),
        )
        .expect("AI 创建图层");

        let payload = sink.last_payload().expect("载荷");
        assert_eq!(payload["layers"][1]["metadata"]["createdBy"], json!("agent"));
        assert_eq!(payload["layers"][1]["type"], json!("aiGenerated"));
    }

    #[test]
    fn undo_and_redo_are_routed_and_broadcast() {
        let (store, sink, document) = setup();
        dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new("create_layer", json!({ "name": "图层 A" }), Actor::User),
        )
        .expect("创建图层");
        sink.take();

        let undone = dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new("undo", json!({}), Actor::User),
        )
        .expect("撤销");
        assert_eq!(undone.layers.len(), 1);
        assert_eq!(sink.count(EVENT_CANVAS_CHANGED), 1);

        let redone = dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new("redo", json!({}), Actor::User),
        )
        .expect("重做");
        assert_eq!(redone.layers.len(), 2);
        assert_eq!(sink.count(EVENT_CANVAS_CHANGED), 2);
    }

    #[test]
    fn failing_action_emits_nothing() {
        let (store, sink, document) = setup();

        let error = dispatch(
            &store,
            &sink,
            &document.id,
            ActionRequest::new("delete_layer", json!({ "layerId": "layer-nope" }), Actor::User),
        )
        .expect_err("图层不存在");

        assert_eq!(error.kind, ErrorKind::NotFound);
        assert!(sink.is_empty());
    }
}
