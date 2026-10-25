//! `.paint` 文件读写与图片导出。
//!
//! `.paint` 是 JSON（画布 + 完整历史），导出走前端渲染好的 data URL：
//! Rust 只负责校验格式、解码、原子落盘（engineering-contract 第 6 节）。

use std::path::{Path, PathBuf};

use base64::Engine as _;

use crate::error::AppError;
use crate::fsutil::write_atomic;
use crate::paint::document::{now_ms, CanvasDocument, PaintFile, PAINT_FILE_VERSION};
use crate::paint::repository;
use crate::store::Store;

/// 专有格式扩展名（不带点）。
pub const PAINT_FILE_EXTENSION: &str = "paint";

/// 单个导出文件的大小上限（解码后）。
pub const MAX_EXPORT_BYTES: usize = 64 * 1024 * 1024;

/// 支持的导出格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Jpg,
    Svg,
}

impl ExportFormat {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "png" => Ok(ExportFormat::Png),
            "jpg" | "jpeg" => Ok(ExportFormat::Jpg),
            "svg" => Ok(ExportFormat::Svg),
            other => Err(AppError::invalid_action(format!(
                "不支持的导出格式：{other}"
            ))),
        }
    }

    pub const fn extension(self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Jpg => "jpg",
            ExportFormat::Svg => "svg",
        }
    }

    pub const fn mime(self) -> &'static str {
        match self {
            ExportFormat::Png => "image/png",
            ExportFormat::Jpg => "image/jpeg",
            ExportFormat::Svg => "image/svg+xml",
        }
    }
}

/// 保存为 `.paint`（路径缺少扩展名时自动补上），返回实际写入的路径。
pub fn save_paint_file(
    store: &Store,
    canvas_id: &str,
    path: impl AsRef<Path>,
) -> Result<PathBuf, AppError> {
    let document = repository::load_canvas(store, canvas_id)?;
    let history = repository::load_snapshots(store, canvas_id)?;
    let cursor = repository::history_cursor(store, canvas_id)?;

    let file = PaintFile {
        version: PAINT_FILE_VERSION,
        canvas: document,
        history,
        cursor,
    };

    let path = with_extension(path.as_ref(), PAINT_FILE_EXTENSION);
    let json = serde_json::to_vec_pretty(&file)?;
    write_atomic(&path, &json)?;
    Ok(path)
}

/// 打开 `.paint`：校验后写回数据库（同 id 画布会被覆盖），返回画布文档。
pub fn open_paint_file(store: &Store, path: impl AsRef<Path>) -> Result<CanvasDocument, AppError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
        .map_err(|error| AppError::io(format!("读取 {} 失败：{error}", path.display())))?;

    let file: PaintFile = serde_json::from_slice(&bytes).map_err(|error| {
        AppError::invalid_action(format!("不是合法的 .paint 文件：{error}"))
    })?;

    if file.version > PAINT_FILE_VERSION {
        return Err(AppError::invalid_action(format!(
            "文件版本 {} 高于当前支持的 {}，请升级 PAINT",
            file.version, PAINT_FILE_VERSION
        )));
    }

    let mut document = file.canvas;
    document.validate()?;
    document.metadata.updated_at = now_ms();

    repository::save_canvas(store, &document)?;

    repository::clear_snapshots(store, &document.id)?;
    for (index, snapshot) in file.history.iter().enumerate() {
        repository::insert_snapshot(store, &document.id, index, snapshot)?;
    }

    if file.history.is_empty() {
        // 没有历史时补一条基线，保证 Undo/Redo 状态自洽。
        let baseline = crate::paint::document::Snapshot::capture(
            "open_document",
            crate::paint::document::Actor::User,
            &document,
        );
        repository::insert_snapshot(store, &document.id, 0, &baseline)?;
        repository::set_history_cursor(store, &document.id, 0)?;
    } else {
        let cursor = file.cursor.min(file.history.len() - 1);
        repository::set_history_cursor(store, &document.id, cursor)?;
    }

    Ok(document)
}

