# PAINT

基于 **Tauri 2 + React 19 + Fabric.js 6** 的 AI 辅助画图工具。画布状态由 Rust 侧统一持有，人类操作与 AI 操作走**同一条写入通道**；AI 能力接在本地已安装的 **DSH（DeepSeek Harness）** 上，也可降级为直连 DeepSeek API。

> 状态：`0.1.0`，Windows 桌面端。规格见 `.scratch/paint-dsh-migration/spec.md`，实现契约见 `docs/engineering-contract.md`。

---

## 1. 核心特性

| 能力 | 说明 |
| --- | --- |
| 画布与图层 | 创建/删除/排序/可见性/透明度/混合模式/锁定，全部持久化到 SQLite（WAL） |
| 绘图工具 | 选择（拖动提交变换）、画笔、橡皮、矩形/椭圆、文字 |
| 历史 | 快照级 Undo/Redo；新操作自动截断 redo 分支；revision 单调，支持乐观锁 |
| 文件 | `.paint`（JSON：画布 + 完整历史）原子读写；导出 PNG / JPG / SVG |
| AI 双引擎 | `harness`（DSH headless 子进程）与 `direct`（DeepSeek SSE 直连）；自动选择 + 不可用降级 |
| AI 改画布 | AI 在答复里输出 ` ```paint-ops ` 代码块，由 PAINT 解析并按 action 白名单执行；删除类操作需用户确认 |
| Token 计量 | 每次调用记录 engine/model/tokens/latency/成败，支持按日汇总 |
| 数字员工 | 内置图形设计师、滤镜专家、画布助手；可自定义人设/技能/知识库，并同步为 DSH persona patch |
| 知识库 | 画笔/配色/形状/规范四分区，可注册为 DSH 工作区，内容经 `AGENTS.md` 索引被 AI 引用 |
| 技能互通 | 7 个内置 `canvas-*` Skill 同步到 DSH；DSH 生态技能可导入 PAINT |
| DSH 工作台 | 启动 `dsh web` 并用独立窗口承载完整对话界面，品牌脚本注入为 PAINT |

---

## 2. 架构要点

```
┌──────────────────────────── React 前端 ────────────────────────────┐
│ MenuBar / Toolbox / CanvasStage(Fabric.js) / LayerPanel / AI 面板   │
│                     zustand（只读快照 + UI 状态）                    │
└───────────────┬───────────────────────────────▲───────────────────┘
        invoke  │ 命令                            │ 事件 canvas://changed
                ▼                                │ ai://delta|done|error|pending-op
┌──────────────────────────── Rust 后端 ──────────┴───────────────────┐
│ commands（Tauri 命令层）                                             │
│   ├── paint::command_dispatcher  ← 人类与 AI 的唯一写入口            │
│   │      ├── paint::state_machine（纯 reducer，action 白名单）        │
│   │      ├── paint::service（乐观锁 + 快照 + 持久化）                 │
│   │      └── paint::repository（SQLite）                             │
│   ├── ai::gateway（引擎选择 / 并发控制 / 计量 / 审计）                │
│   │      ├── ai::engines（HarnessEngine / DirectEngine）             │
│   │      └── ai::ops（paint-ops 解析 → dispatcher）                  │
│   ├── harness.rs（DSH headless：命令拼装、NDJSON 解析、看门狗）        │
│   ├── harness_web.rs（dsh web 子进程 + 品牌注入）                    │
│   ├── skills.rs / employees.rs / knowledge.rs（DSH 生态同步）         │
│   └── store.rs（连接 + PRAGMA + 顺序迁移）                           │
└─────────────────────────────────────────────────────────────────────┘
```

三条不可破坏的约束（详见 `docs/engineering-contract.md`）：

1. **Rust 是画布状态的唯一真相源**，前端只渲染下发的 `CanvasDocument`，不维护第二份可编辑状态。
2. **所有写操作经 Command Dispatcher**，AI 没有旁路；它只能"提议"操作。
3. **业务逻辑不直接摸 `AppHandle` / 环境变量**：事件走 `EventSink`，路径走 `PaintHome`，因此可注入、可单测。

---

## 3. 环境要求

| 依赖 | 版本 | 备注 |
| --- | --- | --- |
| Node.js | ≥ 20（本机 24） | 前端与 Tauri CLI |
| Rust | stable（本机 1.99.1） | 需要 MSVC 工具链 |
| VS Build Tools | 2022，勾选「使用 C++ 的桌面开发」 | 提供 `link.exe` |
| WebView2 Runtime | Windows 11 自带 | Tauri 渲染内核 |
| DSH（可选） | `@deepseek-ai/dsh` 0.2.1+ | AI 改画布与工作台需要；不装则仅剩 direct 引擎 |

> **依赖钉版警告**：`src-tauri/Cargo.lock` 把 tauri / serde 全链钉在特定版本（原因见契约 10.1：rustc 1.99.1 编译 `serde_core` 会触发编译器 ICE）。**不要随手 `cargo update`**，否则会拉到互不兼容的 tauri 组件。

---

## 4. 快速开始

```bash
# 1. 安装前端依赖
npm install

