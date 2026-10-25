# DSH Platform（DevStation）— M1 智能研发桌面工作台 · 完整设计文档

> 版本 v2.0 · 2026-08-20
> 目标机型：MacBook Pro M1（Apple Silicon，8 核 CPU = 4P + 4E，16GB 统一内存）
> 核心理念：**桌面即应用（Desktop as the Application）**
> 设计语言：macOS 深色玻璃拟态、红绿灯标题栏、可收展侧栏、⌘K 命令面板

---

## 1. 产品定位与设计理念

### 1.1 一句话定位

一台**低功耗、可整日常驻**的研发驾驶舱：把开发者桌面上散落的工具（IDE、终端、Postman、Git 客户端、笔记、监控仪表、AI 聊天页）折叠进一个原生质感的工作台，以 **DeepSeek Harness（deepseek-v4-flash）** 作为贯穿全部模块的智能底座。

### 1.2 "桌面即应用"的三层含义

| 层 | 含义 | 落地表现 |
|---|---|---|
| 空间层 | 工作台即桌面中心 | 单窗口多模块 + 可展开侧栏 + ⌘K 全局唤起 |
| 工作流层 | 研发全流程不出工作台 | 代码 ↔ 终端 ↔ Git ↔ API ↔ 知识库 ↔ AI 以"上下文"互传 |
| 智能层 | AI 不是附件而是操作系统级服务 | 所有模块共享 AI 引擎，命令面板里 AI 指令与普通命令同级 |

### 1.3 设计原则

1. **低功耗优先**：自适应采样（1s→5s→30s），空闲自动降频
2. **本地优先**：数据全部本机，AI 走云端
3. **零配置**：开箱即用，内置凭据，无需任何密钥管理
4. **渐进智能**：AI 一律"建议→确认→执行"

---

## 2. 技术架构

### 2.1 技术栈

| 层 | 选型 | 理由 |
|---|---|---|
| 壳 | Tauri 2（系统 WKWebView） | 空载 ~100MB，远低于 Electron |
| 前端 | React 19 + TS + Vite | 组件化、HMR |
| AI 引擎 | DeepSeek Harness（dsh） | headless 子进程，自带工具 |
| 终端 | portable-pty + xterm 6 | 多标签本地终端 |
| 编辑器 | CodeMirror 6 | 多语言高亮 |
| 图表 | CSS/SVG 自绘 | 无动画、省电 |
| 存储 | SQLite WAL + 文件 | 本地优先 |
| 文件比对 | similar crate（行+词级 diff） | Beyond Compare 体验 |

### 2.2 模块总览（13 个导航 + 右栏双面板）

