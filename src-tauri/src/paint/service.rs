//! 画布业务服务：状态机 + 持久化 + 历史（快照）编排。
//!
//! 这里不碰 Tauri：事件由 `command_dispatcher` 负责发，这样服务层可以在单测里
//! 直接用内存库跑完整流程（engineering-contract 第 8 节）。

use crate::error::AppError;
use crate::paint::document::{now_ms, Actor, CanvasDocument, CanvasSummary, Snapshot};
use crate::paint::state_machine::{check_revision, is_snapshot_worthy, ActionRequest};
use crate::paint::{repository, state_machine};
use crate::store::Store;

/// 新建画布并落一条初始快照（游标 0），保证 Undo 有基线。
pub fn create_canvas(
    store: &Store,
    name: impl Into<String>,
    width: u32,
    height: u32,
) -> Result<CanvasDocument, AppError> {
    let document = CanvasDocument::new(name, width, height)?;
    document.validate()?;

    repository::save_canvas(store, &document)?;
    let snapshot = Snapshot::capture("create_canvas", Actor::User, &document);
    repository::insert_snapshot(store, &document.id, 0, &snapshot)?;
    repository::set_history_cursor(store, &document.id, 0)?;

    Ok(document)
}

pub fn load_canvas(store: &Store, canvas_id: &str) -> Result<CanvasDocument, AppError> {
    repository::load_canvas(store, canvas_id)
}

pub fn list_canvases(store: &Store) -> Result<Vec<CanvasSummary>, AppError> {
    repository::list_canvases(store)
}

/// 应用一次 action：乐观锁 → 状态机 → 持久化 → 快照。
pub fn apply_action(
    store: &Store,
    canvas_id: &str,
    request: &ActionRequest,
) -> Result<CanvasDocument, AppError> {
    let current = repository::load_canvas(store, canvas_id)?;
    check_revision(&current, request.expected_revision)?;

    let next = state_machine::apply(&current, &request.action, &request.input, request.actor)?;
    let snapshot_worthy = is_snapshot_worthy(&request.action);

    repository::save_canvas(store, &next)?;
    if snapshot_worthy {
        push_snapshot(store, canvas_id, &next, &request.action, request.actor)?;
    }

    Ok(next)
}

/// 在游标之后写入新快照，并截断原来的 redo 分支。
fn push_snapshot(
    store: &Store,
    canvas_id: &str,
    document: &CanvasDocument,
    action: &str,
    actor: Actor,
) -> Result<(), AppError> {
    let cursor = repository::history_cursor(store, canvas_id)?;
    repository::truncate_snapshots_after(store, canvas_id, cursor)?;

    let next_cursor = cursor + 1;
    let snapshot = Snapshot::capture(action, actor, document);
    repository::insert_snapshot(store, canvas_id, next_cursor, &snapshot)?;
    repository::set_history_cursor(store, canvas_id, next_cursor)?;

    Ok(())
}

/// 回退一步。恢复历史内容，但 revision 继续自增（乐观锁必须单调）。
pub fn undo(store: &Store, canvas_id: &str) -> Result<CanvasDocument, AppError> {
    let current = repository::load_canvas(store, canvas_id)?;
    let cursor = repository::history_cursor(store, canvas_id)?;

    if cursor == 0 {
        return Err(AppError::invalid_action("已经回到最早的一步，无法继续撤销"));
    }

    let target = cursor - 1;
    let snapshot = repository::snapshot_at(store, canvas_id, target)?
        .ok_or_else(|| AppError::not_found(format!("历史快照缺失：cursor {target}")))?;

    let restored = restore_from(store, snapshot.state, &current)?;
    repository::set_history_cursor(store, canvas_id, target)?;
    Ok(restored)
}

