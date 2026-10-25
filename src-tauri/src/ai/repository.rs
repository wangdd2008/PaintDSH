//! AI 网关的数据层：`ai_sessions` / `ai_messages` / `ai_calls`。
//!
//! 计量口径：`ai_calls` 明细逐条记录；按日汇总用 SQL 聚合，不额外建表
//! （engineering-contract 第 7 节）。

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::store::Store;

/// `ai_*` 三张表的建表语句（幂等）。
pub const AI_MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS ai_sessions (
  id TEXT PRIMARY KEY,
  canvas_id TEXT,
  employee_id TEXT,
  engine TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_messages_session ON ai_messages(session_id, id);

CREATE TABLE IF NOT EXISTS ai_calls (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT,
  canvas_id TEXT,
  module TEXT NOT NULL,
  employee TEXT,
  task TEXT NOT NULL,
  engine TEXT NOT NULL,
  model TEXT NOT NULL,
  tokens_input INTEGER NOT NULL DEFAULT 0,
  tokens_output INTEGER NOT NULL DEFAULT 0,
  latency_ms INTEGER NOT NULL DEFAULT 0,
  success INTEGER NOT NULL,
  error TEXT,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_calls_created ON ai_calls(created_at DESC);
"#;

/// 一次 AI 调用的审计记录。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiCall {
    pub id: i64,
    pub session_id: Option<String>,
    pub canvas_id: Option<String>,
    pub module: String,
    pub employee: Option<String>,
    pub task: String,
    pub engine: String,
    pub model: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub latency_ms: i64,
    pub success: bool,
    pub error: Option<String>,
    pub created_at: i64,
}

/// 待写入的调用记录（`id`/`created_at` 由仓储填）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAiCall {
    pub session_id: Option<String>,
    pub canvas_id: Option<String>,
    pub module: String,
    pub employee: Option<String>,
    pub task: String,
    pub engine: String,
    pub model: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub latency_ms: i64,
    pub success: bool,
    pub error: Option<String>,
}

/// 按日汇总。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageRow {
    pub date: String,
    pub calls: i64,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub failures: i64,
}

pub fn insert_session(
    store: &Store,
    session_id: &str,
    canvas_id: Option<&str>,
    employee_id: Option<&str>,
    engine: &str,
    created_at: i64,
) -> Result<(), AppError> {
    store.connection().execute(
        "INSERT OR REPLACE INTO ai_sessions (id, canvas_id, employee_id, engine, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![session_id, canvas_id, employee_id, engine, created_at],
    )?;
    Ok(())
}

pub fn insert_message(
    store: &Store,
    session_id: &str,
    role: &str,
    content: &str,
    created_at: i64,
) -> Result<(), AppError> {
    store.connection().execute(
        "INSERT INTO ai_messages (session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![session_id, role, content, created_at],
    )?;
    Ok(())
}

