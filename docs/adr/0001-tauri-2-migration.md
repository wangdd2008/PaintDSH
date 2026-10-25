# 从 Electron 迁移到 Tauri 2

PAINT 的桌面壳从 Electron 全面迁移到 Tauri 2，后端语言从 Node.js/TypeScript 改为 Rust。这一决策使空载内存从 Electron 的 ~300MB 降至 Tauri 2 的 ~95MB，并与 DSH（DeepSeek Harness）生态对齐——DSH Platform 的成功模式建立在 Tauri 2 之上。代价是团队需要学习 Rust，且 Electron 丰富的第三方生态无法直接复用。

**考虑过的选项**：保留 Electron + Node.js 后端，通过 child_process 调用 DSH。放弃原因是 Tauri 2 的 Rust 后端与 DSH 的 headless 子进程管理（harness.rs）天然对齐，且性能优势明显。
