//! DSH Skill：把画布操作封装成 `SKILL.md`，同步到 PAINT 自己的 DSH_HOME。
//!
//! 技能只是**提示词**，没有执行能力；真正的写入口仍是 Command Dispatcher
//! （见 `ai::ops`）。格式与扫描根见 DSH 的 `dsh-skill-filesystem`：
//! `<root>/<name>/SKILL.md`，frontmatter 必须有 `name`/`description`。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::fsutil::{remove_path, write_if_changed};
use crate::paths::PaintHome;

/// 同步到 DSH 的技能目录名前缀（清理时只动带此前缀的条目）。
pub const MANAGED_PREFIX: &str = "canvas-";

/// 同步清单文件名（放在 DSH 技能根下）。
pub const MANIFEST_FILE: &str = ".paint-manifest.json";

/// 禁用后的目录后缀。
pub const DISABLED_SUFFIX: &str = ".disabled";

/// 全局协议文件（写到 `$DSH_HOME/AGENTS.md`，DSH 每次 run 自动注入）。
pub const PROTOCOL_FILE: &str = "AGENTS.md";

/// 一个内置画布技能。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDefinition {
    pub id: &'static str,
    pub description: &'static str,
    pub when_to_use: &'static str,
    pub body: &'static str,
}

/// 内置技能清单（对应 `docs/architecture-design.md` 4.4 的七个画布操作）。
pub const CANVAS_SKILLS: &[SkillDefinition] = &[
    SkillDefinition {
        id: "canvas-get-context",
        description: "读取当前画布状态（尺寸、图层、选中区域）",
        when_to_use: "需要在动手前了解画布现状时",
        body: "## 用法\n\n画布状态已经随用户消息一起给出（JSON）。\n\
如果需要更新后的状态，在 paint-ops 里先放一条 `select_layer` 把目标图层设为当前图层，\
不要臆造图层 id。\n",
    },
    SkillDefinition {
        id: "canvas-create-layer",
        description: "在画布上新建图层，可带初始 SVG/AI 代码内容",
        when_to_use: "用户要求新增图形、贴图或由你生成的矢量内容时",
        body: "## 参数\n\n- `name`: string — 图层名（必填，简短中文）\n\
- `type`: `vector` | `aiGenerated` | `raster`\n\
- `content`: `{ \"type\": \"aiCode\", \"code\": \"<svg…/>\" }` 或 `{ \"type\": \"vector\", \"svg\": \"…\" }`\n\
- `prompt`: string — 你据以生成这条内容的用户原话\n\n\
## 示例\n\n```paint-ops\n[{\"action\":\"create_layer\",\"input\":{\"name\":\"红色圆\",\"type\":\"aiGenerated\",\
\"content\":{\"type\":\"aiCode\",\"code\":\"<circle cx=\\\"160\\\" cy=\\\"120\\\" r=\\\"60\\\" fill=\\\"#e5484d\\\"/>\"},\
\"prompt\":\"画一个红色的圆\"}}]\n```\n",
    },
    SkillDefinition {
        id: "canvas-set-layer-content",
        description: "替换某个图层的内容（画笔提交、AI 重绘都走它）",
        when_to_use: "在已有图层上重画或修正内容时",
        body: "## 参数\n\n- `layerId`: string — 必须来自画布状态里的真实 id\n\
- `content`: 与 `canvas-create-layer` 相同\n\n\
修改已有图层前，先确认该图层不是你生成的——用户手绘图层只做增量修改。\n",
    },
    SkillDefinition {
        id: "canvas-apply-filter",
        description: "给位图图层追加滤镜（模糊、锐化、调色）",
        when_to_use: "用户要求柔化、锐化、调色或做视觉风格化时",
        body: "## 参数\n\n- `layerId`: string — 必须是 raster 图层\n\
- `filter`: `{\"kind\":\"blur\",\"radius\":8}` | `{\"kind\":\"sharpen\",\"amount\":1.5}` | \
`{\"kind\":\"adjustColor\",\"brightness\":0,\"contrast\":0.1,\"saturation\":0.2,\"hue\":0}`\n\n\
矢量图层不能直接加滤镜；需要时先新建一个 raster 图层并给出栅格化后的内容。\n",
    },
    SkillDefinition {
        id: "canvas-draw-shape",
        description: "按描述生成矢量图形（圆、矩形、线条、路径）",
        when_to_use: "用户要求画几何图形或简单插图时",
        body: "## 约定\n\n- 画布坐标原点在左上角，单位像素；图形用 SVG 元素表达。\n\
- 颜色用十六进制；需要半透明时用 `fill-opacity`。\n\
- 一次最多 20 条操作，复杂画面分多轮完成。\n\n\
输出走 `canvas-create-layer`，并把原始 SVG 片段写进 `content.code`。\n",
    },
    SkillDefinition {
        id: "canvas-select-region",
        description: "设置或清空画布的选中区域",
        when_to_use: "用户指明只处理画面某一块区域时",
        body: "## 参数\n\n- `region`: `{\"x\":0,\"y\":0,\"width\":320,\"height\":240}` 或 `null`\n\n\
选中区域只影响上下文，不改变像素；需要裁剪时另建图层。\n",
    },
    SkillDefinition {
        id: "canvas-export-image",
        description: "提示用户导出为 PNG/JPG/SVG",
        when_to_use: "用户说「导出」「保存成图片」时",
        body: "## 说明\n\n导出由用户在前端菜单完成（需要原生文件对话框）。\n\
你只回复「可以点文件 → 导出」这类指引，**不要**在 paint-ops 里伪造导出操作。\n",
    },
];

