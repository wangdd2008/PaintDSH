# 09: 知识库 + 技能生态互通

**What to build:**

用户可以管理 PAINT 知识库（画笔模板、配色方案），知识库注册为 DSH 工作区，PAINT 技能与 DSH 技能双向互通。

**Blocked by:** 07, 08

**Status:** resolved

- [x] PAINT 知识库可注册为 DSH 工作区
- [x] 知识库内容可被 AI 引用
- [x] PAINT 技能与 DSH 技能双向互通

## Comments

- 提交：`cc78fdf`（`src-tauri/src/knowledge.rs`）、`4479512`（知识库面板）。
- 目录：`~/.paint/knowledge/{brushes,palettes,shapes,design}`，首次运行写入 `design/design-guide.md` 模板；条目名做路径穿越与非法字符清洗，扩展名限 md/json/svg/txt。
- 注册为 DSH 工作区：`storages/workspace.json` 是 DSH 内部 schema，**不手写**；实现为
  ① `<knowledge>/.paint-workspace.json` 标记（name/kind/version/root）；
  ② 该目录下的 `AGENTS.md`（含条目索引），把知识库目录当 DSH 工作区打开时自动生效；
  ③ 全局 `$DSH_HOME/AGENTS.md` 的 `<!-- PAINT:KNOWLEDGE -->` 区块（幂等替换，不覆盖协议正文）。
- 双向互通：PAINT→DSH 用 `skills::sync_skills`（清单式同步）；DSH→PAINT 用 `skills::import_dsh_skill`（复制 SKILL.md 到 `~/.paint/skills/canvas-<id>`）。
- 验证：`knowledge.rs` 7 个单测（目录骨架、保存/列表/读取/删除、非法分区与名称、注册写标记与两份 AGENTS、区块替换不重复、保留既有协议、调色板名单）；`skills.rs` 覆盖导入。
