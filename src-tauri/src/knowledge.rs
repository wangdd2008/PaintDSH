//! 知识库：画笔模板、配色方案、形状模板、设计规范。
//!
//! 互通方式（依据 DSH 实测结论）：
//! - DSH headless 的上下文注入点是 `AGENTS.md` 链，PAINT 在 `$DSH_HOME/AGENTS.md`
//!   里维护一段"知识库索引"区块，AI 每次 run 都能看到；
//! - `storages/workspace.json` 是 DSH 内部 schema，不手写；注册通过
//!   `<knowledge>/.paint-workspace.json` 标记 + 该目录下的 `AGENTS.md` 实现，
//!   把知识库目录当作 DSH 工作区（cwd）打开时会自动生效。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fsutil::write_if_changed;
use crate::paths::PaintHome;

/// 知识库四个分区。
pub const KNOWLEDGE_KINDS: &[&str] = &["brushes", "palettes", "shapes", "design"];

/// 工作区标记文件。
pub const WORKSPACE_MARKER: &str = ".paint-workspace.json";

/// AGENTS.md 里知识库区块的起止标记。
pub const SECTION_BEGIN: &str = "<!-- PAINT:KNOWLEDGE:BEGIN -->";
pub const SECTION_END: &str = "<!-- PAINT:KNOWLEDGE:END -->";

/// 允许的扩展名。
const ALLOWED_EXTENSIONS: &[&str] = &["md", "json", "svg", "txt"];

/// 知识库条目。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntry {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// 工作区注册结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRegistration {
    pub workspace_path: String,
    pub registered: bool,
    pub entry_count: usize,
}

/// 知识库根目录。
pub fn knowledge_root(home: &PaintHome) -> PathBuf {
    home.knowledge_dir()
}

/// 创建目录骨架与示例规范（幂等）。
pub fn ensure_layout(home: &PaintHome) -> Result<(), AppError> {
    for kind in KNOWLEDGE_KINDS {
        let dir = knowledge_root(home).join(kind);
        std::fs::create_dir_all(&dir)
            .map_err(|error| AppError::io(format!("创建知识库目录失败：{error}")))?;
    }

    let guide = knowledge_root(home).join("design").join("design-guide.md");
    if !guide.exists() {
        write_if_changed(
            &guide,
            "# PAINT 设计规范\n\n\
## 颜色\n\n\
- 主色 `#e5484d`，辅助色 `#2f6feb`、`#30a46c`、`#f5a524`。\n\
- 文字用 `#1c2024`，浅背景用 `#f8f9fa`。\n\n\
## 版式\n\n\
- 画布留白不少于 32px，主要图形居中。\n\
- 同屏不超过三种强调色。\n\n\
## 图形\n\n\
- 统一 2px 描边，圆角 4/8/16 三档。\n"
                .as_bytes(),
        )?;
    }
    Ok(())
}

/// 校验分区名。
pub fn validate_kind(kind: &str) -> Result<(), AppError> {
    if KNOWLEDGE_KINDS.contains(&kind) {
        return Ok(());
    }
    Err(AppError::invalid_action(format!(
        "未知的知识库分区：{kind}（可用：{}）",
        KNOWLEDGE_KINDS.join(", ")
    )))
}

/// 列出某分区的条目。
pub fn list_entries(home: &PaintHome, kind: &str) -> Result<Vec<KnowledgeEntry>, AppError> {
    validate_kind(kind)?;
    let dir = knowledge_root(home).join(kind);
    let mut entries = Vec::new();

    if !dir.is_dir() {
        return Ok(entries);
    }

    for entry in std::fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let extension = path
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }
        let bytes = entry.metadata().map(|meta| meta.len()).unwrap_or(0);

        entries.push(KnowledgeEntry {
            kind: kind.to_string(),
            name: path
                .file_stem()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default(),
            path: path.to_string_lossy().into_owned(),
            bytes,
            content: None,
        });
    }

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

/// 读取单条内容（列表接口不返回正文，按需读）。
pub fn read_entry(home: &PaintHome, kind: &str, name: &str) -> Result<KnowledgeEntry, AppError> {
    validate_kind(kind)?;
    let path = entry_path(home, kind, name)?;
    let content = std::fs::read_to_string(&path)
        .map_err(|error| AppError::io(format!("读取知识条目失败：{error}")))?;
    let bytes = content.len() as u64;

    Ok(KnowledgeEntry {
        kind: kind.to_string(),
        name: name.to_string(),
        path: path.to_string_lossy().into_owned(),
        bytes,
        content: Some(content),
    })
}