/// 技能 markdown（frontmatter + 正文）。
pub fn skill_markdown(definition: &SkillDefinition) -> String {
    format!(
        "---\nname: {name}\ndescription: {description}\nwhenToUse: {when}\n---\n\n# {name}\n\n{body}",
        name = definition.id,
        description = definition.description,
        when = definition.when_to_use,
        body = definition.body,
    )
}

/// 全局协议（写到 `$DSH_HOME/AGENTS.md`）。
pub fn protocol_markdown() -> String {
    "<!-- 由 PAINT 生成，请勿手改；每次同步都会覆盖 -->\n\
# PAINT 画布协议\n\n\
你是 PAINT 画图软件里的画布助手。你**不能**直接执行 shell 命令或写画布文件；\
所有画布改动通过最终答复里的 ```paint-ops 代码块交给 PAINT 执行。\n\n\
## paint-ops 格式\n\n\
最终答复里放一段（可多段）如下代码块，内容是操作数组：\n\n\
```paint-ops\n\
[{\"action\":\"create_layer\",\"input\":{\"name\":\"红圆\",\"type\":\"aiGenerated\",\
\"content\":{\"type\":\"aiCode\",\"code\":\"<circle cx=\\\"100\\\" cy=\\\"100\\\" r=\\\"40\\\" fill=\\\"#e5484d\\\"/>\"}}}]\n\
```\n\n\
## 允许的 action\n\n\
`create_layer`、`set_layer_content`、`set_layer_property`、`set_layer_visibility`、\
`set_layer_transform`、`reorder_layer`、`select_layer`、`select_region`、\
`apply_filter`、`clear_filters`、`delete_layer`（需用户确认）。\n\n\
## 硬性约束\n\n\
- 一次答复最多 20 条操作；没有任何改动时不要输出代码块。\n\
- `layerId` 只能引用上下文里出现过的 id；不确定时先 `create_layer`。\n\
- 只输出画布操作，不要解释系统命令、不要请求提高权限。\n\
- 图形用 SVG 元素表达，画布原点在左上角，单位像素。\n"
        .to_string()
}

/// 技能来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    /// PAINT 自己的技能（`~/.paint/skills`）。
    Paint,
    /// DSH 生态技能（`$DSH_HOME/skills` 里非 PAINT 管理的条目）。
    Dsh,
}

impl SkillSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            SkillSource::Paint => "paint",
            SkillSource::Dsh => "dsh",
        }
    }
}

/// 技能列表项。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: SkillSource,
    pub path: String,
    pub enabled: bool,
}

/// 同步结果。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub written: Vec<String>,
    pub unchanged: Vec<String>,
    pub removed: Vec<String>,
}

/// PAINT 技能主目录。
pub fn paint_skills_dir(home: &PaintHome) -> PathBuf {
    home.skills_dir()
}

/// DSH 技能根（PAINT 自己 DSH_HOME 下的 user-dsh 根）。
pub fn dsh_skills_dir(home: &PaintHome) -> PathBuf {
    home.harness_home().join("skills")
}

fn manifest_path(home: &PaintHome) -> PathBuf {
    dsh_skills_dir(home).join(MANIFEST_FILE)
}

fn read_manifest(home: &PaintHome) -> Vec<String> {
    std::fs::read_to_string(manifest_path(home))
        .ok()
        .and_then(|text| serde_json::from_str::<Vec<String>>(&text).ok())
        .unwrap_or_default()
}

