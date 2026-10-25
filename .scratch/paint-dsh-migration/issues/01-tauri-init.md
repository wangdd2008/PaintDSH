# 01: 初始化 Tauri 2 + Fabric.js 基础画布

**What to build:**

用户可以看到一个可交互的空白画布窗口，Tauri 2 项目结构就绪，Fabric.js 正确渲染。

**Blocked by:** None（可立即开始）

**Status:** resolved

- [x] Tauri 2 + React + Vite 项目初始化完成，cargo tauri dev 可正常启动
- [x] Fabric.js 集成到画布组件，空白画布正确渲染
- [x] Tauri IPC 通信框架就绪（前后端可 ping 通）

## Comments

- 提交：`9f28b68`（分支 `ticket/01-tauri-init`），合并：`3ec3a06`。
- 验证：`npm run typecheck` 0 error；`npm run test` 29 passed；`npm run build` 产出 `dist/`；`cargo test` 15 passed；`cargo build` 产出 `paint-dsh.exe`。
- 第 1 项说明：以 `cargo build`（app 二进制）+ `vite build` 证明可构建；`npm run tauri dev` 会拉起 GUI 窗口，未在工作流中实际打开窗口验证。
- 环境踩坑（已写进 `docs/engineering-contract.md` 10.1）：
  - 本机 rustc 1.99.1 编译 `serde_core`（serde ≥1.0.220 拆出的内部 crate）会触发 rustc ICE，必须钉 `serde =1.0.219`；
  - tauri 2.12.x 的 `tauri-build 2.7.4` 经 `cargo_toml` 强拉新版 serde，因此 tauri 钉 `=2.11.6`，并把 `tauri-macros` / `tauri-codegen` 钉 2.6.3、`tauri-runtime` 钉 2.11.3、`tauri-runtime-wry` 钉 2.11.4（这些 caret 范围会拉到互不兼容的新版）；
  - `#[tauri::command]` 不能定义在 crate 根（`#[macro_export]` 与 `pub use` 撞宏命名空间，E0255），统一放进 `commands` 子模块；测试模块也不要 `use super::*`。
- 图标：`scripts/generate-icon-source.mjs` 手写 PNG/ICO，再由 `tauri icon` 生成整套，避免二进制资源靠人工提交。
