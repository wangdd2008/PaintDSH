//! 画布状态机：纯函数式的 action reducer。
//!
//! 输入当前文档 + action + input + actor，输出**新**文档；不改数据库、不发事件、不管历史。
//! 人类与 AI 走同一条路径——状态机不区分 actor，只判断操作在当前状态下是否合法
//! （见 `GLOSSARY.md` 的「人机同权」与 `docs/engineering-contract.md` 第 4、5 节）。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, ErrorKind};
use crate::paint::document::{
    now_ms, Actor, BlendMode, CanvasDocument, FilterSpec, Layer, LayerContent, LayerMetadata,
    LayerType, Region, Transform, MAX_LAYER_NAME_LEN,
};

/// 不落快照的 action：只改 UI 选择，或本身就在移动历史游标。
pub const NON_SNAPSHOT_ACTIONS: &[&str] = &["select_layer", "select_region", "undo", "redo"];

/// 由 service 层处理的 action（状态机不实现）。
pub const HISTORY_ACTIONS: &[&str] = &["undo", "redo"];

/// 前端发来的一次操作请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action: String,
    #[serde(default = "empty_input")]
    pub input: Value,
    pub actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<u64>,
}

fn empty_input() -> Value {
    Value::Object(serde_json::Map::new())
}

impl ActionRequest {
    pub fn new(action: impl Into<String>, input: Value, actor: Actor) -> Self {
        Self {
            action: action.into(),
            input,
            actor,
            expected_revision: None,
        }
    }
}

/// 当前是否需要在操作后保存快照。
pub fn is_snapshot_worthy(action: &str) -> bool {
    !NON_SNAPSHOT_ACTIONS.contains(&action)
}

/// 乐观锁检查。
pub fn check_revision(
    document: &CanvasDocument,
    expected_revision: Option<u64>,
) -> Result<(), AppError> {
    if let Some(expected) = expected_revision {
        if expected != document.revision {
            return Err(AppError::revision_conflict(format!(
                "画布已被修改：期望 revision {expected}，当前 {}",
                document.revision
            )));
        }
    }
    Ok(())
}

/// 状态转移：返回应用 action 后的新文档（revision 已自增）。
pub fn apply(
    document: &CanvasDocument,
    action: &str,
    input: &Value,
    actor: Actor,
) -> Result<CanvasDocument, AppError> {
    if HISTORY_ACTIONS.contains(&action) {
        return Err(AppError::invalid_action(format!(
            "{action} 由历史服务处理，不应进入状态机"
        )));
    }

    let mut next = document.clone();

    match action {
        "create_layer" => create_layer(&mut next, input, actor)?,
        "delete_layer" => delete_layer(&mut next, input)?,
        "reorder_layer" => reorder_layer(&mut next, input)?,
        "set_layer_visibility" => set_layer_visibility(&mut next, input)?,
        "set_layer_property" => set_layer_property(&mut next, input)?,
        "set_layer_transform" => set_layer_transform(&mut next, input)?,
        "set_layer_content" => set_layer_content(&mut next, input)?,
        "select_layer" => select_layer(&mut next, input)?,
        "select_region" => select_region(&mut next, input)?,
        "apply_filter" => apply_filter(&mut next, input)?,
        "clear_filters" => clear_filters(&mut next, input)?,
        other => {
            return Err(AppError::invalid_action(format!("未知的 action：{other}")));
        }
    }

    next.validate()?;
    if is_snapshot_worthy(action) {
        next.bump_revision();
    }
    Ok(next)
}

