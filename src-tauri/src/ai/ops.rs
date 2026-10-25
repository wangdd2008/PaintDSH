//! AI 画布操作协议（paint-ops）。
//!
//! DSH 的沙箱是**文件写边界**而不是命令黑名单，AI 也没有直接调用 PAINT 的能力。
//! 因此约定：AI 在最终答复里输出一段 ```paint-ops 代码块，内含 JSON 操作数组，
//! PAINT 解析后逐条走 `dispatch`（唯一写入口），非法操作整条拒绝并记录。
//!
//! 这条路径同时把安全边界收在 PAINT 自己手里：模型再怎么"想"也只能提议操作。

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{AppError, ErrorKind};
use crate::events::{EventSink, EVENT_AI_PENDING_OP};
use crate::paint::command_dispatcher;
use crate::paint::document::Actor;
use crate::paint::state_machine::ActionRequest;
use crate::store::Store;

/// 代码块语言标记。
pub const OPS_FENCE: &str = "paint-ops";

/// 单条答复里允许的操作上限（防止模型失控刷图层）。
pub const MAX_OPS_PER_REPLY: usize = 20;

/// AI 允许提议的 action 白名单。
pub const ALLOWED_ACTIONS: &[&str] = &[
    "create_layer",
    "set_layer_content",
    "set_layer_property",
    "set_layer_visibility",
    "set_layer_transform",
    "reorder_layer",
    "select_layer",
    "select_region",
    "apply_filter",
    "clear_filters",
    "delete_layer",
];

/// 需要人工确认的高风险操作。
pub const DANGEROUS_ACTIONS: &[&str] = &["delete_layer"];

/// 提案里的一条操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintOp {
    pub action: String,
    #[serde(default)]
    pub input: Value,
}

impl PaintOp {
    pub fn is_dangerous(&self) -> bool {
        DANGEROUS_ACTIONS.contains(&self.action.as_str())
    }
}

/// 高风险的待确认操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOp {
    pub action: String,
    pub input: Value,
    pub reason: String,
}

/// 执行结果。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpsReport {
    pub applied: usize,
    pub rejected: Vec<RejectedOp>,
    pub pending: Vec<PendingOp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_revision: Option<u64>,
}

/// 被拒绝的操作。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedOp {
    pub action: String,
    pub kind: String,
    pub message: String,
}

/// 从模型答复里提取 paint-ops。
///
/// 支持三种形状：`[{...}]`、`{"ops":[...]}`、单个 `{...}`。
pub fn extract_ops(text: &str) -> Result<Vec<PaintOp>, AppError> {
    let mut ops = Vec::new();

    for block in fenced_blocks(text) {
        let value: Value = serde_json::from_str(block.trim()).map_err(|error| {
            AppError::invalid_action(format!("paint-ops 不是合法 JSON：{error}"))
        })?;

        let items = match value {
            Value::Array(items) => items,
            Value::Object(mut object) => match object.remove("ops") {
                Some(Value::Array(items)) => items,
                Some(other) => vec![other],
                None => vec![Value::Object(object)],
            },
            other => vec![other],
        };

        for item in items {
            ops.push(parse_op(item)?);
        }
    }

    if ops.len() > MAX_OPS_PER_REPLY {
        return Err(AppError::invalid_action(format!(
            "单次答复最多 {MAX_OPS_PER_REPLY} 条画布操作，实际 {} 条",
            ops.len()
        )));
    }
    Ok(ops)
}

fn parse_op(value: Value) -> Result<PaintOp, AppError> {
    let op: PaintOp = serde_json::from_value(value)
        .map_err(|error| AppError::invalid_action(format!("画布操作格式错误：{error}")))?;

    if !ALLOWED_ACTIONS.contains(&op.action.as_str()) {
        return Err(AppError::new(
            ErrorKind::InvalidAction,
            format!("AI 不允许执行该操作：{}", op.action),
        ));
    }
    if !op.input.is_null() && !op.input.is_object() {
        return Err(AppError::invalid_action(format!(
            "{} 的 input 必须是对象",
            op.action
        )));
    }
    Ok(op)
}