```
┌─ 标题栏（⌘K 搜索 + 项目/分支 chip）─────────────────────────────┐
├──┬──────────────────────────────────────────────┬──────────────┤
│导│  中央模块区（按 activeNav 路由）                 │ 右侧面板 320px │
│航│  工作台 · AI助手 · 数字员工 · 监控              │ [AI] ⇄ [监控] │
│可│  代码 · 终端 · 建模 · 工具箱                    │   可折叠      │
│收│  服务 · 项目(Git) · API调试 · 知识库 · 设置      │              │
│展│                                                │              │
├──┴──────────────────────────────────────────────┴──────────────┤
│ 状态栏：CPU/MEM/DISK/NET · Git 分支 · 电池/低功耗 · 时钟        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 全模块功能清单

### 3.1 工作台（Dashboard）
- 时段问候 + 系统状态徽标
- 4 张指标卡（CPU P/E 核 / 内存 / 磁盘 / 网络），点击跳转监控下钻
- 今日任务 CRUD（SQLite 持久化，优先级标色）
- 快捷入口宫格 + AI 快捷动作 4 宫格

### 3.2 AI 助手
- **双引擎**：DeepSeek Harness（默认，headless 子进程）/ 直连 API（SSE 逐字流式）
- 流式渲染 + Markdown（代码块高亮/复制 + 表格）
- 思考过程折叠（reasoning_content）
- 6 个快捷指令（代码解释/Bug修复/性能优化/单元测试/生成文档/代码重构）
- 内置凭据，零配置

### 3.3 数字员工
- **内置五岗**：代码分析员 / 问题排查员 / 接口文档整理员 / 业务流程梳理员 / 表结构梳理员
- **自定义员工**：名称 + 角色设定 + 技能（逗号分隔）+ 挂接知识库笔记（上限 16KB）
- 业务流程/表结构两类产出可一键**转为建模中心的流程图 / ER 图**
- 会话历史持久化，按员工独立会话

### 3.4 监控
- **概览**：指标卡点击下钻 → CPU 各核柱状 / 内存 Top 进程 / 网络分接口 / 磁盘分卷
- **进程与端口**：
  - 按端口查进程（lsof）→ 一键结束
  - 进程树（ppid 构树，高 CPU/内存橙色高亮）
  - **启动链路追踪**：选中进程显示从根进程到它的完整祖先链 + 每级启动命令
- 自适应采样频率：可见 1s → 空闲 5s → 后台 30s

### 3.5 代码编辑器
- 项目文件树（跳过 .git/node_modules/target 等）
- CodeMirror 6 多标签编辑：TS/JS/JSON/MD/Python/Rust/SQL/Java/CSS/HTML 高亮
- ⌘S 保存，未保存标签带 ● 标记，新建/删除文件
- 路径访问限制在项目根内

### 3.6 终端
- 本地多标签真终端（portable-pty + xterm.js）
- 默认在项目目录启动 zsh，256 色、滚动回溯
- 关闭标签回收 shell，应用退出全部回收

### 3.7 建模中心
- **数据建模**：概念图（实体+关系基数）/ 逻辑图（属性+PK/FK）/ 物理图（物理类型），单一数据源三层视图
- **物理图 → 生成 DDL**（MySQL/PostgreSQL 切换，含外键约束）
- **业务建模**：业务流程图（开始/活动/判断/结束）/ 接口拓扑图（产品/网关/微服务/DB/MQ + 调用方式标注）
- **AI 生成**：五类画布输入需求描述一键生成，自动分层布局
- 多文档管理，800ms 防抖自动保存 SQLite

### 3.8 工具箱
- **JSON 工具**：格式化/压缩/校错 + 可折叠结构树 + 字段统计（Top 40 出现次数与类型分布）
- **Token 用量统计**：按天/周/月趋势柱状图 + 业务场景分类（AI对话/图表/数字员工）+ 按员工分类 + 最耗 token 任务 Top 30
- **文件比对**：Beyond Compare 式左右分栏 + 行级 diff（修改行配对）+ 词级高亮 + 统计
- **内存分析**（hprof 堆转储）：
  - 流式解析（811MB / 19s），实时进度条动画
  - 诊断发现：启发式规则（byte[] 占比/字符串/集合膨胀/线程过多/超大对象）→ 中文定位建议
  - 类别画像 + 业务包画像（自动推断技术栈/模块）
  - **GC 引用链**：大对象被谁持有（到业务类/GC 根），含字段名
  - 最大单体对象 Top 20 + 分配热点（代码行级）
  - **一键生成代码定位报告**：堆现象 → 检索项目源码 → 结构化 Markdown 报告（文件：行号）→ 存知识库

### 3.9 微服务运行
- JDK/Maven 环境配置（JAVA_HOME 注入，mvnw 优先）
- 项目列表持久化，多项目并行启动（独立进程组）
- 实时日志分页查看，停止时 TERM→KILL 整组回收

### 3.10 项目与 Git
- 分支 chip（↑领先 ↓落后）+ 提交说明 + 提交/暂存全部/拉取/推送
- 已暂存/改动文件列表（M/A/D/R 状态标色），逐文件暂存/取消
- Diff 着色渲染 + 提交历史

### 3.11 API 调试
- 7 种方法（各配色）+ URL + 回车即发
- 请求头（可增删启停 k/v 行）、请求体（JSON 一键格式化）
- 响应：状态徽章、耗时/大小、JSON 自动美化、原始/美化切换
- 会话历史

### 3.12 知识库
- Markdown 笔记：文件列表 + 全文搜索（命中行定位跳转）
- 编辑（⌘S）/ 渲染预览切换（表格支持）
- 新建/删除，路径限制在库内

### 3.13 设置
- 通用：项目目录 / 低功耗模式
- AI 助手：引擎选择（Harness/直连）/ Harness 目录 / Token 预算
- 用量：今日/本月/累计 tokens + 预算进度条

---

## 4. AI 子系统

### 4.1 架构

```
用户输入 → AI 网关（Rust）
           ├─ engine = harness → DeepSeek Harness headless 子进程
           │    └─ 自带 bash/fs/editor 工具 + 会话管理
           ├─ engine = direct  → reqwest SSE → api.deepseek.com
           │    └─ 逐字流式 + reasoning_content 分离
           └─ 内置凭据（零配置，无密钥管理 UI）