pub fn insert_call(store: &Store, call: &NewAiCall, created_at: i64) -> Result<i64, AppError> {
    let conn = store.connection();
    conn.execute(
        "INSERT INTO ai_calls (
            session_id, canvas_id, module, employee, task, engine, model,
            tokens_input, tokens_output, latency_ms, success, error, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            call.session_id,
            call.canvas_id,
            call.module,
            call.employee,
            call.task,
            call.engine,
            call.model,
            call.tokens_input,
            call.tokens_output,
            call.latency_ms,
            i64::from(call.success),
            call.error,
            created_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_calls(store: &Store, limit: u32) -> Result<Vec<AiCall>, AppError> {
    let conn = store.connection();
    let mut statement = conn.prepare(
        "SELECT id, session_id, canvas_id, module, employee, task, engine, model,
                tokens_input, tokens_output, latency_ms, success, error, created_at
         FROM ai_calls ORDER BY id DESC LIMIT ?1",
    )?;

    let rows = statement
        .query_map(params![i64::from(limit.max(1))], |row| {
            Ok(AiCall {
                id: row.get(0)?,
                session_id: row.get(1)?,
                canvas_id: row.get(2)?,
                module: row.get(3)?,
                employee: row.get(4)?,
                task: row.get(5)?,
                engine: row.get(6)?,
                model: row.get(7)?,
                tokens_input: row.get(8)?,
                tokens_output: row.get(9)?,
                latency_ms: row.get(10)?,
                success: row.get::<_, i64>(11)? != 0,
                error: row.get(12)?,
                created_at: row.get(13)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn usage_summary(store: &Store, days: u32) -> Result<Vec<AiUsageRow>, AppError> {
    let conn = store.connection();
    let since = crate::paint::document::now_ms() - i64::from(days.max(1)) * 86_400_000;

    let mut statement = conn.prepare(
        "SELECT date(created_at / 1000, 'unixepoch', 'localtime') AS day,
                COUNT(*),
                COALESCE(SUM(tokens_input), 0),
                COALESCE(SUM(tokens_output), 0),
                COALESCE(SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END), 0)
         FROM ai_calls WHERE created_at >= ?1
         GROUP BY day ORDER BY day DESC",
    )?;

    let rows = statement
        .query_map(params![since], |row| {
            Ok(AiUsageRow {
                date: row.get(0)?,
                calls: row.get(1)?,
                tokens_input: row.get(2)?,
                tokens_output: row.get(3)?,
                failures: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// 某次调用的总 token（审计展示用）。
pub fn call_tokens(store: &Store, id: i64) -> Result<Option<(i64, i64)>, AppError> {
    let conn = store.connection();
    let row = conn
        .query_row(
            "SELECT tokens_input, tokens_output FROM ai_calls WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::document::now_ms;

    fn new_store() -> Store {
        let store = Store::open_in_memory().expect("内存库");
        store.migrate().expect("迁移");
        store
    }

    fn call(success: bool, tokens: (i64, i64)) -> NewAiCall {
        NewAiCall {
            session_id: Some("session-1".to_string()),
            canvas_id: Some("canvas-1".to_string()),
            module: "paint".to_string(),
            employee: Some("图形设计师".to_string()),
            task: "画一个红色的圆".to_string(),
            engine: "harness".to_string(),
            model: "deepseek-flash".to_string(),
            tokens_input: tokens.0,
            tokens_output: tokens.1,
            latency_ms: 1234,
            success,
            error: None,
        }
    }

    #[test]
    fn calls_are_inserted_and_listed_newest_first() {
        let store = new_store();
        insert_call(&store, &call(true, (10, 5)), now_ms()).expect("写入");
        insert_call(&store, &call(false, (7, 0)), now_ms() + 1).expect("写入");

        let list = list_calls(&store, 10).expect("列表");

        assert_eq!(list.len(), 2);
        assert!(list[0].created_at > list[1].created_at);
        assert!(!list[0].success);
        assert_eq!(list[1].tokens_input, 10);
        assert_eq!(list[1].employee.as_deref(), Some("图形设计师"));
    }

    #[test]
    fn usage_summary_aggregates_tokens_and_failures() {
        let store = new_store();
        insert_call(&store, &call(true, (10, 5)), now_ms()).expect("写入");
        insert_call(&store, &call(true, (20, 7)), now_ms()).expect("写入");
        insert_call(&store, &call(false, (1, 0)), now_ms()).expect("写入");

        let summary = usage_summary(&store, 7).expect("汇总");

        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].calls, 3);
        assert_eq!(summary[0].tokens_input, 31);
        assert_eq!(summary[0].tokens_output, 12);
        assert_eq!(summary[0].failures, 1);
    }

    #[test]
    fn sessions_and_messages_are_persisted() {
        let store = new_store();
        insert_session(&store, "session-9", Some("canvas-1"), None, "direct", now_ms())
            .expect("会话");
        insert_message(&store, "session-9", "user", "你好", now_ms()).expect("消息");
        insert_message(&store, "session-9", "assistant", "你好呀", now_ms()).expect("消息");

        let count: i64 = store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM ai_messages WHERE session_id = ?1",
                params!["session-9"],
                |row| row.get(0),
            )
            .expect("统计");
        assert_eq!(count, 2);
    }

    #[test]
    fn call_tokens_returns_none_for_unknown_id() {
        let store = new_store();
        assert!(call_tokens(&store, 42).expect("查询").is_none());
    }
}
