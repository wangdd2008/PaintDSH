//! 数字员工：内置三类角色 + 自定义，并同步为 DSH headless 的 persona patch。
//!
//! 版本事实（见 `docs` 里的 DSH 实测笔记）：DSH 0.2.1 的 headless profile **没有**
//! agent preset registry，`.agent-presets/` 目录也不存在。可行且稳定的等价物是
//! `--patch` 覆盖 `system-prompt.personaPrefix`（必要时再覆盖 `agent-default-model`）。
//! 因此本模块把员工同步成 `<PAINT_HOME>/harness/patches/employee-<id>.patch.yml`。

use std::path::PathBuf;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fsutil::{remove_path, write_if_changed};
use crate::paint::document::now_ms;
use crate::paths::PaintHome;
use crate::store::Store;

/// 员工表（幂等）。
pub const EMPLOYEE_MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS employees (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  role TEXT NOT NULL,
  persona TEXT NOT NULL,
  skills_json TEXT NOT NULL,
  knowledge_json TEXT NOT NULL,
  model TEXT,
  builtin INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
"#;

/// 同步出的 patch 文件名前缀。
pub const PRESET_PREFIX: &str = "employee-";

/// 一个数字员工。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub role: String,
    pub persona: String,
    pub skills: Vec<String>,
    pub knowledge: Vec<String>,
    pub model: Option<String>,
    pub builtin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建/更新员工的入参（`id` 为空表示新建）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeInput {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub role: String,
    pub persona: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub knowledge: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// 内置员工的定义（首次运行写库）。
pub fn builtin_employees() -> Vec<Employee> {
    let now = now_ms();
    vec![
        Employee {
            id: "graphic-designer".to_string(),
            name: "图形设计师".to_string(),
            role: "图形生成与版式建议".to_string(),
            persona: "你是 PAINT 的图形设计师，擅长把自然语言转成简洁的 SVG 矢量图形与版式。\
优先用几何图形表达，颜色克制，必要时给出布局建议。所有画布改动通过 paint-ops 提交。"
                .to_string(),
            skills: vec![
                "canvas-create-layer".to_string(),
                "canvas-draw-shape".to_string(),
                "canvas-select-region".to_string(),
            ],
            knowledge: vec!["brushes".to_string(), "palettes".to_string()],
            model: None,
            builtin: true,
            created_at: now,
            updated_at: now,
        },
        Employee {
            id: "filter-expert".to_string(),
            name: "滤镜专家".to_string(),
            role: "图像处理与调色".to_string(),
            persona: "你是 PAINT 的滤镜专家，擅长模糊、锐化、对比度与色彩平衡。\
给参数时要说明理由和取值范围，优先小步调整；只对 raster 图层提议滤镜。"
                .to_string(),
            skills: vec![
                "canvas-apply-filter".to_string(),
                "canvas-set-layer-content".to_string(),
            ],
            knowledge: vec!["palettes".to_string(), "design".to_string()],
            model: None,
            builtin: true,
            created_at: now,
            updated_at: now,
        },
        Employee {
            id: "canvas-assistant".to_string(),
            name: "画布助手".to_string(),
            role: "图层组织与结构管理".to_string(),
            persona: "你是 PAINT 的画布助手，擅长图层命名、分组排序与可见性管理。\
改动前先说明将要调整的图层，删除类操作必须提醒用户确认。"
                .to_string(),
            skills: vec![
                "canvas-create-layer".to_string(),
                "canvas-set-layer-content".to_string(),
                "canvas-get-context".to_string(),
            ],
            knowledge: vec!["design".to_string()],
            model: None,
            builtin: true,
            created_at: now,
            updated_at: now,
        },
    ]
}

/// 首次运行写入内置员工；已存在的 id 不覆盖（用户可能改过人设）。
pub fn seed_builtins(store: &Store) -> Result<usize, AppError> {
    let mut inserted = 0;
    for employee in builtin_employees() {
        if get(store, &employee.id)?.is_some() {
            continue;
        }
        insert(store, &employee)?;
        inserted += 1;
    }
    Ok(inserted)
}

pub fn list(store: &Store) -> Result<Vec<Employee>, AppError> {
    let conn = store.connection();
    let mut statement = conn.prepare(
        "SELECT id, name, role, persona, skills_json, knowledge_json, model, builtin,
                created_at, updated_at
         FROM employees ORDER BY builtin DESC, created_at ASC",
    )?;

    let rows = statement
        .query_map([], |row| {
            Ok(Employee {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                persona: row.get(3)?,
                skills: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                knowledge: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                model: row.get(6)?,
                builtin: row.get::<_, i64>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rows)
}

pub fn get(store: &Store, id: &str) -> Result<Option<Employee>, AppError> {
    let conn = store.connection();
    let row = conn
        .query_row(
            "SELECT id, name, role, persona, skills_json, knowledge_json, model, builtin,
                    created_at, updated_at
             FROM employees WHERE id = ?1",
            params![id],
            |row| {
                Ok(Employee {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    role: row.get(2)?,
                    persona: row.get(3)?,
                    skills: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    knowledge: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    model: row.get(6)?,
                    builtin: row.get::<_, i64>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .optional()?;

    Ok(row)
}

fn insert(store: &Store, employee: &Employee) -> Result<(), AppError> {
    store.connection().execute(
        "INSERT INTO employees (
            id, name, role, persona, skills_json, knowledge_json, model, builtin,
            created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            role = excluded.role,
            persona = excluded.persona,
            skills_json = excluded.skills_json,
            knowledge_json = excluded.knowledge_json,
            model = excluded.model,
            updated_at = excluded.updated_at",
        params![
            employee.id,
            employee.name,
            employee.role,
            employee.persona,
            serde_json::to_string(&employee.skills)?,
            serde_json::to_string(&employee.knowledge)?,
            employee.model,
            i64::from(employee.builtin),
            employee.created_at,
            employee.updated_at,
        ],
    )?;
    Ok(())
}

/// 新建或更新员工，返回落库后的实体。
pub fn save(store: &Store, input: EmployeeInput) -> Result<Employee, AppError> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::invalid_action("员工名称不能为空"));
    }
    let persona = input.persona.trim();
    if persona.is_empty() {
        return Err(AppError::invalid_action("角色设定不能为空"));
    }

    let existing = match input.id.as_deref() {
        Some(id) => get(store, id)?,
        None => None,
    };

    let now = now_ms();
    let existing_id = existing.as_ref().map(|employee| employee.id.clone());
    let builtin = existing.as_ref().map(|employee| employee.builtin).unwrap_or(false);
    let created_at = existing.as_ref().map(|employee| employee.created_at).unwrap_or(now);

    let employee = Employee {
        id: existing_id.unwrap_or_else(|| format!("employee-{}", uuid::Uuid::new_v4())),
        name: name.to_string(),
        role: input.role.trim().to_string(),
        persona: persona.to_string(),
        skills: input.skills,
        knowledge: input.knowledge,
        model: input.model,
        builtin,
        created_at,
        updated_at: now,
    };

    insert(store, &employee)?;
    Ok(employee)
}

/// 删除自定义员工；内置员工不允许删除。
pub fn delete(store: &Store, id: &str) -> Result<(), AppError> {
    let Some(employee) = get(store, id)? else {
        return Err(AppError::not_found(format!("员工不存在：{id}")));
    };
    if employee.builtin {
        return Err(AppError::invalid_action("内置数字员工不能删除"));
    }

    store
        .connection()
        .execute("DELETE FROM employees WHERE id = ?1", params![id])?;
    Ok(())
}

/// persona patch 的预设 id。
pub fn preset_id(employee_id: &str) -> String {
    format!("{PRESET_PREFIX}{employee_id}")
}

/// patch 目录。
pub fn preset_dir(home: &PaintHome) -> PathBuf {
    home.harness_home().join("patches")
}

/// 生成 persona patch 的 YAML。
pub fn preset_yaml(employee: &Employee) -> String {
    let mut yaml = String::new();
    yaml.push_str("# PAINT 数字员工 persona patch（自动生成，请勿手改）\n");
    yaml.push_str("- id: system-prompt\n");
    yaml.push_str("  config:\n");
    yaml.push_str("    personaPrefix: |-\n");
    for line in employee.persona.trim().lines() {
        yaml.push_str("      ");
        yaml.push_str(line);
        yaml.push('\n');
    }

    if let Some(model) = employee.model.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
        yaml.push_str("- id: agent-default-model\n");
        yaml.push_str("  config:\n");
        yaml.push_str(&format!("    model: {model}\n"));
    }

    yaml
}

/// 把单个员工写成 patch 文件，返回路径。
pub fn write_preset(home: &PaintHome, employee: &Employee) -> Result<PathBuf, AppError> {
    let path = preset_dir(home).join(format!("{}.patch.yml", preset_id(&employee.id)));
    write_if_changed(&path, preset_yaml(employee).as_bytes())?;
    Ok(path)
}

/// 同步全部员工到 patch 目录，清理已删除员工的 patch；返回写入的 preset id。
pub fn sync_employees(home: &PaintHome, store: &Store) -> Result<Vec<String>, AppError> {
    seed_builtins(store)?;

    let dir = preset_dir(home);
    std::fs::create_dir_all(&dir)
        .map_err(|error| AppError::io(format!("创建 preset 目录失败：{error}")))?;

    let employees = list(store)?;
    let mut written = Vec::new();
    let mut expected = Vec::new();

    for employee in &employees {
        let path = write_preset(home, employee)?;
        expected.push(path.file_name().map(|name| name.to_string_lossy().into_owned()));
        written.push(preset_id(&employee.id));
    }

    for entry in std::fs::read_dir(&dir)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(PRESET_PREFIX) || !name.ends_with(".patch.yml") {
            continue;
        }
        if expected.iter().any(|expected| expected.as_deref() == Some(name.as_str())) {
            continue;
        }
        remove_path(&entry.path())?;
    }

    Ok(written)
}

/// 员工对应的 persona patch 路径（存在才算）。
pub fn preset_path(home: &PaintHome, employee_id: &str) -> Option<PathBuf> {
    let path = preset_dir(home).join(format!("{}.patch.yml", preset_id(employee_id)));
    path.is_file().then_some(path)
}

/// 读取员工人设用于提示词（不存在返回 `None`）。
pub fn persona_of(store: &Store, employee_id: &str) -> Result<Option<String>, AppError> {
    Ok(get(store, employee_id)?.map(|employee| employee.persona))
}

/// 员工 preset 是否已同步到磁盘。
pub fn is_preset_synced(home: &PaintHome, employee_id: &str) -> bool {
    preset_path(home, employee_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_store() -> Store {
        let store = Store::open_in_memory().expect("内存库");
        store.migrate().expect("迁移");
        store
    }

    fn home() -> (PaintHome, PathBuf) {
        let root = std::env::temp_dir().join(format!("paint-employees-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("建目录");
        (PaintHome::new(&root), root)
    }

    #[test]
    fn seeding_builtins_is_idempotent() {
        let store = new_store();

        assert_eq!(seed_builtins(&store).expect("首次"), 3);
        assert_eq!(seed_builtins(&store).expect("再次"), 0);

        let employees = list(&store).expect("列表");
        assert_eq!(employees.len(), 3);
        assert!(employees.iter().all(|employee| employee.builtin));
        let names: Vec<_> = employees.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"图形设计师"));
        assert!(names.contains(&"滤镜专家"));
        assert!(names.contains(&"画布助手"));
    }

    #[test]
    fn custom_employee_can_be_created_updated_and_deleted() {
        let store = new_store();

        let created = save(
            &store,
            EmployeeInput {
                id: None,
                name: " 像素艺术家 ".to_string(),
                role: "像素画".to_string(),
                persona: "你是像素风专家".to_string(),
                skills: vec!["canvas-create-layer".to_string()],
                knowledge: vec!["palettes".to_string()],
                model: Some("deepseek-v4-pro".to_string()),
            },
        )
        .expect("创建");

        assert_eq!(created.name, "像素艺术家");
        assert!(!created.builtin);

        let updated = save(
            &store,
            EmployeeInput {
                id: Some(created.id.clone()),
                name: "像素大师".to_string(),
                role: created.role.clone(),
                persona: created.persona.clone(),
                skills: created.skills.clone(),
                knowledge: created.knowledge.clone(),
                model: None,
            },
        )
        .expect("更新");
        assert_eq!(updated.name, "像素大师");
        assert_eq!(updated.created_at, created.created_at);

        delete(&store, &created.id).expect("删除");
        assert!(get(&store, &created.id).expect("查询").is_none());
    }

    #[test]
    fn builtin_and_invalid_inputs_are_guarded() {
        let store = new_store();
        seed_builtins(&store).expect("seed");

        assert_eq!(
            delete(&store, "graphic-designer").expect_err("内置不能删").kind,
            crate::error::ErrorKind::InvalidAction
        );
        assert!(save(
            &store,
            EmployeeInput {
                id: None,
                name: "  ".to_string(),
                role: String::new(),
                persona: "人设".to_string(),
                skills: Vec::new(),
                knowledge: Vec::new(),
                model: None,
            }
        )
        .is_err());
        assert!(save(
            &store,
            EmployeeInput {
                id: None,
                name: "无设定".to_string(),
                role: String::new(),
                persona: "   ".to_string(),
                skills: Vec::new(),
                knowledge: Vec::new(),
                model: None,
            }
        )
        .is_err());
    }

    #[test]
    fn preset_yaml_contains_persona_and_optional_model() {
        let store = new_store();
        seed_builtins(&store).expect("seed");
        let mut employee = get(&store, "graphic-designer").expect("查询").expect("存在");

        let yaml = preset_yaml(&employee);
        assert!(yaml.contains("- id: system-prompt"));
        assert!(yaml.contains("personaPrefix: |-"));
        assert!(yaml.contains("图形设计师"));
        assert!(!yaml.contains("agent-default-model"));

        employee.model = Some("deepseek-v4-pro".to_string());
        let yaml = preset_yaml(&employee);
        assert!(yaml.contains("agent-default-model"));
        assert!(yaml.contains("model: deepseek-v4-pro"));
    }

    #[test]
    fn sync_writes_one_patch_per_employee_and_cleans_stale() {
        let (paint_home, root) = home();
        let store = new_store();

        let written = sync_employees(&paint_home, &store).expect("同步");
        assert_eq!(written.len(), 3);
        assert!(preset_dir(&paint_home)
            .join("employee-graphic-designer.patch.yml")
            .is_file());
        assert!(is_preset_synced(&paint_home, "graphic-designer"));

        let stale = preset_dir(&paint_home).join("employee-gone.patch.yml");
        std::fs::write(&stale, "stale").expect("写 stale");
        sync_employees(&paint_home, &store).expect("再同步");
        assert!(!stale.exists(), "应清理已删除员工的 patch");

        let second = sync_employees(&paint_home, &store).expect("再同步");
        assert_eq!(second.len(), 3);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn persona_is_readable_for_prompt_assembly() {
        let store = new_store();
        seed_builtins(&store).expect("seed");

        let persona = persona_of(&store, "filter-expert").expect("读取");
        assert!(persona.unwrap_or_default().contains("滤镜"));
        assert!(persona_of(&store, "nobody").expect("读取").is_none());
    }
}
