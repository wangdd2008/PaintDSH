# DSH Platform 深度调研报告

> 调研时间：2026-10-25
> 调研范围：源码级分析（Rust + TypeScript）
> 目标：理解项目如何利用 DSH（DeepSeek Harness）构建 AI 原生桌面应用

---

## 一、项目定位：什么是 DSH Platform

DSH Platform（代号 DevStation）是一个**面向研发者的智能桌面工作台**，核心理念是**"桌面即应用（Desktop as the Application）"**。

它将开发者日常散落的各种工具（IDE、终端、Postman、Git 客户端、笔记、监控仪表、AI 聊天页）折叠进一个原生质感的桌面应用中，以 **DeepSeek Harness（deepseek-v4-flash）** 作为贯穿全部模块的智能底座。

### 1.1 技术栈

| 层级 | 选型 | 说明 |
|------|------|------|
| 桌面壳 | Tauri 2 | 基于系统 WKWebView，空载 ~95MB，远低于 Electron |
| 前端 | React 19 + TypeScript + Vite | 组件化、HMR |
| 后端 | Rust + Tauri 2 | 高性能系统级 API 访问 |
| AI 引擎 | DeepSeek Harness (DSH) | headless 子进程 + Web UI 子进程双模式 |
| 终端 | portable-pty + xterm.js | 多标签本地终端 |
| 编辑器 | CodeMirror 6 | 多语言高亮 |
| 存储 | SQLite WAL + 文件 | 本地优先，数据不落云端 |

---

## 二、DSH（DeepSeek Harness）是什么

DSH 是 **DeepSeek 官方推出的桌面 AI 助手运行时**，即 `deepseek-desktop` 的核心。

- **本质**：一个 Node.js CLI 工具，位于 `node_modules/@deepseek-ai/dsh/lib/bin.js`
- **能力**：自带 bash/fs/editor 等工具 + 会话管理 + 插件/MCP 生态
- **运行模式**：
  - `headless`：命令行模式，单次任务执行
  - `web`：启动本地 Web UI 服务（类似 ChatGPT 界面）
- **数据目录**：`~/.dsh-platform/harness`（DSH_HOME），存放会话、配置、插件、技能等

---

## 三、项目如何利用 DSH 实现

DSH Platform 对 DSH 的集成采用**"双引擎 + 双模式"**架构：

### 3.1 双引擎架构

```
用户输入 → AI 网关（Rust 后端）
           ├─ engine = harness → DeepSeek Harness headless 子进程
           │    └─ 自带 bash/fs/editor 工具 + 会话管理
           ├─ engine = direct  → reqwest SSE → api.deepseek.com
           │    └─ 逐字流式 + reasoning_content 分离
           └─ 内置凭据（零配置，无密钥管理 UI）
```

### 3.2 双模式集成

#### 模式 A：Headless 子进程（默认引擎）

**位置**：`src-tauri/src/harness.rs`

**原理**：
```
node <dsh>/lib/bin.js --profile headless "<任务文本>"
```

**关键设计**：
- 以 **headless 子进程**方式调用 DSH，stdout 逐行输出，stderr 截尾用于报错
- 凭据通过 `DEEPSEEK_API_KEY` 环境变量注入，不落盘
- 状态独立存放于 `~/.dsh-platform/harness`
- **全局锁机制**：`RUN_LOCK` 保证同一时间只跑一个任务（避免 Agent 工具并发拖垮机器）
- **超时看门狗**：180 秒超时自动 SIGKILL
- **安全沙箱**：默认 read-only sandbox，approval=never，内核级拒绝 rm/mkfs/dd 等危险命令

