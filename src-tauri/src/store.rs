//! SQLite 数据层：连接、PRAGMA、顺序迁移。
//!
//! 表结构分散在各业务模块里（`paint::repository` / `ai::repository` / `employees`），
//! 这里只按顺序把它们拼进 [`MIGRATIONS`]；已发布的条目不得修改（engineering-contract 第 7 节）。

use std::path::Path;

use rusqlite::Connection;

use crate::error::AppError;

/// 顺序执行、必须幂等的 schema 迁移。**只允许往尾部追加。**
pub const MIGRATIONS: &[&str] = &[
    crate::paint::repository::CANVAS_MIGRATIONS,
    crate::ai::repository::AI_MIGRATIONS,
    crate::employees::EMPLOYEE_MIGRATIONS,
];

/// PAINT 的 SQLite 连接封装。
pub struct Store {
    conn: Connection,
}

impl Store {
    /// 打开（或创建）数据库文件。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    /// 内存数据库（单测用）。
    pub fn open_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()?;
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> Result<Self, AppError> {
        // journal_mode 会返回一行结果，用 query_row 读取，其余用 execute_batch。
        let _journal_mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
        // busy_timeout 是 WAL + 多连接（后台 AI 线程另开连接）的最后一道保险。
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;",
        )?;

        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// 执行尚未应用的迁移，返回本次执行条数。
    pub fn migrate(&self) -> Result<usize, AppError> {
        let mut applied = 0;
        for statement in MIGRATIONS {
            self.conn.execute_batch(statement)?;
            applied += 1;
        }
        Ok(applied)
    }

    /// 底层连接，供各业务模块实现查询/事务。
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_store_enables_foreign_keys() {
        let store = Store::open_in_memory().expect("打开内存库");
        let foreign_keys: i64 = store
            .connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("读取 foreign_keys");

        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn file_store_uses_wal_journal_mode() {
        let dir = std::env::temp_dir().join(format!("paint-store-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("创建临时目录");
        let db_path = dir.join("paint.db");

        let store = Store::open(&db_path).expect("打开文件库");
        let journal_mode: String = store
            .connection()
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("读取 journal_mode");

        assert_eq!(journal_mode.to_lowercase(), "wal");
        assert!(db_path.exists());

        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrate_is_idempotent() {
        let store = Store::open_in_memory().expect("打开内存库");

        let first = store.migrate().expect("首次迁移");
        let second = store.migrate().expect("再次迁移");

        assert_eq!(first, MIGRATIONS.len());
        assert_eq!(second, MIGRATIONS.len());
    }
}
