Status: ready-for-agent

# Spec: PAINT 从 Codex 迁移到 DSH 方案

## Problem Statement

PAINT 项目目前的设计方案基于 OpenAI Codex CLI（Electron + Node.js + Codex），但面临以下问题：

1. **Codex 工具生态狭窄**：Codex 仅支持代码生成，缺乏安全沙箱、Token 计量、数字员工、技能生态等成熟能力。
2. **Electron 性能瓶颈**：空载内存 ~300MB，启动慢，对画图软件来说资源占用过高。
3. **缺乏 AI 运行时管理**：没有 AI 网关统一调度，无法做并发控制、Token 计量、审计追踪。
4. **无法复用成熟模式**：DSH Platform（DevStation）已验证了 Tauri 2 + Rust + DSH 技术栈的可行性，但 PAINT 当前方案与之一致性低。

需要全面迁移到 DeepSeek Harness（DSH）技术栈，以获得更丰富的 AI 能力和更优的性能表现。

## Solution

参考 DSH Platform（DevStation）的成功模式，将 PAINT 从 Electron + Node.js + Codex 全面迁移到 Tauri 2 + Rust + DSH 技术栈。保持"人机同权"核心架构不变，新增 AI 网关、数字员工、技能生态等高级特性。

**关键变更**：
- 桌面壳：Electron → Tauri 2（空载内存从 ~300MB 降至 ~95MB）
- 后端语言：Node.js → Rust
- AI 引擎：Codex CLI → DeepSeek Harness（DSH）
- 通信机制：SSE → Tauri IPC
- 安全策略：无 → 自定义 `canvas-write` 沙箱策略
- AI 工具集：8 个核心工具 → DSH Skill 生态

## User Stories

### 架构层

1. 作为开发者，我希望 PAINT 使用 Tauri 2 作为桌面壳，以便获得更低的内存占用和更快的启动速度。
2. 作为开发者，我希望后端使用 Rust 实现，以便与 DSH 的 headless 子进程管理天然对齐。
3. 作为用户，我希望 PAINT 的 AI 引擎替换为 DeepSeek Harness，以便获得更丰富的 AI 能力和工具链。

### AI 网关

4. 作为用户，我希望所有 AI 调用经过统一的 AI 网关，以便了解每次 AI 操作的 Token 消耗和成本。
5. 作为开发者，我希望 AI 网关支持并发控制（最多 2 个在途请求），以便避免 Agent 工具并发拖垮机器。
6. 作为开发者，我希望 AI 网关记录每次 AI 调用的审计日志（模块、模型、Token、延迟、成功率），以便追踪和优化。

### DSH 集成

7. 作为用户，我希望 PAINT 支持 DSH headless 模式（无人值守执行 AI 画图任务），以便后台自动处理复杂任务。
8. 作为用户，我希望 PAINT 支持 DSH Web UI 模式（iframe 嵌入），以便获得类 ChatGPT 的完整 AI 交互体验。
9. 作为开发者，我希望 DSH 集成包含自定义安全沙箱策略（`canvas-write`），以便允许画布写入操作的同时禁止危险系统命令。

### 数字员工

10. 作为用户，我希望 PAINT 内置"图形设计师"数字员工，以便快速生成图形。
11. 作为用户，我希望 PAINT 内置"滤镜专家"数字员工，以便获得专业的图像处理建议。
12. 作为用户，我希望 PAINT 内置"画布助手"数字员工，以便高效管理图层。
13. 作为用户，我希望自定义数字员工（名称、角色设定、技能、知识库），以便满足个性化需求。

### 技能生态

14. 作为开发者，我希望 PAINT 的画布操作（create_layer、apply_filter 等）封装为 DSH Skill，以便 AI 直接调用。
15. 作为用户，我希望 PAINT 的技能与 DSH 生态互通，以便复用 Harness 的现有技能。

### 知识库

16. 作为用户，我希望 PAINT 建立知识库（画笔模板、配色方案、设计规范），以便 AI 在生成图形时参考。
17. 作为用户，我希望 PAINT 的知识库注册为 DSH 工作区，以便与 deepseek-desktop 共享知识。

### 通信机制

18. 作为开发者，我希望前后端通信统一使用 Tauri IPC，以便获得更简洁、更原生的通信体验。
19. 作为用户，我希望画布变更通过 Tauri IPC 实时推送到前端，以便获得即时的视觉反馈。

### 人机同权

20. 作为用户，我希望人类操作和 AI 操作共用同一套 Command Dispatcher，以便保持统一的操作体验。
21. 作为用户，我希望每次操作后自动生成快照，以便支持 Undo/Redo。

### 数据存储

