# 04: 文件保存/打开 + 导出

**What to build:**

用户可以将作品保存为 .paint 格式并重新打开，支持导出为 PNG/JPG/SVG。

**Blocked by:** 03

**Status:** resolved

- [x] 文件保存/打开（.paint 格式）正确工作
- [x] 导出 PNG/JPG/SVG 正确生成
- [x] 文件菜单和导出对话框 UI 可用

## Comments

- 提交：`cc78fdf`（`paint::files` + 命令）、`4479512`（菜单与导出）。
- `.paint` = JSON：`{ version, canvas, history, cursor }`；打开时校验版本（高版本拒绝）、校验文档结构、覆盖同 id 画布的历史、无历史时补基线快照。写入走 `fsutil::write_atomic`（tmp → rename），避免半截文件。
- 导出：前端 Fabric 生成 data URL（SVG 走 `canvas.toSVG()` 再 base64），Rust 侧校验 MIME 与格式一致后解码落盘，并自动补扩展名；非法 data URL / 空内容 / 超 64MiB 全部拒绝。
- 对话框用 `rfd`（阻塞式原生对话框，跑在 `spawn_blocking` 里，不冻结 UI）。
- 验证：`paint::files` 8 个单测覆盖保存→新库打开往返、版本过高拒绝、损坏 JSON、无历史补基线、PNG 字节落盘、格式/MIME 不一致、SVG base64、扩展名补全。
