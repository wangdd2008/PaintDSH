# Issue tracker: Local Markdown

Issues 和 specs 以 markdown 文件形式存储在 `.scratch/` 目录下。

## 约定

- 每个功能一个目录：`.scratch/<feature-slug>/`
- 规格文件为 `.scratch/<feature-slug>/spec.md`
- 实现问题以单个文件形式存放在 `.scratch/<feature-slug>/issues/<NN>-<slug>.md`，从 `01` 开始编号，不要合并为单个文件
- Triage 状态记录在每个 issue 文件顶部的 `Status:` 行（标签字符串见 `triage-labels.md`）
- 评论和对话历史追加到文件底部的 `## Comments` 标题下

## 当技能说 "publish to the issue tracker"

在 `.scratch/<feature-slug>/` 下创建新文件（如目录不存在则创建）。

## 当技能说 "fetch the relevant ticket"

读取所引用路径的文件。用户通常会直接提供路径或 issue 编号。

## Wayfinding 操作

由 `/wayfinder` 使用。**map** 是一个文件，每个 ticket 对应一个 **child** 文件。

- **Map**: `.scratch/<effort>/map.md`（Notes / Decisions-so-far / Fog 内容）
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`，从 `01` 开始编号，问题写在正文中。`Type:` 行记录 ticket 类型（`research`/`prototype`/`grilling`/`task`）；`Status:` 行记录 `claimed`/`resolved`
- **Blocking**: 文件顶部附近的 `Blocked by: NN, NN` 行。当列出的每个文件都 `resolved` 时，该 ticket 解除阻塞
- **Frontier**: 扫描 `.scratch/<effort>/issues/` 中状态为 open、未阻塞、未认领的文件；按编号排序，第一个为胜
- **Claim**: 设置 `Status: claimed` 并在开始工作前保存
- **Resolve**: 在 `## Answer` 标题下追加答案，设置 `Status: resolved`，然后在 `map.md` 的 Decisions-so-far 中追加上下文指针（gist + 链接）