/// 导出图片：`data_url` 由前端 Fabric.js 生成，这里校验格式后原子写盘。
pub fn export_image(
    path: impl AsRef<Path>,
    format: ExportFormat,
    data_url: &str,
) -> Result<PathBuf, AppError> {
    let (mime, bytes) = decode_data_url(data_url)?;

    if mime != format.mime() {
        return Err(AppError::invalid_action(format!(
            "导出格式与内容不一致：期望 {}，实际 {mime}",
            format.mime()
        )));
    }
    if bytes.is_empty() {
        return Err(AppError::invalid_action("导出内容为空"));
    }
    if bytes.len() > MAX_EXPORT_BYTES {
        return Err(AppError::invalid_action(format!(
            "导出内容超过上限（{} 字节）",
            MAX_EXPORT_BYTES
        )));
    }

    let path = with_extension(path.as_ref(), format.extension());
    write_atomic(&path, &bytes)?;
    Ok(path)
}

/// 解析 `data:<mime>;base64,<payload>`。
pub fn decode_data_url(data_url: &str) -> Result<(String, Vec<u8>), AppError> {
    let trimmed = data_url.trim();
    let rest = trimmed
        .strip_prefix("data:")
        .ok_or_else(|| AppError::invalid_action("导出内容必须是 data URL"))?;

    let (meta, payload) = rest
        .split_once(',')
        .ok_or_else(|| AppError::invalid_action("data URL 缺少逗号分隔的内容"))?;

    let (mime, encoding) = meta
        .split_once(';')
        .ok_or_else(|| AppError::invalid_action("data URL 缺少 base64 编码声明"))?;

    if !encoding.eq_ignore_ascii_case("base64") {
        return Err(AppError::invalid_action(
            "data URL 必须使用 base64 编码（Fabric.js 输出即为 base64）",
        ));
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.trim())
        .map_err(|error| AppError::invalid_action(format!("base64 解码失败：{error}")))?;

    Ok((mime.trim().to_ascii_lowercase(), bytes))
}

