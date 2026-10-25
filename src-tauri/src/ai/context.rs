//! 上下文装配：把画布状态压成紧凑 JSON，拼进给 AI 的提示词。
//!
//! 只放「AI 需要看的东西」：尺寸、背景、图层清单、选中状态。
//! 图层内容长文本会被截断，避免把整幅画塞进上下文（`docs/architecture-design.md` Q11）。

use serde_json::json;

use crate::paint::document::{CanvasDocument, LayerContent};

/// 上下文里最多描述的图层数（自顶向下取，越靠上对 AI 越重要）。
pub const MAX_CONTEXT_LAYERS: usize = 50;

/// 单个图层内容摘录的字符上限。
pub const MAX_CONTENT_EXCERPT_CHARS: usize = 400;

/// 把画布文档压成紧凑 JSON 字符串。
pub fn build_canvas_context(document: &CanvasDocument) -> String {
    let layers: Vec<_> = document
        .layers
        .iter()
        .rev()
        .take(MAX_CONTEXT_LAYERS)
        .map(|layer| {
            json!({
                "id": layer.id,
                "name": layer.name,
                "type": layer.layer_type.as_str(),
                "visible": layer.visible,
                "locked": layer.locked,
                "opacity": layer.opacity,
                "blendMode": layer.blend_mode,
                "excerpt": content_excerpt(&layer.content),
            })
        })
        .collect();

    json!({
        "canvasId": document.id,
        "name": document.name,
        "width": document.width,
        "height": document.height,
        "backgroundColor": document.background_color,
        "activeLayerId": document.active_layer_id,
        "selectedRegion": document.selected_region,
        "revision": document.revision,
        "layerCount": document.layers.len(),
        "layers": layers,
    })
    .to_string()
}

fn content_excerpt(content: &LayerContent) -> String {
    let raw = match content {
        LayerContent::Raster { data_url } => {
            // data URL 的 base64 部分对模型没意义，只给格式和大小。
            return format!("data-url({} bytes)", data_url.len());
        }
        LayerContent::Vector { svg } => svg,
        LayerContent::AiCode { code } => code,
    };

    truncate_chars(raw.trim(), MAX_CONTENT_EXCERPT_CHARS)
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let truncated: String = value.chars().take(limit).collect();
    format!("{truncated}…（已截断）")
}

/// 拼装最终提示词：人设 → 画布上下文 → 用户请求。
pub fn compose_prompt(user_prompt: &str, canvas_context: Option<&str>, persona: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(persona) = persona.map(str::trim).filter(|value| !value.is_empty()) {
        parts.push(persona.to_string());
    }

    if let Some(context) = canvas_context {
        parts.push(format!("当前画布状态（JSON）：\n{context}"));
    }

    parts.push(user_prompt.trim().to_string());
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::document::{Actor, CanvasDocument, Layer, LayerContent};

    fn document_with_layers() -> CanvasDocument {
        let mut document = CanvasDocument::new("示例", 800, 600).expect("新建");
        document.layers.push(Layer::new(
            "AI 圆",
            LayerContent::AiCode {
                code: "<circle cx=\"10\" cy=\"10\" r=\"5\" />".to_string(),
            },
            Actor::Agent,
            crate::paint::document::now_ms(),
        ));
        document
    }

    #[test]
    fn context_lists_layers_top_down_with_excerpts() {
        let document = document_with_layers();
        let context = build_canvas_context(&document);

        assert!(context.contains("\"canvasId\""));
        assert!(context.contains("\"layerCount\":2"));
        // 顶层（后加入的 AI 图层）在前
        let circle = context.find("AI 圆").expect("包含 AI 图层");
        let background = context.find("背景").expect("包含背景图层");
        assert!(circle < background);
        assert!(context.contains("<circle"));
    }

    #[test]
    fn raster_layers_only_expose_size() {
        let mut document = document_with_layers();
        document.layers[0].content = LayerContent::Raster {
            data_url: "data:image/png;base64,AAAA".to_string(),
        };

        let context = build_canvas_context(&document);

        assert!(context.contains("data-url(26 bytes)"));
        assert!(!context.contains("base64,AAAA"));
    }

    #[test]
    fn long_content_is_truncated() {
        let mut document = CanvasDocument::new("长内容", 10, 10).expect("新建");
        document.layers[0].content = LayerContent::Vector {
            svg: "x".repeat(MAX_CONTENT_EXCERPT_CHARS * 2),
        };

        let context = build_canvas_context(&document);

        assert!(context.contains("已截断"));
        assert!(
            context.len() < MAX_CONTENT_EXCERPT_CHARS * 4,
            "上下文应被截断到可控长度，实际 {}",
            context.len()
        );
    }

    #[test]
    fn compose_prompt_orders_persona_context_and_request() {
        let prompt = compose_prompt("画一个圆", Some("{\"width\":800}"), Some("你是图形设计师"));

        let persona_at = prompt.find("你是图形设计师").expect("人设");
        let context_at = prompt.find("当前画布状态").expect("上下文");
        let request_at = prompt.find("画一个圆").expect("请求");

        assert!(persona_at < context_at);
        assert!(context_at < request_at);
    }

    #[test]
    fn compose_prompt_without_context_keeps_request_only() {
        assert_eq!(compose_prompt("你好", None, None), "你好");
        assert_eq!(compose_prompt("  你好  ", None, Some("  ")), "你好");
    }
}
