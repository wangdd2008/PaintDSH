# 统一 Tauri IPC 通信方案

PAINT 的前后端通信从 SSE（Server-Sent Events）全面迁移为 Tauri 原生 IPC（`invoke` + `listen`）。SSE 在原方案中用于前端与 Node.js 后端之间的实时通信，但在 Tauri 2 架构下，Rust 后端与前端之间通过 Tauri IPC 通信更自然、更轻量。AI 网关内部输出的 SSE 格式在 Rust 层解析后，统一通过 Tauri Event 推送到前端。

**考虑过的选项**：保留 SSE（Tauri 中需启动 HTTP server，增加复杂度）；混合方案（SSE + Tauri IPC）。放弃原因是统一 IPC 更简洁，且 Tauri 2 的原生 IPC 性能更好、连接管理更简单。