/// 前进一步（仅当游标之后还有快照）。
pub fn redo(store: &Store, canvas_id: &str) -> Result<CanvasDocument, AppError> {
    let current = repository::load_canvas(store, canvas_id)?;
    let cursor = repository::history_cursor(store, canvas_id)?;
    let count = repository::snapshot_count(store, canvas_id)?;

    if cursor + 1 >= count {
        return Err(AppError::invalid_action("已经是最新的一步，无法重做"));
    }

    let target = cursor + 1;
    let snapshot = repository::snapshot_at(store, canvas_id, target)?
        .ok_or_else(|| AppError::not_found(format!("历史快照缺失：cursor {target}")))?;

    let restored = restore_from(store, snapshot.state, &current)?;
    repository::set_history_cursor(store, canvas_id, target)?;
    Ok(restored)
}

/// 把历史快照写回为当前文档：内容取快照，revision 取「当前 + 1」。
fn restore_from(
    store: &Store,
    mut state: CanvasDocument,
    current: &CanvasDocument,
) -> Result<CanvasDocument, AppError> {
    state.revision = current.revision + 1;
    state.metadata.updated_at = now_ms();
    // 历史快照里的 activeLayerId 可能指向已不存在的图层，兜底到顶层。
    if state.layer(&state.active_layer_id).is_none() {
        if let Some(top) = state.layers.last() {
            state.active_layer_id = top.id.clone();
        }
    }
    state.validate()?;

    repository::save_canvas(store, &state)?;
    Ok(state)
}