/// 写入（或覆盖）一条知识。
pub fn save_entry(
    home: &PaintHome,
    kind: &str,
    name: &str,
    content: &str,
) -> Result<KnowledgeEntry, AppError> {
    ensure_layout(home)?;
    let path = entry_path(home, kind, name)?;
    let body = content.trim_end();
    if body.is_empty() {
        return Err(AppError::invalid_action("知识条目内容不能为空"));
    }
    write_if_changed(&path, body.as_bytes())?;
    sync_knowledge_agents(home)?;

    Ok(KnowledgeEntry {
        kind: kind.to_string(),
        name: name.trim().to_string(),
        path: path.to_string_lossy().into_owned(),
        bytes: body.len() as u64,
        content: Some(body.to_string()),
    })
}

/// 删除一条知识。
pub fn delete_entry(home: &PaintHome, kind: &str, name: &str) -> Result<(), AppError> {
    let path = entry_path(home, kind, name)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|error| AppError::io(format!("删除知识条目失败：{error}")))?;
    }
    sync_knowledge_agents(home)?;
    Ok(())
}

fn entry_path(home: &PaintHome, kind: &str, name: &str) -> Result<PathBuf, AppError> {
    validate_kind(kind)?;
    let sanitized = sanitize_name(name)?;
    let extension = Path::new(&sanitized)
        .extension()
        .map(|value| value.to_string_lossy().to_ascii_lowercase());

    let file_name = match extension.as_deref() {
        Some(ext) if ALLOWED_EXTENSIONS.contains(&ext) => sanitized,
        _ => format!("{sanitized}.md"),
    };

    Ok(knowledge_root(home).join(kind).join(file_name))
}

fn sanitize_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_action("知识条目名称不能为空"));
    }
    let cleaned: String = trimmed
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    if cleaned.contains("..") {
        return Err(AppError::invalid_action("知识条目名称不能包含 .."));
    }
    Ok(cleaned)
}

/// 生成 AGENTS.md 里的知识库索引区块。
pub fn agents_section(home: &PaintHome) -> Result<String, AppError> {
    let mut lines = vec![
        SECTION_BEGIN.to_string(),
        "## PAINT 知识库".to_string(),
        String::new(),
        format!(
            "知识库根目录：`{}`。需要参考时按下表读取对应文件。",
            knowledge_root(home).display()
        ),
        String::new(),
    ];

    for kind in KNOWLEDGE_KINDS {
        let entries = list_entries(home, kind)?;
        if entries.is_empty() {
            continue;
        }
        lines.push(format!("### {kind}"));
        lines.push(String::new());
        for entry in entries {
            lines.push(format!("- `{}`（{} 字节）", entry.path, entry.bytes));
        }
        lines.push(String::new());
    }

    lines.push(SECTION_END.to_string());
    Ok(lines.join("\n"))
}

/// 把知识库索引写进 `$DSH_HOME/AGENTS.md`（保留协议正文，只替换索引区块）。
pub fn sync_knowledge_agents(home: &PaintHome) -> Result<(), AppError> {
    ensure_layout(home)?;
    let agents_path = home.harness_home().join("AGENTS.md");
    let existing = std::fs::read_to_string(&agents_path).unwrap_or_default();
    let section = agents_section(home)?;

    let merged = match (
        existing.find(SECTION_BEGIN),
        existing.find(SECTION_END),
    ) {
        (Some(begin), Some(end)) if end > begin => {
            let after = end + SECTION_END.len();
            format!("{}{}{}", &existing[..begin], section, &existing[after..])
        }
        _ => {
            if existing.trim().is_empty() {
                section
            } else {
                format!("{}\n\n{}\n", existing.trim_end(), section)
            }
        }
    };

    write_if_changed(&agents_path, merged.as_bytes())?;
    Ok(())
}

/// 把知识库注册为 DSH 工作区：写标记文件 + 目录内 AGENTS.md + 全局索引。
pub fn register_workspace(home: &PaintHome) -> Result<WorkspaceRegistration, AppError> {
    ensure_layout(home)?;

    let root = knowledge_root(home);
    let marker = serde_json::json!({
        "name": "PAINT Knowledge",
        "kind": "paint-knowledge",
        "version": 1,
        "root": root.to_string_lossy(),
        "paintHome": home.root().to_string_lossy(),
    });
    write_if_changed(
        &root.join(WORKSPACE_MARKER),
        serde_json::to_string_pretty(&marker)?.as_bytes(),
    )?;

    let agents = format!(
        "# PAINT 知识库工作区\n\n本目录是 PAINT 的知识库，可直接作为 DSH 工作区打开。\n\n{}\n",
        agents_section(home)?.replace(SECTION_BEGIN, "").replace(SECTION_END, "")
    );
    write_if_changed(&root.join("AGENTS.md"), agents.trim().as_bytes())?;

    sync_knowledge_agents(home)?;

    let mut entry_count = 0;
    for kind in KNOWLEDGE_KINDS {
        entry_count += list_entries(home, kind)?.len();
    }

    Ok(WorkspaceRegistration {
        workspace_path: root.to_string_lossy().into_owned(),
        registered: true,
        entry_count,
    })
}

