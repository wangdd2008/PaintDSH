//! 画布文档模型：图层、快照、`.paint` 文件格式。
//!
//! 字段名与 `src/lib/types.ts` 严格对应（camelCase；图层类型字段序列化为 `type`）。
//! 约定见 `docs/engineering-contract.md` 第 3 节。

use serde::{Deserialize, Serialize};

/// `.paint` 文件格式版本。
pub const PAINT_FILE_VERSION: u32 = 1;

/// 图层名长度上限（UTF-8 字符数）。
pub const MAX_LAYER_NAME_LEN: usize = 100;

/// 单个矢量图层内容（SVG / AI 代码）的字节上限。
pub const MAX_VECTOR_CONTENT_BYTES: usize = 2 * 1024 * 1024;

/// 单个位图图层 data URL 的字节上限。
pub const MAX_RASTER_DATA_URL_BYTES: usize = 32 * 1024 * 1024;

/// 画布尺寸上限（像素）。
pub const MAX_CANVAS_DIMENSION: u32 = 16384;

/// 操作来源。人类与 AI 共用同一套状态机，只有这个字段不同。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    User,
    Agent,
}

impl Actor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Actor::User => "user",
            Actor::Agent => "agent",
        }
    }
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Actor {
    type Err = crate::error::AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "user" => Ok(Actor::User),
            "agent" => Ok(Actor::Agent),
            other => Err(crate::error::AppError::invalid_action(format!(
                "未知的 actor：{other}"
            ))),
        }
    }
}

/// 图层种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayerType {
    Raster,
    Vector,
    AiGenerated,
}

impl LayerType {
    pub const fn as_str(self) -> &'static str {
        match self {
            LayerType::Raster => "raster",
            LayerType::Vector => "vector",
            LayerType::AiGenerated => "aiGenerated",
        }
    }
}

impl std::str::FromStr for LayerType {
    type Err = crate::error::AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "raster" => Ok(LayerType::Raster),
            "vector" => Ok(LayerType::Vector),
            "aiGenerated" | "ai_generated" | "ai" => Ok(LayerType::AiGenerated),
            other => Err(crate::error::AppError::invalid_action(format!(
                "未知的图层类型：{other}"
            ))),
        }
    }
}

/// 混合模式（Fabric.js 的 `globalCompositeOperation` 子集）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
}

impl std::str::FromStr for BlendMode {
    type Err = crate::error::AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" => Ok(BlendMode::Normal),
            "multiply" => Ok(BlendMode::Multiply),
            "screen" => Ok(BlendMode::Screen),
            "overlay" => Ok(BlendMode::Overlay),
            other => Err(crate::error::AppError::invalid_action(format!(
                "未知的混合模式：{other}"
            ))),
        }
    }
}

/// 选中区域（画布坐标，像素）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Region {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        let finite = [self.x, self.y, self.width, self.height]
            .iter()
            .all(|value| value.is_finite());
        if !finite || self.width <= 0.0 || self.height <= 0.0 {
            return Err(crate::error::AppError::invalid_action(
                "选中区域必须是有限数值且宽高大于 0",
            ));
        }
        Ok(())
    }
}

/// 图层内容。三种形态互斥，序列化时以 `type` 判别。
///
/// 注意：`rename_all` 只作用于变体名，变体里的**字段**要靠 `rename_all_fields`
/// 才会变成 camelCase（否则是 `data_url`，与前端类型对不上）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LayerContent {
    Raster { data_url: String },
    Vector { svg: String },
    AiCode { code: String },
}

impl LayerContent {
    /// 内容对应的图层种类。
    pub const fn layer_type(&self) -> LayerType {
        match self {
            LayerContent::Raster { .. } => LayerType::Raster,
            LayerContent::Vector { .. } => LayerType::Vector,
            LayerContent::AiCode { .. } => LayerType::AiGenerated,
        }
    }

    /// 结构校验：位图必须是 data URL，矢量内容不能为空且不得超限。
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        use crate::error::AppError;

        match self {
            LayerContent::Raster { data_url } => {
                if !data_url.starts_with("data:image/") {
                    return Err(AppError::invalid_action(
                        "位图图层内容必须是 data:image/ 开头的 data URL",
                    ));
                }
                if data_url.len() > MAX_RASTER_DATA_URL_BYTES {
                    return Err(AppError::invalid_action(format!(
                        "位图图层内容超过上限（{} 字节）",
                        MAX_RASTER_DATA_URL_BYTES
                    )));
                }
            }
            LayerContent::Vector { svg } | LayerContent::AiCode { code: svg } => {
                if svg.trim().is_empty() {
                    return Err(AppError::invalid_action("图层内容不能为空"));
                }
                if svg.len() > MAX_VECTOR_CONTENT_BYTES {
                    return Err(AppError::invalid_action(format!(
                        "图层内容超过上限（{} 字节）",
                        MAX_VECTOR_CONTENT_BYTES
                    )));
                }
            }
        }
        Ok(())
    }
}