fn write_manifest(home: &PaintHome, entries: &[String]) -> Result<(), AppError> {
    let json = serde_json::to_string_pretty(entries)?;
    write_if_changed(&manifest_path(home), json.as_bytes())?;
    Ok(())
}

/// 首次运行把内置技能落盘到 `~/.paint/skills`（已存在的不覆盖，用户可自行编辑）。
pub fn seed_builtin_skills(home: &PaintHome) -> Result<Vec<String>, AppError> {
    let mut created = Vec::new();
    for definition in CANVAS_SKILLS {
        let dir = paint_skills_dir(home).join(definition.id);
        let disabled_dir = paint_skills_dir(home).join(format!("{}{DISABLED_SUFFIX}", definition.id));
        // 被用户禁用（改名成 *.disabled）的技能不要"复活"。
        if dir.join(SKILL_FILE).exists() || disabled_dir.exists() {
            continue;
        }
        write_if_changed(&dir.join(SKILL_FILE), skill_markdown(definition).as_bytes())?;
        created.push(definition.id.to_string());
    }
    Ok(created)
}

/// 同步技能与协议到 DSH 目录；返回写入/未变/清理明细。
pub fn sync_skills(home: &PaintHome) -> Result<SyncReport, AppError> {
    seed_builtin_skills(home)?;

    let target_root = dsh_skills_dir(home);
    std::fs::create_dir_all(&target_root).map_err(|error| {
        AppError::io(format!("创建技能目录 {} 失败：{error}", target_root.display()))
    })?;

    let mut report = SyncReport::default();
    let mut managed: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(paint_skills_dir(home)).map_err(|error| {
        AppError::io(format!("读取技能目录失败：{error}"))
    })?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let (id, enabled) = split_enabled(&name);
        if !id.starts_with(MANAGED_PREFIX) {
            continue;
        }
        let source_dir = entry.path();
        let source_file = if source_dir.is_dir() {
            source_dir.join("SKILL.md")
        } else {
            source_dir.clone()
        };
        if !source_file.is_file() {
            continue;
        }

        if !enabled {
            // 禁用：从 DSH 侧移除（下次启用会重建）。
            continue;
        }

        let content = std::fs::read(&source_file)?;
        let target_dir = target_root.join(&id);
        let target_file = target_dir.join("SKILL.md");
        let changed = write_if_changed(&target_file, &content)?;
        managed.push(id.clone());
        if changed {
            report.written.push(id);
        } else {
            report.unchanged.push(id);
        }
    }

    // 清理上次同步过、这次已不存在的条目（只动 manifest 里的 canvas-*）。
    for stale in read_manifest(home) {
        if managed.contains(&stale) {
            continue;
        }
        remove_path(&target_root.join(&stale))?;
        report.removed.push(stale);
    }

    write_manifest(home, &managed)?;
    write_if_changed(
        &home.harness_home().join(PROTOCOL_FILE),
        protocol_markdown().as_bytes(),
    )?;

    Ok(report)
}

fn split_enabled(dir_name: &str) -> (String, bool) {
    match dir_name.strip_suffix(DISABLED_SUFFIX) {
        Some(id) => (id.to_string(), false),
        None => (dir_name.to_string(), true),
    }
}

/// 启用/禁用 PAINT 技能（改名而非删除，避免 DSH 文件监听抖动）。
pub fn set_skill_enabled(home: &PaintHome, id: &str, enabled: bool) -> Result<(), AppError> {
    if !id.starts_with(MANAGED_PREFIX) {
        return Err(AppError::invalid_action(format!(
            "只能管理 {MANAGED_PREFIX}* 技能，收到 {id}"
        )));
    }

    let root = paint_skills_dir(home);
    let enabled_dir = root.join(id);
    let disabled_dir = root.join(format!("{id}{DISABLED_SUFFIX}"));

    if enabled {
        if disabled_dir.exists() {
            std::fs::rename(&disabled_dir, &enabled_dir)
                .map_err(|error| AppError::io(format!("启用技能失败：{error}")))?;
        }
    } else if enabled_dir.exists() {
        std::fs::rename(&enabled_dir, &disabled_dir)
            .map_err(|error| AppError::io(format!("禁用技能失败：{error}")))?;
    }

    sync_skills(home)?;
    Ok(())
}

/// 列出 PAINT 技能与 DSH 生态技能。
pub fn list_skills(home: &PaintHome) -> Result<Vec<SkillInfo>, AppError> {
    let managed = read_manifest(home);
    let mut skills = Vec::new();

    collect_skills(&paint_skills_dir(home), SkillSource::Paint, &mut skills)?;
    collect_skills(&dsh_skills_dir(home), SkillSource::Dsh, &mut skills)?;

    // DSH 侧里由 PAINT 同步过去的条目不算「DSH 生态技能」。
    skills.retain(|skill| {
        !(skill.source == SkillSource::Dsh
            && managed.contains(&skill.id.replace(DISABLED_SUFFIX, "")))
    });

    skills.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(skills)
}