**代码核心**（`harness.rs` 第 140-234 行）：
```rust
pub fn run_headless(paths: &HarnessPaths, api_key: &str, task: &str, cwd: Option<&Path>, on_line: impl Fn(&str)) -> HarnessOutcome {
    // 1. 全局锁：同一时间只跑一个任务
    let _guard = RUN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    
    // 2. 写入 headless 安全补丁（read-only sandbox）
    let sandbox_patch = write_headless_sandbox_patch();
    
    // 3. 启动子进程
    let mut child = Command::new(&paths.node_executable)
        .arg(&paths.dsh_entry)
        .arg("--profile").arg("headless")
        .arg("--patch").arg(&sandbox_patch)
        .env("DSH_HOME", &paths.dsh_home)
        .env("DEEPSEEK_API_KEY", api_key)
        .spawn()?;
    
    // 4. 逐行读取 stdout，实时回调前端
    // 5. 超时看门狗自动 SIGKILL
}
```

#### 模式 B：Web UI 子进程（完整 Harness 界面）

**位置**：`src-tauri/src/harness_web.rs`、`src/ai/HarnessView.tsx`

**原理**：
```
node <dsh>/lib/bin.js web --host 127.0.0.1 --port <随机>
```

**关键设计**：
- 启动本地 Web 服务后，前端通过 **iframe 嵌入** Harness 的完整界面
- 获得 deepseek-desktop 同款体验：会话管理、工作区、插件、技能、MCP、模型厂商管理
- 启动前自动完成多项"应用侧集成"：
  - **深色主题同步**：写入 `settings.yaml` 使 Harness UI 与平台外观一致
  - **知识库工作区注册**：将 `~/.dsh-platform/knowledge` 注册为 Harness 工作区
  - **数字员工同步**：将平台内置/自定义数字员工同步为 Harness agent preset
  - **技能同步**：将启用的技能从 `~/.dsh-platform/skills` 同步到 Harness 技能目录
  - **管控补丁**：禁用外部网络检索，AI 只使用应用内知识库与技能

**品牌定制**（`harness_web.rs` 第 149-192 行）：
```javascript
// 注入脚本到 iframe，将 DeepSeek 品牌替换为「AI 助手」
// 1) 隐藏鱼形图标，品牌字标替换
// 2) 深色主题色阶映射为平台蓝调深色
// 3) 欢迎语替换为应用内文案
```

### 3.3 AI 网关统一调度

**位置**：`src-tauri/src/ai/mod.rs`

AI 网关是平台所有 AI 调用的唯一入口，承担以下职责：

1. **上下文装配**：system prompt + 历史会话 + 附加上下文（代码、文档等）
2. **并发控制**：全局 Semaphore，同一时刻最多 2 个在途请求
3. **Token 计量**：所有调用统一经网关计量，落 SQLite `ai_calls` 表
4. **审计追踪**：记录模块、员工、任务描述、模型、token 数、延迟、成功率
5. **SSE 流式解析**：实时解析 `data: {...}` 格式，分离 content 和 reasoning_content
6. **多模态支持**：图片附件自动落盘，走 direct 引擎 + vision 模型

**关键数据流**：
```
前端发送消息
  → ai_chat_start (Rust command)
    → 选择引擎（harness/direct）
      → harness: run_headless() 子进程执行
      → direct: stream_chat() SSE 流式请求 api.deepseek.com
    → 实时 emit ai://delta 事件到前端
    → 完成 emit ai://done + 计量入库
```

---

## 四、项目本身做了哪些（非 DSH 部分）

### 4.1 13 个功能模块

| 模块 | 说明 |
|------|------|
| **工作台（Dashboard）** | 时段问候、系统状态、4 张指标卡、今日任务 CRUD、快捷入口 |
| **AI 助手** | 双引擎对话、流式渲染、Markdown、思考过程折叠、6 个快捷指令 |
| **数字员工** | 内置五岗（代码分析/问题排查/接口文档/业务流程/表结构）+ 自定义员工 |
| **监控** | CPU P/E 核、内存、磁盘、网络、进程树、端口查进程、启动链路追踪 |
| **代码编辑器** | CodeMirror 6、项目文件树、多标签编辑、多语言高亮 |
| **终端** | portable-pty + xterm.js、多标签真终端 |
| **建模中心** | 概念/逻辑/物理三层 ER 图、业务流程图、接口拓扑图、AI 生成 |
| **工具箱** | JSON 格式化、Token 用量统计、文件比对（Beyond Compare 式）、hprof 堆分析 |
| **微服务运行** | Maven/JDK 配置、多项目并行启动、实时日志、进程管理 |
| **项目与 Git** | 分支状态、提交/暂存/拉取/推送、Diff 着色、提交历史 |
| **API 调试** | 7 种 HTTP 方法、请求头/体、JSON 美化、会话历史 |
| **知识库** | Markdown 笔记、编辑/预览、全文搜索 |
| **设置** | 通用/AI/用量/外观/终端配置 |