/// 图层变换（相对画布左上角）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation: f64,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

impl Transform {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        let finite = [self.x, self.y, self.scale_x, self.scale_y, self.rotation]
            .iter()
            .all(|value| value.is_finite());
        if !finite {
            return Err(crate::error::AppError::invalid_action(
                "图层变换包含非法数值",
            ));
        }
        if self.scale_x.abs() < f64::EPSILON || self.scale_y.abs() < f64::EPSILON {
            return Err(crate::error::AppError::invalid_action("图层缩放不能为 0"));
        }
        Ok(())
    }
}

/// 滤镜参数。声明式元数据：由前端按 spec 重绘（见 engineering-contract 第 3 节）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FilterSpec {
    Blur {
        radius: f32,
    },
    Sharpen {
        amount: f32,
    },
    AdjustColor {
        brightness: f32,
        contrast: f32,
        saturation: f32,
        hue: f32,
    },
}

impl FilterSpec {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        use crate::error::AppError;

        let in_range = |value: f32, low: f32, high: f32| value.is_finite() && (low..=high).contains(&value);

        match *self {
            FilterSpec::Blur { radius } => {
                if !in_range(radius, 0.0, 100.0) {
                    return Err(AppError::invalid_action("模糊半径必须在 0 到 100 之间"));
                }
            }
            FilterSpec::Sharpen { amount } => {
                if !in_range(amount, 0.0, 10.0) {
                    return Err(AppError::invalid_action("锐化强度必须在 0 到 10 之间"));
                }
            }
            FilterSpec::AdjustColor {
                brightness,
                contrast,
                saturation,
                hue,
            } => {
                for (label, value, low, high) in [
                    ("亮度", brightness, -1.0, 1.0),
                    ("对比度", contrast, -1.0, 1.0),
                    ("饱和度", saturation, -1.0, 1.0),
                    ("色相", hue, -180.0, 180.0),
                ] {
                    if !in_range(value, low, high) {
                        return Err(AppError::invalid_action(format!(
                            "{label}必须在 {low} 到 {high} 之间"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

/// 图层元数据：谁创建的、AI 用的提示词、已应用的滤镜。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerMetadata {
    pub created_by: Actor,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default)]
    pub filters: Vec<FilterSpec>,
}

impl LayerMetadata {
    pub fn created_by(actor: Actor) -> Self {
        Self {
            created_by: actor,
            created_at: now_ms(),
            prompt: None,
            code: None,
            filters: Vec::new(),
        }
    }
}

/// 一个图层。`layers[0]` 在最底层。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub layer_type: LayerType,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub transform: Transform,
    pub content: LayerContent,
    pub metadata: LayerMetadata,
}

impl Layer {
    pub fn new(
        name: impl Into<String>,
        content: LayerContent,
        actor: Actor,
        now: i64,
    ) -> Self {
        let layer_type = content.layer_type();
        Self {
            id: new_layer_id(),
            name: name.into(),
            layer_type,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            transform: Transform::default(),
            content,
            metadata: LayerMetadata {
                created_by: actor,
                created_at: now,
                prompt: None,
                code: None,
                filters: Vec::new(),
            },
        }
    }

    fn validate(&self) -> Result<(), crate::error::AppError> {
        use crate::error::AppError;

        let name = self.name.trim();
        if name.is_empty() {
            return Err(AppError::invalid_action("图层名不能为空"));
        }
        if name.chars().count() > MAX_LAYER_NAME_LEN {
            return Err(AppError::invalid_action(format!(
                "图层名超出 {MAX_LAYER_NAME_LEN} 字符上限"
            )));
        }
        if !(0.0..=1.0).contains(&self.opacity) || !self.opacity.is_finite() {
            return Err(AppError::invalid_action("图层不透明度必须在 0 到 1 之间"));
        }
        if self.layer_type != self.content.layer_type() {
            return Err(AppError::invalid_action(format!(
                "图层类型 {} 与内容类型 {} 不一致",
                self.layer_type.as_str(),
                self.content.layer_type().as_str()
            )));
        }
        self.content.validate()?;
        self.transform.validate()?;
        for filter in &self.metadata.filters {
            filter.validate()?;
        }
        Ok(())
    }
}

/// 文档元信息。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMetadata {
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u32,
}

/// 画布文档：前后端唯一的画布状态表示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasDocument {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub layers: Vec<Layer>,
    pub active_layer_id: String,
    #[serde(default)]
    pub selected_region: Option<Region>,
    pub revision: u64,
    pub metadata: DocumentMetadata,
}

impl CanvasDocument {
    /// 新建一个空白画布：一个默认空矢量图层。
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Result<Self, crate::error::AppError> {
        use crate::error::AppError;

        if width == 0 || height == 0 {
            return Err(AppError::invalid_action("画布尺寸必须大于 0"));
        }
        if width > MAX_CANVAS_DIMENSION || height > MAX_CANVAS_DIMENSION {
            return Err(AppError::invalid_action(format!(
                "画布尺寸不能超过 {MAX_CANVAS_DIMENSION}"
            )));
        }

        let now = now_ms();
        // 空白图层用空 SVG 文档占位，保证内容永远可被前端解析。
        let background = Layer::new(
            "背景",
            LayerContent::Vector {
                svg: "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_string(),
            },
            Actor::User,
            now,
        );

        Ok(Self {
            id: new_canvas_id(),
            name: normalize_document_name(name),
            width,
            height,
            background_color: "#ffffff".to_string(),
            active_layer_id: background.id.clone(),
            layers: vec![background],
            selected_region: None,
            revision: 0,
            metadata: DocumentMetadata {
                created_at: now,
                updated_at: now,
                version: PAINT_FILE_VERSION,
            },
        })
    }

    pub fn layer_index(&self, layer_id: &str) -> Option<usize> {
        self.layers.iter().position(|layer| layer.id == layer_id)
    }

    pub fn layer(&self, layer_id: &str) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == layer_id)
    }