/// 知识库里的调色板，供前端/提示词直接引用。
pub fn palette_names(home: &PaintHome) -> Result<Vec<String>, AppError> {
    Ok(list_entries(home, "palettes")?
        .into_iter()
        .map(|entry| entry.name)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> (PaintHome, PathBuf) {
        let root = std::env::temp_dir().join(format!("paint-knowledge-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("建目录");
        (PaintHome::new(&root), root)
    }

    #[test]
    fn layout_creates_four_partitions_and_guide() {
        let (paint_home, root) = home();

        ensure_layout(&paint_home).expect("建骨架");

        for kind in KNOWLEDGE_KINDS {
            assert!(knowledge_root(&paint_home).join(kind).is_dir(), "缺少 {kind}");
        }
        assert!(knowledge_root(&paint_home)
            .join("design")
            .join("design-guide.md")
            .is_file());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn save_and_list_entries() {
        let (paint_home, root) = home();

        let saved = save_entry(&paint_home, "palettes", "落日", "# 落日\n- #ff7a45\n").expect("保存");
        assert_eq!(saved.name, "落日");
        assert!(saved.path.ends_with("落日.md"));

        let entries = list_entries(&paint_home, "palettes").expect("列表");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "落日");

        let read = read_entry(&paint_home, "palettes", "落日").expect("读取");
        assert!(read.content.unwrap_or_default().contains("#ff7a45"));

        delete_entry(&paint_home, "palettes", "落日").expect("删除");
        assert!(list_entries(&paint_home, "palettes").expect("列表").is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn unknown_kind_and_bad_names_are_rejected() {
        let (paint_home, root) = home();

        assert!(save_entry(&paint_home, "music", "x", "y").is_err());
        assert!(save_entry(&paint_home, "brushes", "  ", "y").is_err());
        assert!(save_entry(&paint_home, "brushes", "../escape", "y").is_err());
        assert!(save_entry(&paint_home, "brushes", "empty", "   ").is_err());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn workspace_registration_writes_marker_and_agents() {
        let (paint_home, root) = home();
        save_entry(&paint_home, "brushes", "铅笔", "2px 硬边").expect("保存");

        let registration = register_workspace(&paint_home).expect("注册");

        assert!(registration.registered);
        assert!(registration.entry_count >= 2, "含默认设计规范");
        assert!(knowledge_root(&paint_home).join(WORKSPACE_MARKER).is_file());
        let agents = std::fs::read_to_string(knowledge_root(&paint_home).join("AGENTS.md"))
            .expect("工作区 AGENTS");
        assert!(agents.contains("铅笔"));

        let harness_agents =
            std::fs::read_to_string(paint_home.harness_home().join("AGENTS.md")).expect("全局 AGENTS");
        assert!(harness_agents.contains(SECTION_BEGIN));
        assert!(harness_agents.contains("铅笔"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn knowledge_section_is_replaced_not_duplicated() {
        let (paint_home, root) = home();
        save_entry(&paint_home, "palettes", "冷色", "- #2f6feb").expect("保存");
        sync_knowledge_agents(&paint_home).expect("同步");

        save_entry(&paint_home, "palettes", "暖色", "- #e5484d").expect("保存");
        sync_knowledge_agents(&paint_home).expect("再同步");

        let agents =
            std::fs::read_to_string(paint_home.harness_home().join("AGENTS.md")).expect("读取");
        assert_eq!(agents.matches(SECTION_BEGIN).count(), 1);
        assert!(agents.contains("冷色"));
        assert!(agents.contains("暖色"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_protocol_text_is_preserved() {
        let (paint_home, root) = home();
        let agents_path = paint_home.harness_home().join("AGENTS.md");
        std::fs::create_dir_all(paint_home.harness_home()).expect("建目录");
        std::fs::write(&agents_path, "# PAINT 画布协议\n\npaint-ops 说明\n").expect("写");

        sync_knowledge_agents(&paint_home).expect("同步");

        let agents = std::fs::read_to_string(&agents_path).expect("读取");
        assert!(agents.starts_with("# PAINT 画布协议"));
        assert!(agents.contains("paint-ops 说明"));
        assert!(agents.contains(SECTION_BEGIN));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn palette_names_are_exposed() {
        let (paint_home, root) = home();
        save_entry(&paint_home, "palettes", "落日", "a").expect("保存");
        save_entry(&paint_home, "palettes", "海洋", "b").expect("保存");

        assert_eq!(
            palette_names(&paint_home).expect("调色板"),
            vec!["海洋".to_string(), "落日".to_string()]
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
