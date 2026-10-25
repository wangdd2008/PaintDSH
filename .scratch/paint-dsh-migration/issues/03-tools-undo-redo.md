# 03: 基础工具 + 快照 Undo/Redo

**What to build:**

用户可以使用选择、画笔、橡皮擦、形状、文字工具操作画布，支持 Undo/Redo（快照级）。

**Blocked by:** 02

**Status:** resolved

- [x] 基础工具可用（选择、画笔、橡皮擦、形状、文字）
- [x] 快照机制正确保存每次操作后的完整状态
- [x] Undo/Redo 可正确恢复任意历史状态

## Comments

- 提交：`cc78fdf`（历史服务）、`4479512`（工具栏与画布交互）。
- 历史模型：`snapshots(canvas_id, cursor, snapshot_json)` + `canvases.history_cursor`；新操作先 `truncate_snapshots_after(cursor)` 再写入，天然截断 redo 分支；undo/redo 恢复内容但 `revision` 继续自增，保证乐观锁单调。
- 工具实现（前端 → dispatch，人类与 AI 同一条路径）：
  - 选择：Fabric 对象可拖动，`object:modified` → `set_layer_transform`；
  - 画笔：`path:created` → 把 Fabric path 的 SVG 追加进当前图层 → `set_layer_content`；
  - 形状 / 文字：按下与抬起坐标换算成画布坐标，生成 `<rect>/<ellipse>/<text>` 追加；
  - 橡皮：以画布背景色作为笔刷颜色覆盖（**限制**：矢量图层无法真正擦除像素，已在工具提示里写明；位图图层需要真正的 destination-out 擦除，留作后续增强）。
- 验证：`paint::service` 覆盖 undo/redo 往返、undo 到底/redo 到顶报错、新操作截断 redo、`select_*` 不落快照；前端 `svg.test.ts` 覆盖元素追加顺序与坐标换算（含反向拖动）。