    pub fn layer_mut(&mut self, layer_id: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|layer| layer.id == layer_id)
    }

    /// 结构校验（保存前、打开文件后、加载自数据库后都要跑）。
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        use crate::error::AppError;

        if self.width == 0 || self.height == 0 {
            return Err(AppError::invalid_action("画布尺寸必须大于 0"));
        }
        if self.layers.is_empty() {
            return Err(AppError::invalid_action("画布至少需要一个图层"));
        }

        let mut seen = std::collections::HashSet::with_capacity(self.layers.len());
        for layer in &self.layers {
            layer.validate()?;
            if !seen.insert(layer.id.as_str()) {
                return Err(AppError::invalid_action(format!(
                    "图层 id 重复：{}",
                    layer.id
                )));
            }
        }

        if self.layer(&self.active_layer_id).is_none() {
            return Err(AppError::invalid_action("activeLayerId 指向不存在的图层"));
        }
        Ok(())
    }

    /// 更新 `updatedAt` 并自增 revision。
    pub fn bump_revision(&mut self) {
        self.revision += 1;
        self.metadata.updated_at = now_ms();
    }
}

/// 一次操作后的完整状态快照，用于快照级 Undo/Redo。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub id: String,
    pub revision: u64,
    pub action: String,
    pub actor: Actor,
    pub timestamp: i64,
    pub state: CanvasDocument,
}

impl Snapshot {
    pub fn capture(action: &str, actor: Actor, state: &CanvasDocument) -> Self {
        Self {
            id: new_snapshot_id(),
            revision: state.revision,
            action: action.to_string(),
            actor,
            timestamp: now_ms(),
            state: state.clone(),
        }
    }
}

/// `.paint` 文件的顶层结构。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaintFile {
    pub version: u32,
    pub canvas: CanvasDocument,
    pub history: Vec<Snapshot>,
    pub cursor: usize,
}

/// 画布列表项（不含图层内容，供「打开最近作品」列表用）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasSummary {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub revision: u64,
    pub layer_count: u32,
    pub updated_at: i64,
}

/// 当前时间（Unix 毫秒）。
pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn new_canvas_id() -> String {
    format!("canvas-{}", uuid::Uuid::new_v4())
}

pub fn new_layer_id() -> String {
    format!("layer-{}", uuid::Uuid::new_v4())
}

pub fn new_snapshot_id() -> String {
    format!("snap-{}", uuid::Uuid::new_v4())
}