# 2. 开发（前端 dev server + Tauri 窗口）
npm run tauri dev

# 3. 只跑纯前端（浏览器里没有 Rust 后端，AI/保存等操作不可用）
npm run dev
```

生产构建：

```bash
npm run build                      # typecheck + vite build → dist/
npm run tauri build                # 打包安装器（需要一次性的 NSIS/WiX 下载）
```

首次启动会自动创建数据目录、写入内置数字员工与 7 个画布技能，并把技能/人设/知识库索引同步到 PAINT 自己的 `DSH_HOME`。

---

## 5. 配置（环境变量）

| 变量 | 默认值 | 作用 |
| --- | --- | --- |
| `DEEPSEEK_API_KEY` | 空 | **必填**（否则两个 AI 引擎都不可用）。同时用于 harness 与 direct |
| `PAINT_HOME` | `~/.paint` | 数据根目录（便携模式或测试用） |
| `PAINT_DSH_ENTRY` | `%APPDATA%\npm\node_modules\@deepseek-ai\dsh\lib\bin.js` | DSH 入口文件 |
| `PAINT_NODE_EXE` | PATH 里的 `node` | node 可执行文件 |
| `PAINT_HARNESS_SANDBOX` | `canvas-write` | 沙箱档位：`canvas-write`（工作区可写）或 `read-only` |

---

## 6. 数据目录

```
~/.paint/
├── paint.db                     # SQLite（canvases / layers / snapshots / ai_* / employees）
├── skills/                      # PAINT 技能源（canvas-*/SKILL.md，禁用时改名 *.disabled）
├── knowledge/                   # 知识库四分区 + .paint-workspace.json 标记
│   ├── brushes/  palettes/  shapes/  design/
│   └── AGENTS.md
└── harness/                     # PAINT 专用 DSH_HOME
    ├── AGENTS.md                # 画布协议 + 知识库索引（DSH 每次 run 注入）
    ├── skills/                  # 同步过去的 canvas-* 技能 + .paint-manifest.json
    ├── patches/                 # 沙箱策略与员工 persona patch
    ├── workspaces/<canvasId>/    # headless 的 cwd（沙箱边界）
    └── …（DSH 自己的 sessions / storages / profiles）
```

---

## 7. AI 是怎么改画布的

AI 没有执行权限。约定它在最终答复里输出：

````markdown
```paint-ops
[{"action":"create_layer","input":{
  "name":"红色圆","type":"aiGenerated",
  "content":{"type":"aiCode","code":"<circle cx=\"160\" cy=\"120\" r=\"60\" fill=\"#e5484d\"/>"},
  "prompt":"画一个红色的圆"}}]
```
````

PAINT 侧的处理链：

```
ai::ops::extract_ops
  → action 白名单（create_layer / set_layer_content / apply_filter / …）
  → 单条 ≤ 20 个操作
  → command_dispatcher::dispatch（actor = agent）
  → canvas://changed → 前端重绘
```

- `delete_layer` 属高风险：默认不执行，发 `ai://pending-op` 等前端确认。
- 单条失败不影响后续，失败原因逐条记录在 `OpsReport.rejected`。
- 协议正文写在 `$PAINT_HOME/harness/AGENTS.md`，随每次同步幂等更新。

---

## 8. 与 DSH 的集成（三处注意）