fn fenced_blocks(text: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("```") {
        let after_fence = &rest[start + 3..];
        let Some(newline) = after_fence.find('\n') else {
            break;
        };
        let language = after_fence[..newline].trim();
        let body_start = &after_fence[newline + 1..];
        let Some(end) = body_start.find("```") else {
            break;
        };

        if language.eq_ignore_ascii_case(OPS_FENCE) {
            blocks.push(&body_start[..end]);
        }
        rest = &body_start[end + 3..];
    }

    blocks
}

/// 执行策略。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpsPolicy {
    /// 是否自动执行高风险操作（删除图层）。默认否——先让用户确认。
    pub auto_apply_dangerous: bool,
}

/// 逐条执行操作；单条失败不影响后续（都记进 `rejected`）。
pub fn apply_ops(
    store: &Store,
    sink: &dyn EventSink,
    canvas_id: &str,
    ops: &[PaintOp],
    policy: OpsPolicy,
) -> OpsReport {
    let mut report = OpsReport::default();

    for op in ops {
        if op.is_dangerous() && !policy.auto_apply_dangerous {
            let pending = PendingOp {
                action: op.action.clone(),
                input: op.input.clone(),
                reason: "高风险操作，需要用户确认".to_string(),
            };
            sink.emit(
                EVENT_AI_PENDING_OP,
                json!({
                    "canvasId": canvas_id,
                    "action": pending.action,
                    "input": pending.input,
                    "reason": pending.reason,
                }),
            );
            report.pending.push(pending);
            continue;
        }

        let request = ActionRequest::new(op.action.clone(), op.input.clone(), Actor::Agent);
        match command_dispatcher::dispatch(store, sink, canvas_id, request) {
            Ok(document) => {
                report.applied += 1;
                report.last_revision = Some(document.revision);
            }
            Err(error) => report.rejected.push(RejectedOp {
                action: op.action.clone(),
                kind: error.kind.as_str().to_string(),
                message: error.message,
            }),
        }
    }

    report
}