### 4.2 低功耗设计

- **PowerState 状态机**：Active 1s → Idle 5s → Background 30s
- Rust 是唯一采样源，前端零 setInterval
- 空载内存 ~95MB，空载 CPU 0.3%

### 4.3 数据层设计

**SQLite 表结构**（`src-tauri/src/store.rs`）：
- `tasks`：工作台任务
- `ai_sessions` / `ai_messages`：AI 会话与消息
- `ai_calls`：AI 调用计量（模块、模型、token、延迟、成功率）
- `models`：建模文档（ER 图、流程图等）
- `employees`：自定义数字员工

---

## 五、集成 DSH 的具体工作（可复用的技术模式）

### 5.1 子进程生命周期管理

```rust
// 路径解析：自动探测或手动指定 deepseek-desktop 目录
pub fn resolve_paths(harness_dir: Option<&str>) -> Result<HarnessPaths, String>;

// Headless 单次任务执行
pub fn run_headless(paths: &HarnessPaths, api_key: &str, task: &str, cwd: Option<&Path>, on_line: impl Fn(&str)) -> HarnessOutcome;

// Web UI 子进程管理（start/stop/restart/status）
pub fn start(state: HarnessWebState, db: Arc<Db>);
pub fn stop(state: &HarnessWebState);
```

### 5.2 安全沙箱设计

```yaml
# headless-sandbox.patch.yml
- id: approval
  name: '@deepseek-ai/dsh-user-approval'
  config:
    policy: never  # 无人值守不弹审批

- id: permission
  name: '@deepseek-ai/dsh-permission-presets'
  config:
    presets:
      read-only:
        sandbox: read-only  # 内核级只读
        approval: never
```

### 5.3 品牌与主题定制

- 通过 Tauri `initialization_script_for_all_frames` 注入品牌替换脚本
- 替换 DeepSeek 品牌字标为「AI 助手」
- 同步深色主题色阶
- 仅作用于 127.0.0.1 的 Harness 页面

### 5.4 数据互通（DSH_HOME 共享）

```
~/.dsh-platform/
├── config.toml          # 平台配置
├── platform.db          # SQLite 数据库
├── harness/             # DSH_HOME（与 deepseek-desktop 共享）
│   ├── settings.yaml    # Harness 配置
│   ├── storages/        # 会话存储
│   ├── skills/          # 同步的技能
│   └── .agent-presets/  # 同步的数字员工 preset
├── knowledge/           # 知识库（注册为 Harness 工作区）
└── skills/              # 平台技能主目录
```

### 5.5 数字员工作为 Agent Preset

```rust
// 将平台数字员工同步为 Harness agent preset
fn sync_employees_to_presets(employees: &[EmployeeDef], dsh_entry: &Path, dsh_home: &Path) {
    // 1. 读取 standard preset 模板
    // 2. 注入员工作为人设的 persona 配置
    // 3. 写入 $DSH_HOME/.agent-presets/<preset_id>/
    // 4. 清理已下线员工的 preset
}
```

### 5.6 技能管理系统

```rust
// 平台技能主目录：~/.dsh-platform/skills/<id>/SKILL.md
// 启用的技能自动同步到 Harness 技能目录
pub fn ai_skill_list() -> Vec<SkillMeta>;
pub fn ai_skill_save(id: String, name: String, description: String, content: String, enabled: bool);
pub fn ai_skill_get(id: String) -> Result<String, String>;
pub fn ai_skill_delete(id: String);
```