// ---------------------------------------------------------------------------
// 各 action 的实现
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateLayerInput {
    #[serde(default)]
    name: Option<String>,
    #[serde(default, rename = "type")]
    layer_type: Option<String>,
    #[serde(default)]
    content: Option<LayerContent>,
    #[serde(default)]
    index: Option<usize>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

fn create_layer(document: &mut CanvasDocument, input: &Value, actor: Actor) -> Result<(), AppError> {
    let request: CreateLayerInput = parse_input(input)?;

    let layer_type = match request.layer_type.as_deref() {
        Some(value) => value.parse::<LayerType>()?,
        None => request
            .content
            .as_ref()
            .map(|content| content.layer_type())
            .unwrap_or(LayerType::Vector),
    };

    let content = match request.content {
        Some(content) => {
            content.validate()?;
            content
        }
        None => default_content(layer_type)?,
    };

    if content.layer_type() != layer_type {
        return Err(AppError::invalid_action(format!(
            "图层类型 {} 与内容类型 {} 不一致",
            layer_type.as_str(),
            content.layer_type().as_str()
        )));
    }

    let name = request
        .name
        .unwrap_or_else(|| format!("图层 {}", document.layers.len() + 1));
    let now = now_ms();
    let mut layer = Layer::new(name, content, actor, now);
    layer.metadata.prompt = request.prompt;
    layer.metadata.code = request.code;

    let index = request.index.unwrap_or(document.layers.len());
    if index > document.layers.len() {
        return Err(AppError::invalid_action(format!(
            "插入位置 {index} 越界（共 {} 个图层）",
            document.layers.len()
        )));
    }

    document.active_layer_id = layer.id.clone();
    document.layers.insert(index, layer);
    Ok(())
}

fn default_content(layer_type: LayerType) -> Result<LayerContent, AppError> {
    const EMPTY_SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";

    match layer_type {
        LayerType::Vector => Ok(LayerContent::Vector {
            svg: EMPTY_SVG.to_string(),
        }),
        LayerType::AiGenerated => Ok(LayerContent::AiCode {
            code: EMPTY_SVG.to_string(),
        }),
        LayerType::Raster => Err(AppError::invalid_action(
            "新建位图图层必须提供 dataUrl 内容",
        )),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerIdInput {
    layer_id: String,
}

fn layer_index_or_err(document: &CanvasDocument, layer_id: &str) -> Result<usize, AppError> {
    document
        .layer_index(layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{layer_id}")))
}

fn delete_layer(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerIdInput = parse_input(input)?;
    let index = layer_index_or_err(document, &request.layer_id)?;

    if document.layers.len() <= 1 {
        return Err(AppError::invalid_action("画布至少需要保留一个图层"));
    }

    document.layers.remove(index);

    if document.active_layer_id == request.layer_id {
        let fallback = index.min(document.layers.len() - 1);
        document.active_layer_id = document.layers[fallback].id.clone();
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReorderLayerInput {
    layer_id: String,
    to_index: usize,
}

fn reorder_layer(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: ReorderLayerInput = parse_input(input)?;
    let from = layer_index_or_err(document, &request.layer_id)?;

    if request.to_index >= document.layers.len() {
        return Err(AppError::invalid_action(format!(
            "目标位置 {} 越界（共 {} 个图层）",
            request.to_index,
            document.layers.len()
        )));
    }
    if from == request.to_index {
        return Ok(());
    }

    let layer = document.layers.remove(from);
    document.layers.insert(request.to_index, layer);
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VisibilityInput {
    layer_id: String,
    visible: bool,
}

fn set_layer_visibility(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: VisibilityInput = parse_input(input)?;
    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;
    layer.visible = request.visible;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerPropertyInput {
    layer_id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    opacity: Option<f32>,
    #[serde(default)]
    blend_mode: Option<String>,
    #[serde(default)]
    locked: Option<bool>,
}

fn set_layer_property(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerPropertyInput = parse_input(input)?;

    let blend_mode = match request.blend_mode.as_deref() {
        Some(value) => Some(value.parse::<BlendMode>()?),
        None => None,
    };

    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;

    if let Some(name) = request.name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::invalid_action("图层名不能为空"));
        }
        if trimmed.chars().count() > MAX_LAYER_NAME_LEN {
            return Err(AppError::invalid_action(format!(
                "图层名超出 {MAX_LAYER_NAME_LEN} 字符上限"
            )));
        }
        layer.name = trimmed.to_string();
    }
    if let Some(opacity) = request.opacity {
        if !(0.0..=1.0).contains(&opacity) || !opacity.is_finite() {
            return Err(AppError::invalid_action("图层不透明度必须在 0 到 1 之间"));
        }
        layer.opacity = opacity;
    }
    if let Some(blend_mode) = blend_mode {
        layer.blend_mode = blend_mode;
    }
    if let Some(locked) = request.locked {
        layer.locked = locked;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerTransformInput {
    layer_id: String,
    transform: Transform,
}

fn set_layer_transform(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerTransformInput = parse_input(input)?;
    request.transform.validate()?;

    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;
    layer.transform = request.transform;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerContentInput {
    layer_id: String,
    content: LayerContent,
}

fn set_layer_content(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerContentInput = parse_input(input)?;
    request.content.validate()?;

    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;
    layer.layer_type = request.content.layer_type();
    layer.content = request.content;
    Ok(())
}

fn select_layer(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerIdInput = parse_input(input)?;
    layer_index_or_err(document, &request.layer_id)?;
    document.active_layer_id = request.layer_id;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectRegionInput {
    #[serde(default)]
    region: Option<Region>,
}

fn select_region(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: SelectRegionInput = parse_input(input)?;
    if let Some(region) = request.region {
        region.validate()?;
        document.selected_region = Some(region);
    } else {
        document.selected_region = None;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyFilterInput {
    layer_id: String,
    filter: FilterSpec,
}

fn apply_filter(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: ApplyFilterInput = parse_input(input)?;
    request.filter.validate()?;

    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;

    if layer.layer_type != LayerType::Raster {
        return Err(AppError::invalid_action(
            "滤镜只能应用到 raster 图层，请先栅格化该图层",
        ));
    }

    layer.metadata.filters.push(request.filter);
    Ok(())
}

fn clear_filters(document: &mut CanvasDocument, input: &Value) -> Result<(), AppError> {
    let request: LayerIdInput = parse_input(input)?;
    let layer = document
        .layer_mut(&request.layer_id)
        .ok_or_else(|| AppError::not_found(format!("图层不存在：{}", request.layer_id)))?;
    layer.metadata.filters.clear();
    Ok(())
}

// ---------------------------------------------------------------------------

fn parse_input<T: for<'de> Deserialize<'de>>(input: &Value) -> Result<T, AppError> {
    serde_json::from_value(input.clone()).map_err(|error| {
        AppError::new(
            ErrorKind::InvalidAction,
            format!("action 参数不合法：{error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn document() -> CanvasDocument {
        CanvasDocument::new("测试画布", 800, 600).expect("新建画布")
    }

    fn background_id(document: &CanvasDocument) -> String {
        document.layers[0].id.clone()
    }

    #[test]
    fn create_layer_appends_above_and_activates() {
        let document = document();
        let next = apply(
            &document,
            "create_layer",
            &json!({ "name": "AI 红圆", "type": "aiGenerated", "content": { "type": "aiCode", "code": "<circle r=\"5\" />" }, "prompt": "红圆" }),
            Actor::Agent,
        )
        .expect("创建图层");

        assert_eq!(next.layers.len(), 2);
        assert_eq!(next.layers[1].name, "AI 红圆");
        assert_eq!(next.layers[1].layer_type, LayerType::AiGenerated);
        assert_eq!(next.active_layer_id, next.layers[1].id);
        assert_eq!(next.layers[1].metadata.created_by, Actor::Agent);
        assert_eq!(next.layers[1].metadata.prompt.as_deref(), Some("红圆"));
        assert_eq!(next.revision, 1);
    }

    #[test]
    fn create_layer_at_index_inserts_between() {
        let document = document();
        let next = apply(
            &document,
            "create_layer",
            &json!({ "name": "中层", "index": 0 }),
            Actor::User,
        )
        .expect("插入图层");

        assert_eq!(next.layers[0].name, "中层");
        assert_eq!(next.layers.len(), 2);
    }

    #[test]
    fn create_raster_layer_requires_data_url() {
        let document = document();
        let error = apply(
            &document,
            "create_layer",
            &json!({ "type": "raster" }),
            Actor::User,
        )
        .expect_err("缺少 dataUrl 应报错");

        assert_eq!(error.kind, ErrorKind::InvalidAction);
    }

    #[test]
    fn delete_layer_falls_back_to_neighbour_and_rejects_last() {
        let document = document();
        let with_two = apply(
            &document,
            "create_layer",
            &json!({ "name": "第二层" }),
            Actor::User,
        )
        .expect("创建图层");
        let active = with_two.active_layer_id.clone();

        let next = apply(&with_two, "delete_layer", &json!({ "layerId": active }), Actor::User)
            .expect("删除图层");

        assert_eq!(next.layers.len(), 1);
        assert_eq!(next.active_layer_id, background_id(&document));

        let error = apply(
            &next,
            "delete_layer",
            &json!({ "layerId": background_id(&document) }),
            Actor::User,
        )
        .expect_err("不能删除唯一图层");
        assert_eq!(error.kind, ErrorKind::InvalidAction);
    }

    #[test]
    fn reorder_layer_moves_by_index() {
        let document = document();
        let with_three = ["A", "B"]
            .iter()
            .try_fold(document, |acc, name| {
                apply(&acc, "create_layer", &json!({ "name": name }), Actor::User)
            })
            .expect("创建两个图层");

        let top_id = with_three.layers[2].id.clone();
        let next = apply(
            &with_three,
            "reorder_layer",
            &json!({ "layerId": top_id, "toIndex": 0 }),
            Actor::User,
        )
        .expect("重排图层");

        assert_eq!(next.layers[0].name, "B");
        assert!(apply(
            &next,
            "reorder_layer",
            &json!({ "layerId": top_id, "toIndex": 99 }),
            Actor::User
        )
        .is_err());
    }

    #[test]
    fn visibility_and_properties_are_updated() {
        let document = document();
        let id = background_id(&document);

        let hidden = apply(
            &document,
            "set_layer_visibility",
            &json!({ "layerId": id, "visible": false }),
            Actor::User,
        )
        .expect("切换可见性");
        assert!(!hidden.layers[0].visible);

        let renamed = apply(
            &hidden,
            "set_layer_property",
            &json!({ "layerId": id, "name": " 线稿 ", "opacity": 0.5, "blendMode": "multiply" }),
            Actor::User,
        )
        .expect("修改属性");
        assert_eq!(renamed.layers[0].name, "线稿");
        assert_eq!(renamed.layers[0].opacity, 0.5);
        assert_eq!(renamed.layers[0].blend_mode, BlendMode::Multiply);

        assert!(apply(
            &renamed,
            "set_layer_property",
            &json!({ "layerId": id, "opacity": 3.0 }),
            Actor::User
        )
        .is_err());
        assert!(apply(
            &renamed,
            "set_layer_property",
            &json!({ "layerId": id, "name": "   " }),
            Actor::User
        )
        .is_err());
    }

    #[test]
    fn set_layer_content_switches_type_with_content() {
        let document = document();
        let id = background_id(&document);

        let next = apply(
            &document,
            "set_layer_content",
            &json!({ "layerId": id, "content": { "type": "raster", "dataUrl": "data:image/png;base64,AA" } }),
            Actor::User,
        )
        .expect("写入位图内容");

        assert_eq!(next.layers[0].layer_type, LayerType::Raster);
    }

    #[test]
    fn select_actions_do_not_bump_revision() {
        let document = document();
        let id = background_id(&document);

        let selected = apply(
            &document,
            "select_layer",
            &json!({ "layerId": id }),
            Actor::User,
        )
        .expect("选中图层");
        assert_eq!(selected.revision, document.revision);

        let region = apply(
            &selected,
            "select_region",
            &json!({ "region": { "x": 1.0, "y": 2.0, "width": 10.0, "height": 20.0 } }),
            Actor::User,
        )
        .expect("选中区域");
        assert_eq!(region.revision, document.revision);
        assert!(region.selected_region.is_some());

        let cleared = apply(
            &region,
            "select_region",
            &json!({ "region": null }),
            Actor::User,
        )
        .expect("清空选区");
        assert!(cleared.selected_region.is_none());

        assert!(apply(
            &selected,
            "select_region",
            &json!({ "region": { "x": 0, "y": 0, "width": 0, "height": 10 } }),
            Actor::User
        )
        .is_err());
    }

    #[test]
    fn filters_require_raster_layer() {
        let document = document();
        let id = background_id(&document);

        let error = apply(
            &document,
            "apply_filter",
            &json!({ "layerId": id, "filter": { "kind": "blur", "radius": 4.0 } }),
            Actor::User,
        )
        .expect_err("矢量图层不能直接加滤镜");
        assert_eq!(error.kind, ErrorKind::InvalidAction);

        let raster = apply(
            &document,
            "set_layer_content",
            &json!({ "layerId": id, "content": { "type": "raster", "dataUrl": "data:image/png;base64,AA" } }),
            Actor::User,
        )
        .expect("转位图");

        let blurred = apply(
            &raster,
            "apply_filter",
            &json!({ "layerId": id, "filter": { "kind": "blur", "radius": 4.0 } }),
            Actor::User,
        )
        .expect("加滤镜");
        assert_eq!(blurred.layers[0].metadata.filters.len(), 1);

        let cleared = apply(
            &blurred,
            "clear_filters",
            &json!({ "layerId": id }),
            Actor::User,
        )
        .expect("清空滤镜");
        assert!(cleared.layers[0].metadata.filters.is_empty());
    }

    #[test]
    fn unknown_action_and_bad_input_are_rejected() {
        let document = document();

        assert_eq!(
            apply(&document, "explode", &json!({}), Actor::User)
                .expect_err("未知 action")
                .kind,
            ErrorKind::InvalidAction
        );
        assert_eq!(
            apply(&document, "delete_layer", &json!({}), Actor::User)
                .expect_err("缺少参数")
                .kind,
            ErrorKind::InvalidAction
        );
        assert_eq!(
            apply(
                &document,
                "delete_layer",
                &json!({ "layerId": "layer-nope" }),
                Actor::User
            )
            .expect_err("图层不存在")
            .kind,
            ErrorKind::NotFound
        );
    }

    #[test]
    fn undo_redo_never_enter_state_machine() {
        let document = document();
        for action in ["undo", "redo"] {
            assert_eq!(
                apply(&document, action, &json!({}), Actor::User)
                    .expect_err("历史操作不该进状态机")
                    .kind,
                ErrorKind::InvalidAction
            );
            assert!(!is_snapshot_worthy(action));
        }
        assert!(is_snapshot_worthy("create_layer"));
        assert!(!is_snapshot_worthy("select_layer"));
    }

    #[test]
    fn revision_conflict_is_detected() {
        let document = document();

        assert!(check_revision(&document, Some(document.revision)).is_ok());
        assert_eq!(
            check_revision(&document, Some(document.revision + 1))
                .expect_err("revision 不符")
                .kind,
            ErrorKind::RevisionConflict
        );
        assert!(check_revision(&document, None).is_ok());
    }

    #[test]
    fn actor_does_not_change_state_transitions() {
        let document = document();
        let input = json!({ "name": "同权图层" });

        let by_user = apply(&document, "create_layer", &input, Actor::User).expect("人类创建");
        let by_agent = apply(&document, "create_layer", &input, Actor::Agent).expect("AI 创建");

        assert_eq!(by_user.layers.len(), by_agent.layers.len());
        assert_eq!(
            by_user.layers[1].metadata.created_by,
            Actor::User
        );
        assert_eq!(by_agent.layers[1].metadata.created_by, Actor::Agent);
        assert_eq!(by_user.layers[1].content, by_agent.layers[1].content);
    }

    #[test]
    fn create_layer_defaults_name_and_content() {
        let document = document();
        let next = apply(&document, "create_layer", &json!({}), Actor::User).expect("默认图层");

        assert_eq!(next.layers[1].name, "图层 2");
        assert_eq!(next.layers[1].layer_type, LayerType::Vector);
    }

    #[test]
    fn request_parses_from_camel_case_json() {
        let request: ActionRequest = serde_json::from_value(json!({
            "action": "create_layer",
            "input": { "name": "x" },
            "actor": "agent",
            "expectedRevision": 7
        }))
        .expect("解析请求");

        assert_eq!(request.actor, Actor::Agent);
        assert_eq!(request.expected_revision, Some(7));
        assert_eq!(request.action, "create_layer");
    }

    #[test]
    fn metadata_helper_records_actor() {
        let metadata = LayerMetadata::created_by(Actor::Agent);

        assert_eq!(metadata.created_by, Actor::Agent);
        assert!(metadata.filters.is_empty());
        assert!(metadata.prompt.is_none());
    }
}
