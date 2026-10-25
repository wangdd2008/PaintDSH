# 02: 画布状态机 + 图层管理 + 持久化

**What to build:**

用户可以在画布上创建、删除、排序、隐藏/显示图层，所有操作持久化到 SQLite。

**Blocked by:** 01

**Status:** resolved

- [x] Rust 画布状态机可正确执行图层 CRUD 操作
- [x] 前端图层面板 UI 可用（创建、删除、排序、可见性切换）
- [x] 数据持久化到 SQLite（canvases、layers 表）

## Comments

- 提交：`cc78fdf`（后端）、`4479512`（前端）。
- 后端分层：`paint::document`（模型）→ `paint::state_machine`（纯 reducer，按 action 白名单）→ `paint::service`（乐观锁 + 持久化 + 快照）→ `paint::command_dispatcher`（唯一写入口 + `canvas://changed` 事件）；SQL 全部收在 `paint::repository`。
- 关键取舍：
  - `CanvasDocument` 增加 `selectedRegion`（`select_region` 需要落点，契约已补）；
  - 图层表按 `layer_json` 整行存 JSON，保存时整表替换——百图层规模下简单可靠优于增量 diff；
  - 迁移用 `CREATE TABLE IF NOT EXISTS` + 顺序执行，`store::MIGRATIONS` 由各模块常量拼装。
- 验证：`paint::state_machine` 14 个单测覆盖全部 action 的合法/非法路径、actor 无关性、revision 乐观锁；`paint::repository` 覆盖读写往返、级联删除、游标保持；`npm run test` 覆盖图层面板与 store 乐观锁。
- 踩坑：`#[serde(rename_all = "camelCase")]` 只改变体名，不改变体字段，`LayerContent` 必须额外加 `rename_all_fields = "camelCase"`，否则前端拿到 `data_url`。