---

## 六、对你构建 AI 原生应用的参考价值

### 6.1 如果你要做一个"基于 DSH 的 AI 应用"

| 参考点 | 本项目做法 | 可复用模式 |
|--------|----------|-----------|
| **DSH 作为子进程调用** | `Command::spawn()` 启动 headless/web 模式 | 直接复用 `harness.rs` 的封装逻辑 |
| **iframe 嵌入 Harness UI** | Tauri Webview + iframe + 品牌脚本注入 | 复用 `HarnessView.tsx` + `harness_web.rs` |
| **Token 计量与成本控制** | SQLite `ai_calls` 表 + 官方价目表 | 复用 `store.rs` 的计量逻辑 |
| **安全沙箱** | headless 补丁：read-only + approval=never | 直接复用 `HEADLESS_SANDBOX_PATCH` |
| **数字员工/角色系统** | 同步为 Harness agent preset | 复用 `sync_employees_to_presets` |
| **技能管理** | 本地 SKILL.md + 同步到 Harness | 复用 `ai_skill_list/save/get/delete` |
| **知识库互通** | 注册为 Harness 工作区 | 复用 `register_knowledge_workspace` |
| **主题同步** | 写入 `settings.yaml` | 复用 `ensure_dark_theme` |

### 6.2 关键设计决策值得借鉴

1. **"网关层"设计**：所有 AI 调用必须经过统一网关，便于计量、审计、限流、切换引擎
2. **"双引擎"设计**：Harness 用于复杂任务（带工具链），Direct API 用于简单对话（流式体验更好）
3. **DSH_HOME 共享**：与 deepseek-desktop 共用数据目录，用户无需重复配置
4. **应用侧管控**：通过 `--patch` 参数注入配置，禁用外部网络检索，保证 AI 只使用应用内知识
5. **安全优先**：默认 read-only 沙箱，approval=never，内核级拒绝危险命令

---

## 七、文件索引

| 文件 | 职责 |
|------|------|
| `src-tauri/src/harness.rs` | Headless 子进程管理、安全沙箱、超时控制 |
| `src-tauri/src/harness_web.rs` | Web UI 子进程管理、品牌定制、主题同步、知识库注册、员工 preset 同步、技能同步 |
| `src-tauri/src/ai/mod.rs` | AI 网关、SSE 流式、Token 计量、数字员工执行、图表生成 |
| `src-tauri/src/store.rs` | SQLite 数据层：任务、会话、消息、计量、员工 |
| `src/ai/ChatPanel.tsx` | AI 对话面板 UI（Direct 引擎） |
| `src/ai/HarnessView.tsx` | Harness Web UI 容器（iframe） |
| `src/stores/ai.ts` | AI 状态管理：会话、消息、发送、历史加载 |
| `src/lib/bridge.ts` | 前后端桥接：所有 Tauri Command 封装 |
| `README.md` | 项目概述与功能清单 |
| `DESIGN.md` | 完整设计文档（架构、功能、决策） |

---

## 八、总结

DSH Platform 是一个**以 DSH 为智能底座、以 Tauri 为技术栈、以研发工作流为核心的 AI 原生桌面应用**。

它展示了如何将 DeepSeek Harness 从一个独立的 AI 助手工具，转化为**一个更大系统的智能底座**：
- 通过 **headless 子进程**执行分析型任务（代码分析、建模、排查）
- 通过 **Web UI 子进程**提供完整 AI 交互体验（iframe 嵌入）
- 通过 **AI 网关**统一调度、计量、审计所有 AI 调用
- 通过 **preset/skill/knowledge 同步**实现应用生态与 Harness 生态的互通
- 通过 **安全沙箱**确保无人值守场景下的系统安全

对于你要构建的 AI 原生应用，这个项目在**"如何集成 DSH"**、**"如何设计 AI 网关"**、**"如何进行 token 计量"**、**"如何管理数字员工和技能"**等方面，都提供了可直接参考的实现模式。