fn with_extension(path: &Path, extension: &str) -> PathBuf {
    let has_expected = path
        .extension()
        .map(|value| value.to_string_lossy().eq_ignore_ascii_case(extension))
        .unwrap_or(false);

    if has_expected {
        return path.to_path_buf();
    }

    let mut owned = path.to_path_buf();
    let file_name = owned
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "未命名作品".to_string());
    owned.set_file_name(format!("{file_name}.{extension}"));
    owned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::document::{Actor, LayerContent, LayerType};
    use crate::paint::service;
    use crate::paint::state_machine::ActionRequest;
    use serde_json::json;

    fn new_store() -> Store {
        let store = Store::open_in_memory().expect("内存库");
        store.migrate().expect("迁移");
        store
    }

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("paint-files-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("创建临时目录");
        dir
    }

    #[test]
    fn save_and_open_round_trips_document_with_history() {
        let store = new_store();
        let document = service::create_canvas(&store, "作品", 300, 200).expect("新建");
        service::apply_action(
            &store,
            &document.id,
            &ActionRequest::new("create_layer", json!({ "name": "图层 A" }), Actor::User),
        )
        .expect("创建图层");

        let dir = temp_dir();
        let path = save_paint_file(&store, &document.id, dir.join("作品")).expect("保存");
        assert_eq!(path.extension().unwrap(), "paint");
        assert!(path.exists());

        // 换一个干净的库来验证"打开即恢复"
        let fresh = new_store();
        let reopened = open_paint_file(&fresh, &path).expect("打开");

        assert_eq!(reopened.id, document.id);
        assert_eq!(reopened.layers.len(), 2);
        let status = service::history_status(&fresh, &reopened.id).expect("历史");
        assert_eq!(status.total, 2);
        assert_eq!(status.cursor, 1);
        assert!(status.can_undo);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_rejects_newer_file_version() {
        let store = new_store();
        let dir = temp_dir();
        let path = dir.join("future.paint");
        let document = CanvasDocument::new("x", 10, 10).expect("新建");
        let file = PaintFile {
            version: PAINT_FILE_VERSION + 1,
            canvas: document,
            history: Vec::new(),
            cursor: 0,
        };
        std::fs::write(&path, serde_json::to_vec(&file).expect("序列化")).expect("写文件");

        let error = open_paint_file(&store, &path).expect_err("版本过高应拒绝");
        assert!(error.message.contains("版本"), "message = {}", error.message);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_rejects_broken_json_and_missing_files() {
        let store = new_store();
        let dir = temp_dir();
        let broken = dir.join("broken.paint");
        std::fs::write(&broken, b"{not json").expect("写文件");

        assert!(open_paint_file(&store, &broken).is_err());
        assert!(open_paint_file(&store, dir.join("nope.paint")).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_without_history_seeds_baseline_snapshot() {
        let store = new_store();
        let dir = temp_dir();
        let path = dir.join("no-history.paint");
        let document = CanvasDocument::new("无历史", 10, 10).expect("新建");
        let file = PaintFile {
            version: PAINT_FILE_VERSION,
            canvas: document,
            history: Vec::new(),
            cursor: 0,
        };
        std::fs::write(&path, serde_json::to_vec(&file).expect("序列化")).expect("写文件");

        let opened = open_paint_file(&store, &path).expect("打开");
        let status = service::history_status(&store, &opened.id).expect("历史");

        assert_eq!(status.total, 1);
        assert_eq!(status.cursor, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_writes_decoded_bytes_with_expected_extension() {
        let dir = temp_dir();
        let data_url = "data:image/png;base64,iVBORw0KGgo=";

        let path = export_image(dir.join("out"), ExportFormat::Png, data_url).expect("导出");

        assert_eq!(path.extension().unwrap(), "png");
        assert_eq!(std::fs::read(&path).expect("读取"), b"\x89PNG\r\n\x1a\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_rejects_format_mismatch_and_plain_svg_payload() {
        let dir = temp_dir();

        let error = export_image(
            dir.join("out"),
            ExportFormat::Jpg,
            "data:image/png;base64,iVBORw0KGgo=",
        )
        .expect_err("格式不一致");
        assert!(error.message.contains("不一致"), "message = {}", error.message);

        assert!(export_image(
            dir.join("out"),
            ExportFormat::Svg,
            "data:image/svg+xml,<svg></svg>"
        )
        .is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn svg_export_accepts_base64_payload() {
        let dir = temp_dir();
        let payload = base64::engine::general_purpose::STANDARD.encode(b"<svg></svg>");

        let path = export_image(
            dir.join("vector.svg"),
            ExportFormat::Svg,
            &format!("data:image/svg+xml;base64,{payload}"),
        )
        .expect("导出 SVG");

        assert_eq!(std::fs::read(&path).expect("读取"), b"<svg></svg>");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_format_parsing_is_case_insensitive() {
        assert_eq!(ExportFormat::parse("PNG").expect("png"), ExportFormat::Png);
        assert_eq!(ExportFormat::parse("jpeg").expect("jpeg"), ExportFormat::Jpg);
        assert!(ExportFormat::parse("webp").is_err());
    }

    #[test]
    fn raster_layer_survives_round_trip() {
        let store = new_store();
        let document = service::create_canvas(&store, "位图", 10, 10).expect("新建");
        service::apply_action(
            &store,
            &document.id,
            &ActionRequest::new(
                "set_layer_content",
                json!({ "layerId": document.layers[0].id, "content": { "type": "raster", "dataUrl": "data:image/png;base64,AA" } }),
                Actor::User,
            ),
        )
        .expect("转位图");

        let dir = temp_dir();
        let path = save_paint_file(&store, &document.id, dir.join("raster.paint")).expect("保存");
        let opened = open_paint_file(&new_store(), &path).expect("打开");

        assert_eq!(opened.layers[0].layer_type, LayerType::Raster);
        assert!(matches!(opened.layers[0].content, LayerContent::Raster { .. }));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