/// 从模型答复里提取并执行；没有 paint-ops 块时返回空报告。
pub fn apply_from_text(
    store: &Store,
    sink: &dyn EventSink,
    canvas_id: &str,
    text: &str,
    policy: OpsPolicy,
) -> Result<OpsReport, AppError> {
    let ops = extract_ops(text)?;
    if ops.is_empty() {
        return Ok(OpsReport::default());
    }
    Ok(apply_ops(store, sink, canvas_id, &ops, policy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::RecordingSink;
    use crate::paint::service;

    fn store_with_canvas() -> (Store, String) {
        let store = Store::open_in_memory().expect("内存库");
        let document = service::create_canvas(&store, "草稿", 400, 300).expect("新建画布");
        (store, document.id)
    }

    #[test]
    fn extracts_array_block() {
        let text = r#"好的，我来画：
```paint-ops
[
  {"action":"create_layer","input":{"name":"AI 圆","type":"aiGenerated","content":{"type":"aiCode","code":"<circle r=\"5\" />"}}}
]
```
完成。"#;

        let ops = extract_ops(text).expect("提取");

        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].action, "create_layer");
        assert_eq!(ops[0].input["name"], json!("AI 圆"));
    }

    #[test]
    fn extracts_wrapped_and_single_objects() {
        let wrapped = "```paint-ops\n{\"ops\":[{\"action\":\"create_layer\",\"input\":{}}]}\n```";
        assert_eq!(extract_ops(wrapped).expect("wrapped").len(), 1);

        let single = "```paint-ops\n{\"action\":\"create_layer\",\"input\":{}}\n```";
        assert_eq!(extract_ops(single).expect("single").len(), 1);
    }

    #[test]
    fn ignores_other_code_fences() {
        let text = "```json\n{\"a\":1}\n```\n```paint-ops\n[]\n```";

        assert!(extract_ops(text).expect("提取").is_empty());
    }

    #[test]
    fn rejects_unknown_actions_and_bad_json() {
        let unknown = "```paint-ops\n[{\"action\":\"rm_rf\",\"input\":{}}]\n```";
        let error = extract_ops(unknown).expect_err("未知 action");
        assert_eq!(error.kind, ErrorKind::InvalidAction);
        assert!(error.message.contains("rm_rf"));

        assert!(extract_ops("```paint-ops\n{oops\n```").is_err());
        assert!(extract_ops("```paint-ops\n[{\"action\":\"create_layer\",\"input\":3}]\n```").is_err());
    }

    #[test]
    fn caps_ops_per_reply() {
        let items: Vec<String> = (0..MAX_OPS_PER_REPLY + 1)
            .map(|_| "{\"action\":\"create_layer\",\"input\":{}}".to_string())
            .collect();
        let text = format!("```paint-ops\n[{}]\n```", items.join(","));

        assert!(extract_ops(&text).is_err());
    }

    #[test]
    fn applies_ops_through_dispatcher_and_emits_canvas_changed() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();
        let text = r#"```paint-ops
[{"action":"create_layer","input":{"name":"AI 图层","type":"aiGenerated","content":{"type":"aiCode","code":"<rect />"}}}]
```"#;

        let report = apply_from_text(&store, &sink, &canvas_id, text, OpsPolicy::default())
            .expect("执行");

        assert_eq!(report.applied, 1);
        assert!(report.rejected.is_empty());
        assert_eq!(sink.count(crate::events::EVENT_CANVAS_CHANGED), 1);

        let document = service::load_canvas(&store, &canvas_id).expect("读取");
        assert_eq!(document.layers.len(), 2);
        assert_eq!(document.layers[1].name, "AI 图层");
        assert_eq!(document.layers[1].metadata.created_by, Actor::Agent);
    }

    #[test]
    fn invalid_op_is_rejected_without_blocking_the_rest() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();
        let ops = vec![
            PaintOp {
                action: "set_layer_property".to_string(),
                input: json!({ "layerId": "layer-missing", "name": "x" }),
            },
            PaintOp {
                action: "create_layer".to_string(),
                input: json!({ "name": "还能建" }),
            },
        ];

        let report = apply_ops(&store, &sink, &canvas_id, &ops, OpsPolicy::default());

        assert_eq!(report.applied, 1);
        assert_eq!(report.rejected.len(), 1);
        assert_eq!(report.rejected[0].kind, "notFound");
    }

    #[test]
    fn dangerous_ops_wait_for_confirmation() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();
        let document = service::load_canvas(&store, &canvas_id).expect("读取");
        let ops = vec![PaintOp {
            action: "delete_layer".to_string(),
            input: json!({ "layerId": document.layers[0].id }),
        }];

        let report = apply_ops(&store, &sink, &canvas_id, &ops, OpsPolicy::default());

        assert_eq!(report.applied, 0);
        assert_eq!(report.pending.len(), 1);
        assert_eq!(sink.count(crate::events::EVENT_AI_PENDING_OP), 1);
        assert_eq!(
            service::load_canvas(&store, &canvas_id).expect("读取").layers.len(),
            1,
            "未确认前不应删除图层"
        );

        let confirmed = apply_ops(
            &store,
            &sink,
            &canvas_id,
            &ops,
            OpsPolicy {
                auto_apply_dangerous: true,
            },
        );
        assert_eq!(confirmed.applied, 0, "唯一图层不能删");
        assert_eq!(confirmed.rejected.len(), 1);
    }

    #[test]
    fn reply_without_ops_is_a_noop() {
        let (store, canvas_id) = store_with_canvas();
        let sink = RecordingSink::new();

        let report = apply_from_text(&store, &sink, &canvas_id, "今天天气不错", OpsPolicy::default())
            .expect("执行");

        assert_eq!(report, OpsReport::default());
        assert!(sink.is_empty());
    }
}