/// 当前可撤销/可重做的步数（供前端按钮态）。
pub fn history_status(store: &Store, canvas_id: &str) -> Result<HistoryStatus, AppError> {
    let cursor = repository::history_cursor(store, canvas_id)?;
    let count = repository::snapshot_count(store, canvas_id)?;

    Ok(HistoryStatus {
        cursor,
        total: count,
        can_undo: cursor > 0,
        can_redo: cursor + 1 < count,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStatus {
    pub cursor: usize,
    pub total: usize,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;
    use crate::paint::document::{LayerContent, LayerType};
    use serde_json::json;

    fn new_store() -> Store {
        Store::open_in_memory().expect("内存库")
    }

    fn request(action: &str, input: serde_json::Value) -> ActionRequest {
        ActionRequest::new(action, input, Actor::User)
    }

    #[test]
    fn create_canvas_seeds_initial_snapshot() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 640, 480).expect("新建画布");

        let status = history_status(&store, &document.id).expect("历史状态");
        assert_eq!(status.total, 1);
        assert_eq!(status.cursor, 0);
        assert!(!status.can_undo);
        assert!(!status.can_redo);
        assert_eq!(document.revision, 0);
    }

    #[test]
    fn apply_action_persists_document_and_snapshot() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 640, 480).expect("新建画布");

        let next = apply_action(
            &store,
            &document.id,
            &request("create_layer", json!({ "name": "画笔层" })),
        )
        .expect("创建图层");

        assert_eq!(next.revision, 1);
        assert_eq!(next.layers.len(), 2);

        let reloaded = load_canvas(&store, &document.id).expect("重新读取");
        assert_eq!(reloaded, next);

        let status = history_status(&store, &document.id).expect("历史状态");
        assert_eq!((status.cursor, status.total), (1, 2));
        assert!(status.can_undo);
    }

    #[test]
    fn undo_and_redo_restore_snapshot_contents_and_keep_revision_monotonic() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 640, 480).expect("新建画布");

        let with_layer = apply_action(
            &store,
            &document.id,
            &request("create_layer", json!({ "name": "第一层" })),
        )
        .expect("创建图层");
        let renamed = apply_action(
            &store,
            &document.id,
            &request(
                "set_layer_property",
                json!({ "layerId": with_layer.layers[1].id, "name": "改过名" }),
            ),
        )
        .expect("改名");

        let undone = undo(&store, &document.id).expect("撤销");
        assert_eq!(undone.layers[1].name, "第一层");
        assert_eq!(undone.revision, renamed.revision + 1);

        let undone_twice = undo(&store, &document.id).expect("再撤销");
        assert_eq!(undone_twice.layers.len(), 1);
        assert_eq!(
            undo(&store, &document.id).expect_err("已到最早").kind,
            ErrorKind::InvalidAction
        );

        let redone = redo(&store, &document.id).expect("重做");
        assert_eq!(redone.layers.len(), 2);
        assert_eq!(redone.layers[1].name, "第一层");

        let redone_twice = redo(&store, &document.id).expect("再重做");
        assert_eq!(redone_twice.layers[1].name, "改过名");
        assert_eq!(
            redo(&store, &document.id).expect_err("已到最新").kind,
            ErrorKind::InvalidAction
        );
    }

    #[test]
    fn new_action_after_undo_truncates_redo_branch() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 100, 100).expect("新建画布");

        apply_action(&store, &document.id, &request("create_layer", json!({ "name": "A" })))
            .expect("创建 A");
        apply_action(&store, &document.id, &request("create_layer", json!({ "name": "B" })))
            .expect("创建 B");

        undo(&store, &document.id).expect("撤销");
        let branched = apply_action(
            &store,
            &document.id,
            &request("create_layer", json!({ "name": "C" })),
        )
        .expect("创建 C");

        let status = history_status(&store, &document.id).expect("历史状态");
        assert_eq!(status.total, 3);
        assert!(!status.can_redo);
        assert_eq!(branched.layers.last().map(|layer| layer.name.as_str()), Some("C"));
    }

    #[test]
    fn select_actions_do_not_create_snapshots() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 100, 100).expect("新建画布");

        apply_action(
            &store,
            &document.id,
            &request("select_region", json!({ "region": { "x": 0, "y": 0, "width": 5, "height": 5 } })),
        )
        .expect("选中区域");

        let status = history_status(&store, &document.id).expect("历史状态");
        assert_eq!(status.total, 1);
        assert_eq!(status.cursor, 0);

        let reloaded = load_canvas(&store, &document.id).expect("读取");
        assert!(reloaded.selected_region.is_some());
    }

    #[test]
    fn optimistic_lock_rejects_stale_revision() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 100, 100).expect("新建画布");
        apply_action(&store, &document.id, &request("create_layer", json!({ "name": "A" })))
            .expect("创建 A");

        let mut stale = request("create_layer", json!({ "name": "B" }));
        stale.expected_revision = Some(0);

        let error = apply_action(&store, &document.id, &stale).expect_err("revision 过期");
        assert_eq!(error.kind, ErrorKind::RevisionConflict);
    }

    #[test]
    fn agent_and_user_share_the_same_pipeline() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 100, 100).expect("新建画布");

        let by_agent = apply_action(
            &store,
            &document.id,
            &ActionRequest::new(
                "create_layer",
                json!({
                    "name": "AI 图层",
                    "type": "aiGenerated",
                    "content": { "type": "aiCode", "code": "<circle r=\"9\" />" }
                }),
                Actor::Agent,
            ),
        )
        .expect("AI 创建图层");

        assert_eq!(by_agent.layers.last().unwrap().layer_type, LayerType::AiGenerated);
        assert_eq!(by_agent.layers.last().unwrap().metadata.created_by, Actor::Agent);
        assert_eq!(by_agent.revision, 1);
    }

    #[test]
    fn set_layer_content_can_rasterize_for_filters() {
        let store = new_store();
        let document = create_canvas(&store, "草稿", 100, 100).expect("新建画布");
        let layer_id = document.layers[0].id.clone();

        let raster = apply_action(
            &store,
            &document.id,
            &request(
                "set_layer_content",
                json!({ "layerId": layer_id, "content": { "type": "raster", "dataUrl": "data:image/png;base64,AA" } }),
            ),
        )
        .expect("栅格化");
        assert_eq!(raster.layers[0].content.layer_type(), LayerType::Raster);

        let filtered = apply_action(
            &store,
            &document.id,
            &request("apply_filter", json!({ "layerId": layer_id, "filter": { "kind": "blur", "radius": 2.0 } })),
        )
        .expect("加滤镜");
        assert_eq!(filtered.layers[0].metadata.filters.len(), 1);
        assert!(matches!(
            filtered.layers[0].content,
            LayerContent::Raster { .. }
        ));
    }
}