fn collect_skills(
    root: &Path,
    source: SkillSource,
    out: &mut Vec<SkillInfo>,
) -> Result<(), AppError> {
    if !root.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(root)?.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == MANIFEST_FILE || name.starts_with('.') {
            continue;
        }
        let (id, enabled) = split_enabled(&name);
        let path = entry.path();
        let file = if path.is_dir() {
            path.join("SKILL.md")
        } else {
            path.clone()
        };
        if !file.is_file() {
            continue;
        }

        let text = std::fs::read_to_string(&file).unwrap_or_default();
        let (skill_name, description) = parse_frontmatter(&text);
        out.push(SkillInfo {
            id: id.clone(),
            name: skill_name.unwrap_or_else(|| id.clone()),
            description: description.unwrap_or_default(),
            source,
            path: file.to_string_lossy().into_owned(),
            enabled,
        });
    }

    Ok(())
}

/// 从 DSH 生态导入一个技能到 PAINT 技能目录（复制 SKILL.md）。
pub fn import_dsh_skill(home: &PaintHome, skill_id: &str) -> Result<SkillInfo, AppError> {
    let source_file = dsh_skills_dir(home).join(skill_id).join("SKILL.md");
    if !source_file.is_file() {
        return Err(AppError::not_found(format!(
            "DSH 技能不存在：{skill_id}"
        )));
    }

    let target_id = if skill_id.starts_with(MANAGED_PREFIX) {
        skill_id.to_string()
    } else {
        format!("{MANAGED_PREFIX}{skill_id}")
    };
    let content = std::fs::read(&source_file)?;
    let target_file = paint_skills_dir(home).join(&target_id).join("SKILL.md");
    write_if_changed(&target_file, &content)?;
    sync_skills(home)?;

    let (name, description) = parse_frontmatter(&String::from_utf8_lossy(&content));
    Ok(SkillInfo {
        id: target_id,
        name: name.unwrap_or_else(|| skill_id.to_string()),
        description: description.unwrap_or_default(),
        source: SkillSource::Paint,
        path: target_file.to_string_lossy().into_owned(),
        enabled: true,
    })
}