22. 作为开发者，我希望使用 SQLite WAL 存储画布状态、图层、历史记录，以便获得可靠的持久化能力。
23. 作为用户，我希望作品保存为专有格式（`.paint`），以便保留完整的编辑信息。

### 安全与合规

24. 作为用户，我希望 AI 执行危险操作（如删除图层）前请求确认，以便避免误操作。
25. 作为用户，我希望 AI 的默认安全策略是 read-only（除画布操作外），以便保护系统安全。

## Implementation Decisions

### 1. 技术栈迁移

- **桌面壳**：从 Electron 迁移到 Tauri 2，空载内存从 ~300MB 降至 ~95MB。
- **后端语言**：从 Node.js + TypeScript 迁移到 Rust，与 DSH 的 headless 子进程管理天然对齐。
- **前端框架**：React 19 + TypeScript + Vite + Fabric.js，保留渲染引擎不变（与 AI 引擎解耦）。

### 2. AI 引擎替换

- **AI 引擎**：从 Codex CLI 替换为 DeepSeek Harness（DSH）。
- **双引擎模式**：Harness（复杂任务）+ Direct API（简单对话）。
- **DSH 集成模式**：headless（默认，无人值守）+ Web UI（可选，iframe 嵌入）。

### 3. AI 网关设计

- **统一入口**：所有 AI 调用必须经过 AI 网关。
- **职责**：上下文装配、引擎选择（harness/direct）、并发控制（Semaphore，最多 2 个在途请求）、Token 计量、审计追踪。
- **数据流**：前端发送消息 → Rust Command → 选择引擎 → harness 子进程执行 / direct SSE 请求 → 实时 emit Tauri Event → 前端接收。

### 4. DSH 集成层

- **Headless 子进程管理**：复用 DSH Platform 的 `harness.rs` 逻辑。
  - 全局锁：同一时间只跑一个任务
  - 安全补丁：自定义 `canvas-write` 策略
  - 超时看门狗：180 秒超时自动 SIGKILL
  - 凭据注入：通过 `DEEPSEEK_API_KEY` 环境变量注入
- **Web UI 子进程管理**：复用 DSH Platform 的 `harness_web.rs` 逻辑。
  - 启动本地 Web 服务
  - 应用侧集成：主题同步、知识库工作区注册、数字员工 preset 同步、技能同步
  - 品牌定制：通过 `initialization_script_for_all_frames` 注入品牌替换脚本

### 5. 安全沙箱策略

- **默认策略**：DSH 默认 read-only + approval=never。
- **自定义策略**：`canvas-write`，允许画布相关的写入操作（图层增删、像素修改），内核级拒绝 rm/mkfs/dd 等危险系统命令。
- **实现方式**：通过 `--patch` 参数注入 DSH。

### 6. AI 工具集设计

- **DSH Skill**：每个画布操作（create_layer、apply_filter、select_region 等）封装为独立的 `SKILL.md`。
- **技能目录**：`~/.paint/skills/<id>/SKILL.md`，启用后自动同步到 Harness 技能目录。
- **工具调用协议**：DSH 的自然语言描述 + 参数 JSON 格式。

### 7. 数字员工系统

- **内置三类**：图形设计师、滤镜专家、画布助手。
- **同步机制**：同步为 Harness agent preset，写入 `~/.paint/harness/.agent-presets/<preset_id>/`。
- **自定义**：支持名称 + 角色设定 + 技能 + 挂接知识库。

### 8. 知识库互通

- **路径**：`~/.paint/knowledge/`
- **内容**：画笔模板、配色方案、形状模板、设计规范。
- **注册**：作为 DSH 工作区注册，AI 生成图形时可引用。

### 9. 通信机制

- **统一 Tauri IPC**：前端通过 `invoke` 调用 Rust Command，通过 `listen` 订阅 Rust 事件。
- **事件类型**：`canvas://changed`（画布变更）、`ai://delta`（AI 输出增量）、`ai://done`（AI 完成）。
- **AI 网关内部**：harness 子进程的 stdout 逐行读取，direct 引擎的 SSE 流式解析，统一转换为 Tauri Event 推送到前端。

### 10. 数据层设计

- **数据库**：SQLite WAL，复用 DSH Platform 的 `store.rs` 设计。
- **核心表**：`canvases`、`layers`、`snapshots`、`ai_calls`、`ai_sessions`、`ai_messages`、`employees`。
- **文件格式**：专有格式 `.paint`（JSON 序列化，完整画布状态），导出 PNG/JPG/SVG。

### 11. 人机同权架构

- **Command Dispatcher**：统一入口，接收 `applyAction(action, input, actor)`。
- **actor 区分**：`user`（人类）| `agent`（AI），状态机不区分 actor。
- **快照机制**：每次操作后保存完整状态深拷贝，支持快照级 Undo/Redo。