1. **沙箱是文件写边界，不是命令黑名单。** `canvas-write` 用 `--patch` 覆盖 `sandbox-policy` / `approval` / `permission` / `tool-web`，把写入限制在工作区；真正的操作白名单在 PAINT 的 `paint-ops`。
2. **数字员工不是 agent preset。** DSH 0.2.1 没有 `.agent-presets/`，员工人设落成 `harness/patches/employee-<id>.patch.yml`，覆盖 `system-prompt.personaPrefix`。
3. **Web UI 不能用 iframe。** 启动 token 换的是 `SameSite=Strict` cookie，iframe 属第三方上下文会 401；改为独立 `WebviewWindow` + `initialization_script` 品牌注入。

---

## 9. 常用命令

| 命令 | 说明 |
| --- | --- |
| `npm run dev` | Vite dev server（1420） |
| `npm run tauri dev` | 前端 + Tauri 窗口 |
| `npm run build` | `tsc --noEmit` + `vite build` |
| `npm run typecheck` | 只做类型检查 |
| `npm run test` | Vitest（jsdom，fabric 全部 mock） |
| `cd src-tauri && cargo test` | Rust 单测（含真实子进程/看门狗用例） |
| `cd src-tauri && cargo clippy --all-targets -- -D warnings` | 静态检查（当前 0 告警） |
| `cd src-tauri && cargo build` | 编译 app 二进制 |

---

## 10. 目录结构

```
├── src/                          # React 前端
│   ├── lib/{types,bridge,errors}.ts   # DTO 镜像 / IPC 唯一封装 / 错误解析
│   ├── stores/paint.ts           # zustand：只读快照 + UI 状态
│   └── features/
│       ├── canvas/               # engine(Fabric 封装) / CanvasStage / svg / filters
│       ├── layers/ toolbox/ menu/ status/ filters/ knowledge/
│       └── ai/                   # ChatPanel / EmployeePanel / HarnessView
├── src-tauri/src/                # Rust 后端
│   ├── paint/                    # document / state_machine / service / repository / files
│   ├── ai/                       # gateway / engines / ops / context / repository
│   └── harness.rs harness_web.rs skills.rs employees.rs knowledge.rs store.rs paths.rs
├── docs/
│   ├── architecture-design.md    # 架构设计与选型
│   ├── engineering-contract.md   # 跨模块契约（IPC/类型/存储/测试/钉版）
│   ├── adr/                      # 4 份架构决策记录
│   └── dsh-platform-reference/   # DSH 平台调研与设计原型
├── .scratch/paint-dsh-migration/ # 规格与 11 张已关闭的 ticket
└── scripts/generate-icon-source.mjs
```

---

## 11. 已知限制 / 未验证项

- **未做真实端到端验证**：本机没有 `DEEPSEEK_API_KEY`，`dsh --profile headless` 与 DeepSeek 直连只经过纯逻辑与子进程桩测试。
- **未实际打开 GUI 窗口验证**：以 `cargo build` + `vite build` 证明可构建，未跑 `npm run tauri dev` 观察交互。
- **橡皮是"背景色覆盖"**：矢量图层无法真正擦除像素，位图图层的 destination-out 擦除留待后续。
- **滤镜只作用于位图图层**：矢量/aiCode 图层需先栅格化。
- **品牌注入是外观层**：DSH 没有品牌配置面，脚本替换 DOM 文本与 CSS 变量，跨版本可能失效；主题仅同步 `--paint-accent`。
- **测试进程用例依赖 Windows**：`harness::run` 的看门狗/回放用例用 `cmd.exe` 与 `taskkill`。

---

## 12. 质量基线

| 项目 | 结果 |
| --- | --- |
| `cargo test` | 149 passed / 0 failed |
| `cargo clippy --all-targets -D warnings` | 0 告警 |
| `npm run test` | 58 passed / 8 files |
| `npm run typecheck` | 0 error |
| `npm run build` / `cargo build` | 成功 |

设计原则里最值得强调的一条：**任何 AI 相关的失败都不会污染画布状态**——AI 只能提议 `paint-ops`，落库与事件广播始终由 Dispatcher 统一完成。

---

## 13. 许可

仓库尚未包含 LICENSE 文件，默认保留所有权利；如需开源请先补充许可证。