/// 解析 YAML frontmatter 里的 `name` / `description`（只认这两行，不引入 yaml 依赖）。
pub fn parse_frontmatter(text: &str) -> (Option<String>, Option<String>) {
    let mut name = None;
    let mut description = None;
    let mut in_frontmatter = false;

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if index == 0 {
            in_frontmatter = trimmed == "---";
            continue;
        }
        if in_frontmatter && trimmed == "---" {
            break;
        }
        if !in_frontmatter {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("name:") {
            name = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("description:") {
            description = Some(value.trim().to_string());
        }
    }

    (name, description)
}

/// 技能文件名（DSH 约定）。
pub const SKILL_FILE: &str = "SKILL.md";

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> (PaintHome, PathBuf) {
        let root = std::env::temp_dir().join(format!("paint-skills-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("创建临时目录");
        (PaintHome::new(&root), root)
    }

    #[test]
    fn builtin_skill_markdown_has_required_frontmatter() {
        let markdown = skill_markdown(&CANVAS_SKILLS[1]);

        assert!(markdown.starts_with("---\n"));
        assert!(markdown.contains("name: canvas-create-layer"));
        assert!(markdown.contains("description: 在画布上新建图层"));
        assert!(markdown.contains("whenToUse:"));
        let (name, description) = parse_frontmatter(&markdown);
        assert_eq!(name.as_deref(), Some("canvas-create-layer"));
        assert!(description.is_some());
    }

    #[test]
    fn all_builtin_ids_are_prefixed_and_unique() {
        let mut ids: Vec<_> = CANVAS_SKILLS.iter().map(|skill| skill.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), total, "技能 id 必须唯一");
        assert!(ids.iter().all(|id| id.starts_with(MANAGED_PREFIX)));
    }

    #[test]
    fn sync_writes_every_skill_manifest_and_protocol() {
        let (paint_home, root) = home();

        let report = sync_skills(&paint_home).expect("同步");

        assert_eq!(report.written.len(), CANVAS_SKILLS.len());
        assert!(report.removed.is_empty());

        let target = dsh_skills_dir(&paint_home);
        for definition in CANVAS_SKILLS {
            assert!(
                target.join(definition.id).join(SKILL_FILE).is_file(),
                "缺少 {}",
                definition.id
            );
        }
        assert!(target.join(MANIFEST_FILE).is_file());
        let protocol = std::fs::read_to_string(paint_home.harness_home().join(PROTOCOL_FILE))
            .expect("协议文件");
        assert!(protocol.contains("paint-ops"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_is_idempotent() {
        let (paint_home, root) = home();
        sync_skills(&paint_home).expect("首次同步");

        let second = sync_skills(&paint_home).expect("再次同步");

        assert!(second.written.is_empty(), "不应重复写入：{:?}", second.written);
        assert_eq!(second.unchanged.len(), CANVAS_SKILLS.len());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disabling_moves_dir_and_removes_synced_copy() {
        let (paint_home, root) = home();
        sync_skills(&paint_home).expect("同步");

        set_skill_enabled(&paint_home, "canvas-create-layer", false).expect("禁用");

        assert!(paint_home
            .skills_dir()
            .join("canvas-create-layer.disabled")
            .is_dir());
        assert!(!dsh_skills_dir(&paint_home).join("canvas-create-layer").exists());

        set_skill_enabled(&paint_home, "canvas-create-layer", true).expect("启用");
        assert!(dsh_skills_dir(&paint_home).join("canvas-create-layer").is_dir());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn stale_managed_skill_is_cleaned_up() {
        let (paint_home, root) = home();
        sync_skills(&paint_home).expect("同步");

        let stale = paint_home.skills_dir().join("canvas-temporary");
        std::fs::create_dir_all(&stale).expect("建目录");
        std::fs::write(stale.join(SKILL_FILE), skill_markdown(&CANVAS_SKILLS[0])).expect("写文件");
        sync_skills(&paint_home).expect("同步");
        assert!(dsh_skills_dir(&paint_home).join("canvas-temporary").is_dir());

        std::fs::remove_dir_all(&stale).expect("删除源");
        let report = sync_skills(&paint_home).expect("再同步");

        assert!(report.removed.contains(&"canvas-temporary".to_string()));
        assert!(!dsh_skills_dir(&paint_home).join("canvas-temporary").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn list_skills_separates_paint_from_dsh_native() {
        let (paint_home, root) = home();
        sync_skills(&paint_home).expect("同步");
        let native = dsh_skills_dir(&paint_home).join("dsh-native");
        std::fs::create_dir_all(&native).expect("建目录");
        std::fs::write(
            native.join(SKILL_FILE),
            "---\nname: dsh-native\ndescription: 生态技能\n---\n",
        )
        .expect("写文件");

        let skills = list_skills(&paint_home).expect("列表");

        let paint_count = skills
            .iter()
            .filter(|skill| skill.source == SkillSource::Paint)
            .count();
        assert_eq!(paint_count, CANVAS_SKILLS.len());
        assert!(skills
            .iter()
            .any(|skill| skill.id == "dsh-native" && skill.source == SkillSource::Dsh));
        assert!(!skills
            .iter()
            .any(|skill| skill.id == "canvas-create-layer" && skill.source == SkillSource::Dsh));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn only_managed_prefix_can_be_toggled() {
        let (paint_home, root) = home();

        let error = set_skill_enabled(&paint_home, "someone-else", false).expect_err("应拒绝");
        assert_eq!(error.kind, crate::error::ErrorKind::InvalidAction);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn import_copies_dsh_skill_into_paint_home() {
        let (paint_home, root) = home();
        sync_skills(&paint_home).expect("同步");
        let native = dsh_skills_dir(&paint_home).join("dsh-chart");
        std::fs::create_dir_all(&native).expect("建目录");
        std::fs::write(
            native.join(SKILL_FILE),
            "---\nname: dsh-chart\ndescription: 图表技能\n---\n正文",
        )
        .expect("写文件");

        let imported = import_dsh_skill(&paint_home, "dsh-chart").expect("导入");

        assert_eq!(imported.id, "canvas-dsh-chart");
        assert!(paint_home
            .skills_dir()
            .join("canvas-dsh-chart")
            .join(SKILL_FILE)
            .is_file());
        assert!(dsh_skills_dir(&paint_home).join("canvas-dsh-chart").is_dir());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn importing_unknown_skill_is_not_found() {
        let (paint_home, root) = home();
        let error = import_dsh_skill(&paint_home, "nope").expect_err("应报错");
        assert_eq!(error.kind, crate::error::ErrorKind::NotFound);
        let _ = std::fs::remove_dir_all(&root);
    }
}