/// 文档名规整：去空白、限长、空名兜底。
pub fn normalize_document_name(name: impl Into<String>) -> String {
    let trimmed = name.into().trim().to_string();
    if trimmed.is_empty() {
        return "未命名作品".to_string();
    }
    trimmed.chars().take(120).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_layer() -> Layer {
        Layer::new(
            "图层 1",
            LayerContent::Vector {
                svg: "<circle cx=\"10\" cy=\"10\" r=\"5\" />".to_string(),
            },
            Actor::User,
            now_ms(),
        )
    }

    #[test]
    fn layer_serializes_type_field_and_camel_case() {
        let layer = sample_layer();
        let value = serde_json::to_value(&layer).expect("序列化图层");

        assert_eq!(value["type"], json!("vector"));
        assert_eq!(value["blendMode"], json!("normal"));
        assert_eq!(value["metadata"]["createdBy"], json!("user"));
        assert!(value.get("layer_type").is_none());
        assert!(value.get("blend_mode").is_none());
        assert_eq!(value["content"]["type"], json!("vector"));
        assert!(value["content"]["svg"].is_string());
    }

    #[test]
    fn raster_content_serializes_data_url() {
        let content = LayerContent::Raster {
            data_url: "data:image/png;base64,AAAA".to_string(),
        };
        let value = serde_json::to_value(&content).expect("序列化内容");

        assert_eq!(value["type"], json!("raster"));
        assert_eq!(value["dataUrl"], json!("data:image/png;base64,AAAA"));
    }

    #[test]
    fn ai_generated_layer_type_is_camel_case() {
        let layer = Layer::new(
            "AI 图层",
            LayerContent::AiCode {
                code: "<rect />".to_string(),
            },
            Actor::Agent,
            now_ms(),
        );
        let value = serde_json::to_value(&layer).expect("序列化图层");

        assert_eq!(value["type"], json!("aiGenerated"));
        assert_eq!(value["metadata"]["createdBy"], json!("agent"));
    }

    #[test]
    fn document_round_trips_through_json() {
        let document = CanvasDocument::new("测试", 800, 600).expect("新建画布");
        let json = serde_json::to_string(&document).expect("序列化");
        let parsed: CanvasDocument = serde_json::from_str(&json).expect("反序列化");

        assert_eq!(parsed, document);
        assert_eq!(parsed.layers.len(), 1);
        assert_eq!(parsed.active_layer_id, document.layers[0].id);
    }

    #[test]
    fn transform_and_content_are_validated() {
        let mut layer = sample_layer();
        layer.opacity = 1.5;
        assert!(layer.validate().is_err(), "不透明度越界应报错");

        let mut layer = sample_layer();
        layer.content = LayerContent::Raster {
            data_url: "https://example.com/x.png".to_string(),
        };
        layer.layer_type = LayerType::Raster;
        assert!(layer.validate().is_err(), "非 data URL 应报错");

        let mut layer = sample_layer();
        layer.transform.scale_x = 0.0;
        assert!(layer.validate().is_err(), "零缩放应报错");
    }

    #[test]
    fn layer_type_must_match_content() {
        let mut layer = sample_layer();
        layer.layer_type = LayerType::Raster;

        let error = layer.validate().expect_err("类型不一致应报错");
        assert_eq!(error.kind, crate::error::ErrorKind::InvalidAction);
    }

    #[test]
    fn empty_document_name_falls_back() {
        assert_eq!(normalize_document_name("   "), "未命名作品");
        assert_eq!(normalize_document_name("  草稿  "), "草稿");
    }

    #[test]
    fn bump_revision_increments_and_touches_updated_at() {
        let mut document = CanvasDocument::new("测试", 100, 100).expect("新建画布");
        let before = document.metadata.updated_at;

        document.bump_revision();

        assert_eq!(document.revision, 1);
        assert!(document.metadata.updated_at >= before);
    }

    #[test]
    fn filter_specs_validate_ranges() {
        assert!(FilterSpec::Blur { radius: 8.0 }.validate().is_ok());
        assert!(FilterSpec::Blur { radius: -1.0 }.validate().is_err());
        assert!(FilterSpec::Sharpen { amount: 3.0 }.validate().is_ok());
        assert!(FilterSpec::AdjustColor {
            brightness: 0.2,
            contrast: 0.1,
            saturation: -0.3,
            hue: 90.0,
        }
        .validate()
        .is_ok());
        assert!(FilterSpec::AdjustColor {
            brightness: 2.0,
            contrast: 0.0,
            saturation: 0.0,
            hue: 0.0,
        }
        .validate()
        .is_err());

        let value = serde_json::to_value(FilterSpec::AdjustColor {
            brightness: 0.0,
            contrast: 0.0,
            saturation: 0.0,
            hue: 0.0,
        })
        .expect("序列化滤镜");
        assert_eq!(value["kind"], json!("adjustColor"));
    }
}