```

### 4.2 Token 计量

所有 AI 调用统一经网关计量（actor = 员工名/入口，action = 任务描述），落 SQLite `ai_calls` 表，工具箱 → 用量统计 可查看。

---

## 5. 低功耗设计（§7 量化目标已达标）

| 指标 | 目标 | 实测 |
|---|---|---|
| 空载内存 | ≤ 150MB | **~95MB** ✓ |
| 空载 CPU | ≤ 1.5% | **0.3%** ✓ |
| 冷启动 | ≤ 1.5s | **~2s**（含 dev 编译） |
| 风扇 | 常规 0 转 | ✓ |

核心机制：PowerState 状态机（Active 1s → Idle 5s → Background 30s），Rust 是唯一采样源，前端零 setInterval。

---

## 6. 关键技术决策记录

| 决策 | 理由 | ADR |
|---|---|---|
| Tauri 2 而非 Swift | 已有 workbench 资产复用，功耗差距可接受 | ADR-001 |
| 零密钥管理 | 个人工具，内置凭据即用 | — |
| hprof 锚定扫描 | 兼容 1.0.1 变体格式，自愈容错 | — |
| 引用链替代支配树 | 实现简洁，满足"谁持有"定位需求 | — |
| headless 默认 read-only 沙箱 | 五岗均为分析型任务无需写入；写入类风险（rm/mkfs/dd）由内核级沙箱结构性拒绝，优于 Prompt 软约束；无人值守 approval=never 避免 fail-closed 卡死 | — |

---

## 7. 目录结构

```
dsh-platform/
├─ design/               # 设计稿（React 原型，视觉规范）
├─ docs/adr-001-shell-choice.md
├─ src/                  # 前端 React 19 + TS
│  ├─ shell/             # TitleBar / NavRail / StatusBar / RightPanel
│  ├─ modules/           # Dashboard / ChatPanel / Employees / Monitor /
│  │                     # Editor / Terminal / Modeling / Tools / Services /
│  │                     # ProjectGit / ApiDebug / Knowledge / Settings
│  ├─ ai/                # ChatPanel（AI 对话面板）
│  ├─ components/        # CommandPalette / Monitor / ProcessPanel / modeling/
│  ├─ stores/            # zustand（app / ai）
│  ├─ lib/               # bridge / layout / ddl / markdown
│  └─ styles.css
├─ src-tauri/
│  ├─ src/
│  │  ├── lib.rs         # 入口 + 任务/Git/知识库/员工命令
│  │  ├── ai/mod.rs      # AI 网关 + 数字员工 + 代码定位报告
│  │  ├── harness.rs     # DeepSeek Harness 引擎
│  │  ├── hprof.rs       # 堆转储解析（锚定扫描 + 引用链）
│  │  ├── terminal.rs    # PTY 终端
│  │  ├── runner.rs      # 微服务运行器
│  │  ├── gitx.rs        # Git 可视化
│  │  ├── httpx.rs       # HTTP 客户端
│  │  ├── fsx.rs         # 文件树
│  │  ├── diffx.rs       # 文件比对
│  │  ├── procs.rs       # 进程/端口
│  │  ├── sampler.rs     # 自适应采样
│  │  ├── power.rs       # PowerState 状态机
│  │  ├── config.rs      # 配置
│  │  └── store.rs       # SQLite
│  └─ tauri.conf.json
├─ DESIGN.md             # 本文件
└─ README.md
```
