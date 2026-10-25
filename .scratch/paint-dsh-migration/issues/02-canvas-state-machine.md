# 02: 画布状态机 + 图层管理 + 持久化

**What to build:**

用户可以在画布上创建、删除、排序、隐藏/显示图层，所有操作持久化到 SQLite。

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Rust 画布状态机可正确执行图层 CRUD 操作
- [ ] 前端图层面板 UI 可用（创建、删除、排序、可见性切换）
- [ ] 数据持久化到 SQLite（canvases、layers 表）
