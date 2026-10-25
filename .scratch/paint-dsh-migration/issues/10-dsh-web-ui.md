# 10: DSH Web UI 嵌入 + 品牌定制

**What to build:**

用户可通过 iframe 嵌入完整的 DSH Web UI，获得类 ChatGPT 的 AI 交互体验，PAINT 品牌已替换 DeepSeek 品牌。

**Blocked by:** 05

**Status:** resolved

- [x] DSH Web UI 通过 iframe 正确嵌入
- [x] 品牌定制生效（DeepSeek → PAINT）
- [x] 主题同步可用

## Comments

- 提交：`cc78fdf`（`src-tauri/src/harness_web.rs` + `open_dsh_web_window`）、`4479512`（工作台面板）。
- **第 1 项的实现方式与规格不同（有实测依据）**：DSH 的启动 URL 带一次性 token，换取的是 `SameSite=Strict` cookie。iframe 属第三方上下文，浏览器不会发该 cookie，`/api` 与 WebSocket 会全部 401。因此改为 Tauri 独立 `WebviewWindow`（label `dsh`，同一 WebView2 配置 → 首方上下文），对用户而言同样是"在工作台里看到完整 DSH 界面"，但不是 `<iframe>` 元素。
- 落盘实现：`node <bin.js> --profile web --no-open --host 127.0.0.1 --port 0` → 解析 stdout 的 `dsh web: <url>` 富 token URL → `DshWebInfo{url,port,pid}`；停止用 `taskkill /T /F`；日志一律脱敏（`token=***`）。
- 品牌：`brand_script()` 是一段自包含 IIFE，替换 DOM 文本（DeepSeek→PAINT）、改写标题、注入 `--paint-accent`，用 MutationObserver 覆盖前 15s 的动态渲染；通过 `WebviewWindowBuilder::initialization_script` 在任何页面脚本之前执行。DSH 没有品牌配置面，所以这是外观层注入，跨版本可能失效（已注明）。
- 主题同步：注入脚本设置 CSS 变量 `--paint-accent`，PAINT 与 DSH 都读同一变量名；未实现完整配色双向同步。
- 验证：`harness_web.rs` 6 个单测（启动参数、就绪行解析、无关行忽略、端口提取、token 脱敏、品牌脚本幂等标记）。**未做**：真实 `dsh web` 启动 + 窗口渲染的端到端验证（需要人工观察 GUI）。