## Testing Decisions

### 测试 seams

由于目前仓库处于设计阶段，测试 seams 需要在实现时建立：

1. **AI Gateway seam**：测试 AI 调用统一入口（引擎选择、Token 计量、并发控制、审计追踪）。
   - 输入：不同引擎（harness/direct）、不同任务、并发请求。
   - 输出：正确的引擎选择、Token 计量准确、并发控制生效、审计日志完整。

2. **Command Dispatcher seam**：测试人机同权命令分发。
   - 输入：不同 actor（user/agent）、不同 action、不同输入。
   - 输出：状态机正确执行、SSE 事件正确广播、快照正确保存。

3. **Canvas State Machine seam**：测试画布状态转移和快照机制。
   - 输入：不同操作（创建图层、删除图层、应用滤镜、Undo、Redo）。
   - 输出：状态正确转移、快照正确保存和恢复、乐观锁冲突检测。

4. **DSH Integration seam**：测试 DSH 子进程管理和安全沙箱。
   - 输入：不同任务、不同安全策略、超时场景。
   - 输出：子进程正确启动和终止、沙箱策略正确生效、超时正确触发。

### 测试策略

- **单元测试**：Rust 层（`cargo test`）测试核心模块（状态机、AI 网关、DSH 集成）。
- **集成测试**：Playwright 测试前端交互（画布操作、AI 对话、Undo/Redo）。
- **端到端测试**：Vitest + Playwright 测试完整工作流（创建画布 → AI 生成图形 → 导出图片）。

### 测试优先级

1. **P0（关键路径）**：AI Gateway → Command Dispatcher → Canvas State Machine
2. **P1（重要）**：DSH Integration → 数字员工 → 技能生态
3. **P2（次要）**：知识库 → 主题同步 → 品牌定制

## Out of Scope

以下功能不在本次迁移范围内：

1. **云同步**：作品云同步、跨设备协作。
2. **社区模板市场**：用户上传和分享模板。
3. **插件系统（运行时加载）**：自定义工具和滤镜的运行时扩展。
4. **多语言支持**：国际化（i18n）不在本次范围内。
5. **高级图像生成 API**：照片级真实感图像生成（DSH 代码生成模式不支持）。
6. **移动端**：仅桌面端（Tauri 2）。

## Further Notes

### 参考材料

- `docs/dsh-platform-reference/01-核心调研与设计文档/DESIGN.md` — DSH Platform 完整设计文档
- `docs/dsh-platform-reference/01-核心调研与设计文档/research-dsh-platform.md` — DSH Platform 深度调研报告
- `docs/adr/0001-tauri-2-migration.md` — Tauri 2 迁移决策
- `docs/adr/0002-dsh-replacement.md` — DSH 替换决策
- `docs/adr/0003-canvas-sandbox.md` — 自定义安全沙箱决策
- `docs/adr/0004-unified-tauri-ipc.md` — 统一 Tauri IPC 决策
- `GLOSSARY.md` — PAINT + DSH 术语表

### 关键依赖

- **Tauri 2**：桌面壳框架
- **DeepSeek Harness**：AI 引擎（需要用户安装 deepseek-desktop）
- **Fabric.js**：Canvas 2D 渲染引擎
- **SQLite**：数据持久化

### 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 团队 Rust 学习曲线 | 高 | 参考 DSH Platform 源码，复用成熟模式 |
| DSH API 稳定性 | 中 | 封装 DSH 集成层，隔离变化 |
| Fabric.js 与 Tauri 2 兼容性 | 低 | Fabric.js 是纯前端库，与 Tauri 无冲突 |
| 性能（大画布、多图层） | 中 | Phase 4 专门优化 |

### 验收标准

1. [ ] Tauri 2 项目结构初始化完成，可正常启动
2. [ ] Fabric.js 画布渲染正常，基础工具可用
3. [ ] DSH headless 子进程可正常启动和执行任务
4. [ ] AI 网关可正确选择引擎（harness/direct）
5. [ ] Token 计量和审计日志正确记录
6. [ ] 自定义安全沙箱策略生效（允许画布写入，禁止危险命令）
7. [ ] 数字员工系统可用（内置三类 + 自定义）
8. [ ] 技能生态互通（PAINT 技能 ↔ DSH 技能）
9. [ ] 知识库注册为 DSH 工作区
10. [ ] Tauri IPC 通信正常（画布变更、AI 输出实时推送）
11. [ ] 快照级 Undo/Redo 可用
12. [ ] 文件保存/打开（`.paint`）和导出（PNG/JPG/SVG）可用
