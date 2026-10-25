//! 画布数据的 SQLite 仓储（`canvases` / `layers` / `snapshots` 三张表）。
//!
//! 所有 SQL 都收在本模块，`Store` 只负责连接与迁移。表结构见 [`CANVAS_MIGRATIONS`]，
//! 由 `store::MIGRATIONS` 在打开数据库时顺序执行（engineering-contract 第 7 节）。

use rusqlite::{params, OptionalExtension};

use crate::error::AppError;
use crate::paint::document::{CanvasDocument, CanvasSummary, Layer, Snapshot};
use crate::store::Store;

/// `canvases` / `layers` / `snapshots` 的建表语句（幂等）。
pub const CANVAS_MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS canvases (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  background_color TEXT NOT NULL,
  active_layer_id TEXT NOT NULL,
  selected_region_json TEXT,
  revision INTEGER NOT NULL DEFAULT 0,
  history_cursor INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS layers (
  id TEXT PRIMARY KEY,
  canvas_id TEXT NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
  z_index INTEGER NOT NULL,
  layer_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_layers_canvas_z ON layers(canvas_id, z_index);

CREATE TABLE IF NOT EXISTS snapshots (
  canvas_id TEXT NOT NULL REFERENCES canvases(id) ON DELETE CASCADE,
  cursor INTEGER NOT NULL,
  snapshot_json TEXT NOT NULL,
  PRIMARY KEY (canvas_id, cursor)
);
"#;

/// 写入（或整体覆盖）一个画布：`canvases` 行 + 全部 `layers` 行。
///
/// 图层整表替换——文档规模在上百图层以内，简单可靠优先于增量 diff。
pub fn save_canvas(store: &Store, document: &CanvasDocument) -> Result<(), AppError> {
    let conn = store.connection();
    let tx = conn.unchecked_transaction()?;

    let selected_region_json = match &document.selected_region {
        Some(region) => Some(serde_json::to_string(region)?),
        None => None,
    };
    let metadata_json = serde_json::to_string(&document.metadata)?;

    tx.execute(
        "INSERT INTO canvases (
            id, name, width, height, background_color, active_layer_id,
            selected_region_json, revision, history_cursor, metadata_json, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            COALESCE((SELECT history_cursor FROM canvases WHERE id = ?1), 0),
            ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            width = excluded.width,
            height = excluded.height,
            background_color = excluded.background_color,
            active_layer_id = excluded.active_layer_id,
            selected_region_json = excluded.selected_region_json,
            revision = excluded.revision,
            metadata_json = excluded.metadata_json,
            updated_at = excluded.updated_at",
        params![
            document.id,
            document.name,
            document.width,
            document.height,
            document.background_color,
            document.active_layer_id,
            selected_region_json,
            document.revision as i64,
            metadata_json,
            document.metadata.created_at,
            document.metadata.updated_at,
        ],
    )?;

    tx.execute("DELETE FROM layers WHERE canvas_id = ?1", params![document.id])?;
    {
        let mut statement = tx.prepare(
            "INSERT INTO layers (id, canvas_id, z_index, layer_json) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (index, layer) in document.layers.iter().enumerate() {
            let layer_json = serde_json::to_string(layer)?;
            statement.execute(params![layer.id, document.id, index as i64, layer_json])?;
        }
    }

    tx.commit()?;
    Ok(())
}

/// 读取一个画布；不存在时报 `notFound`。
pub fn load_canvas(store: &Store, canvas_id: &str) -> Result<CanvasDocument, AppError> {
    let conn = store.connection();

    let row = conn
        .query_row(
            "SELECT id, name, width, height, background_color, active_layer_id,
                    selected_region_json, revision, metadata_json
             FROM canvases WHERE id = ?1",
            params![canvas_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| AppError::not_found(format!("画布不存在：{canvas_id}")))?;

    let mut statement =
        conn.prepare("SELECT layer_json FROM layers WHERE canvas_id = ?1 ORDER BY z_index ASC")?;
    let layers = statement
        .query_map(params![canvas_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|json| serde_json::from_str::<Layer>(&json))
        .collect::<Result<Vec<_>, _>>()?;

    let selected_region = match row.6 {
        Some(json) => Some(serde_json::from_str(&json)?),
        None => None,
    };
    let metadata = serde_json::from_str(&row.8)?;

    Ok(CanvasDocument {
        id: row.0,
        name: row.1,
        width: u32::try_from(row.2)
            .map_err(|_| AppError::db(format!("画布宽度非法：{}", row.2)))?,
        height: u32::try_from(row.3)
            .map_err(|_| AppError::db(format!("画布高度非法：{}", row.3)))?,
        background_color: row.4,
        layers,
        active_layer_id: row.5,
        selected_region,
        revision: u64::try_from(row.7)
            .map_err(|_| AppError::db(format!("画布 revision 非法：{}", row.7)))?,
        metadata,
    })
}

/// 画布列表（按更新时间倒序）。
pub fn list_canvases(store: &Store) -> Result<Vec<CanvasSummary>, AppError> {
    let conn = store.connection();
    let mut statement = conn.prepare(
        "SELECT c.id, c.name, c.width, c.height, c.revision, c.updated_at,
                (SELECT COUNT(*) FROM layers l WHERE l.canvas_id = c.id)
         FROM canvases c ORDER BY c.updated_at DESC",
    )?;

    let rows = statement
        .query_map([], |row| {
            Ok(CanvasSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                width: row.get::<_, i64>(2)? as u32,
                height: row.get::<_, i64>(3)? as u32,
                revision: row.get::<_, i64>(4)? as u64,
                updated_at: row.get(5)?,
                layer_count: row.get::<_, i64>(6)? as u32,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn delete_canvas(store: &Store, canvas_id: &str) -> Result<(), AppError> {
    let conn = store.connection();
    conn.execute("DELETE FROM canvases WHERE id = ?1", params![canvas_id])?;
    Ok(())
}

/// 历史游标（`snapshots.cursor` 的下标）。
pub fn history_cursor(store: &Store, canvas_id: &str) -> Result<usize, AppError> {
    let conn = store.connection();
    let cursor: Option<i64> = conn
        .query_row(
            "SELECT history_cursor FROM canvases WHERE id = ?1",
            params![canvas_id],
            |row| row.get(0),
        )
        .optional()?;

    let cursor = cursor.ok_or_else(|| AppError::not_found(format!("画布不存在：{canvas_id}")))?;
    Ok(usize::try_from(cursor).unwrap_or(0))
}

pub fn set_history_cursor(store: &Store, canvas_id: &str, cursor: usize) -> Result<(), AppError> {
    let conn = store.connection();
    conn.execute(
        "UPDATE canvases SET history_cursor = ?2 WHERE id = ?1",
        params![canvas_id, cursor as i64],
    )?;
    Ok(())
}

/// 删除 `cursor` 之后的所有快照（新操作截断 redo 分支）。
pub fn truncate_snapshots_after(
    store: &Store,
    canvas_id: &str,
    cursor: usize,
) -> Result<(), AppError> {
    let conn = store.connection();
    conn.execute(
        "DELETE FROM snapshots WHERE canvas_id = ?1 AND cursor > ?2",
        params![canvas_id, cursor as i64],
    )?;
    Ok(())
}

/// 清空一个画布的全部快照（打开文件时整体覆盖历史）。
pub fn clear_snapshots(store: &Store, canvas_id: &str) -> Result<(), AppError> {
    let conn = store.connection();
    conn.execute(
        "DELETE FROM snapshots WHERE canvas_id = ?1",
        params![canvas_id],
    )?;
    Ok(())
}

pub fn insert_snapshot(
    store: &Store,
    canvas_id: &str,
    cursor: usize,
    snapshot: &Snapshot,
) -> Result<(), AppError> {
    let conn = store.connection();
    let json = serde_json::to_string(snapshot)?;
    conn.execute(
        "INSERT OR REPLACE INTO snapshots (canvas_id, cursor, snapshot_json)
         VALUES (?1, ?2, ?3)",
        params![canvas_id, cursor as i64, json],
    )?;
    Ok(())
}

pub fn snapshot_at(
    store: &Store,
    canvas_id: &str,
    cursor: usize,
) -> Result<Option<Snapshot>, AppError> {
    let conn = store.connection();
    let json: Option<String> = conn
        .query_row(
            "SELECT snapshot_json FROM snapshots WHERE canvas_id = ?1 AND cursor = ?2",
            params![canvas_id, cursor as i64],
            |row| row.get(0),
        )
        .optional()?;

    match json {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

pub fn snapshot_count(store: &Store, canvas_id: &str) -> Result<usize, AppError> {
    let conn = store.connection();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM snapshots WHERE canvas_id = ?1",
        params![canvas_id],
        |row| row.get(0),
    )?;
    Ok(usize::try_from(count).unwrap_or(0))
}

/// 全部快照（按游标升序），供 `.paint` 文件导出。
pub fn load_snapshots(store: &Store, canvas_id: &str) -> Result<Vec<Snapshot>, AppError> {
    let conn = store.connection();
    let mut statement = conn
        .prepare("SELECT snapshot_json FROM snapshots WHERE canvas_id = ?1 ORDER BY cursor ASC")?;

    let snapshots = statement
        .query_map(params![canvas_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|json| serde_json::from_str::<Snapshot>(&json))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::document::{Actor, LayerContent};

    fn new_store() -> Store {
        let store = Store::open_in_memory().expect("内存库");
        store.migrate().expect("迁移");
        store
    }

    #[test]
    fn save_and_load_round_trips_layers_and_order() {
        let store = new_store();
        let mut document = CanvasDocument::new("示例", 320, 200).expect("新建");
        document.layers.push(Layer::new(
            "第二层",
            LayerContent::Vector {
                svg: "<rect width=\"10\" height=\"10\" />".to_string(),
            },
            Actor::Agent,
            crate::paint::document::now_ms(),
        ));
        document.layers.push(Layer::new(
            "第三层",
            LayerContent::AiCode {
                code: "<circle r=\"4\" />".to_string(),
            },
            Actor::Agent,
            crate::paint::document::now_ms(),
        ));

        save_canvas(&store, &document).expect("保存");
        let loaded = load_canvas(&store, &document.id).expect("读取");

        assert_eq!(loaded, document);
        assert_eq!(loaded.layers.len(), 3);
        assert_eq!(loaded.layers[2].name, "第三层");
    }

    #[test]
    fn saving_twice_replaces_layers_without_duplicates() {
        let store = new_store();
        let mut document = CanvasDocument::new("示例", 100, 100).expect("新建");
        save_canvas(&store, &document).expect("首次保存");

        document.layers[0].name = "改名后的背景".to_string();
        document.bump_revision();
        save_canvas(&store, &document).expect("再次保存");

        let loaded = load_canvas(&store, &document.id).expect("读取");
        assert_eq!(loaded.layers.len(), 1);
        assert_eq!(loaded.layers[0].name, "改名后的背景");
        assert_eq!(loaded.revision, 1);
    }

    #[test]
    fn missing_canvas_is_not_found() {
        let store = new_store();
        let error = load_canvas(&store, "canvas-missing").expect_err("应报错");

        assert_eq!(error.kind, crate::error::ErrorKind::NotFound);
    }

    #[test]
    fn list_reports_layer_count_and_latest_first() {
        let store = new_store();
        let first = CanvasDocument::new("旧", 10, 10).expect("新建");
        save_canvas(&store, &first).expect("保存");
        let mut second = CanvasDocument::new("新", 20, 20).expect("新建");
        second.metadata.updated_at += 10;
        save_canvas(&store, &second).expect("保存");

        let list = list_canvases(&store).expect("列表");

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, second.id);
        assert_eq!(list[0].layer_count, 1);
        assert_eq!(list[0].width, 20);
    }

    #[test]
    fn snapshots_are_truncated_and_read_by_cursor() {
        let store = new_store();
        let document = CanvasDocument::new("示例", 50, 50).expect("新建");
        save_canvas(&store, &document).expect("保存");

        for cursor in 0..3 {
            let snapshot = Snapshot::capture("create_layer", Actor::User, &document);
            insert_snapshot(&store, &document.id, cursor, &snapshot).expect("写入快照");
        }
        assert_eq!(snapshot_count(&store, &document.id).expect("计数"), 3);

        truncate_snapshots_after(&store, &document.id, 1).expect("截断");
        assert_eq!(snapshot_count(&store, &document.id).expect("计数"), 2);
        assert!(snapshot_at(&store, &document.id, 2).expect("查询").is_none());
        assert!(snapshot_at(&store, &document.id, 1).expect("查询").is_some());
        assert_eq!(load_snapshots(&store, &document.id).expect("全量").len(), 2);
    }

    #[test]
    fn deleting_canvas_cascades_layers_and_snapshots() {
        let store = new_store();
        let document = CanvasDocument::new("示例", 50, 50).expect("新建");
        save_canvas(&store, &document).expect("保存");
        insert_snapshot(
            &store,
            &document.id,
            0,
            &Snapshot::capture("create_layer", Actor::User, &document),
        )
        .expect("写入快照");

        delete_canvas(&store, &document.id).expect("删除");

        let layers: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM layers", [], |row| row.get(0))
            .expect("统计图层");
        let snapshots: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("统计快照");
        assert_eq!((layers, snapshots), (0, 0));
    }

    #[test]
    fn history_cursor_defaults_to_zero_and_updates() {
        let store = new_store();
        let document = CanvasDocument::new("示例", 50, 50).expect("新建");
        save_canvas(&store, &document).expect("保存");

        assert_eq!(history_cursor(&store, &document.id).expect("读游标"), 0);
        set_history_cursor(&store, &document.id, 4).expect("写游标");
        assert_eq!(history_cursor(&store, &document.id).expect("读游标"), 4);

        save_canvas(&store, &document).expect("再次保存");
        assert_eq!(
            history_cursor(&store, &document.id).expect("读游标"),
            4,
            "保存文档不应重置历史游标"
        );
    }
}
