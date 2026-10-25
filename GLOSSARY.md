# PAINT + DSH Glossary

PAINT 是一款面向个人用户的 AI 辅助画图软件，以 DeepSeek Harness（DSH）为智能底座，核心理念是"人机同权"。

## 核心架构

**人机同权（Human-AI Parity）**:
人类用户和 AI Agent 共用同一套命令接口和状态机，通过 ctor = 'user' | 'agent' 区分来源。状态机不区分 actor，只关心操作在当前状态下是否合法。
_Avoid_: 人机协同、AI 协作

**Command Dispatcher**:
PAINT 的统一命令分发器，所有操作（人类或 AI）的唯一入口。接收 pplyAction(action, input, actor)，委托给状态机执行。
_Avoid_: 命令总线、操作分发器

**画布状态机（Canvas State Machine）**:
管理画布生命周期（创建、修改、撤销、重做）的状态机。每次操作后保存完整快照，支持快照级 Undo/Redo。
_Avoid_: 画布引擎、渲染引擎

## AI 生态

**DeepSeek Harness（DSH）**:
DeepSeek 官方推出的桌面 AI 助手运行时，本质为 Node.js CLI 工具。支持 headless（命令行）和 web（本地 Web UI）两种运行模式。
_Avoid_: Harness、deepseek-desktop

**AI 网关（AI Gateway）**:
PAINT 所有 AI 调用的唯一入口。职责：上下文装配、引擎选择（harness/direct）、并发控制（Semaphore，最多 2 个在途请求）、Token 计量、审计追踪。
_Avoid_: AI 代理、AI 中间件

**数字员工（Digital Employee）**:
预配置的 AI 角色 preset，同步为 Harness agent preset。PAINT 内置三类：图形设计师、滤镜专家、画布助手。自定义员工支持名称 + 角色设定 + 技能 + 挂接知识库。
_Avoid_: AI 角色、AI 助手预设

**DSH Skill**:
Harness 技能，通过 SKILL.md 定义。PAINT 的画布操作工具（create_layer、apply_filter 等）封装为 skill，注册到 Harness 技能目录后 AI 可直接调用。
_Avoid_: 插件、扩展

## DSH 集成

**Headless 模式**:
DSH 的命令行模式，通过 
ode <dsh>/lib/bin.js --profile headless ""<任务>"" 启动。适合无人值守的 AI 画图任务，stdout 逐行输出，stderr 截尾报错。
_Avoid_: 后台模式、CLI 模式

**Web UI 模式**:
DSH 的完整 Web 界面模式，通过 
ode <dsh>/lib/bin.js web --host 127.0.0.1 --port <port> 启动。PAINT 通过 iframe 嵌入，获得类 ChatGPT 的完整 AI 交互体验。
_Avoid_: 前端模式、UI 模式

**安全沙箱（Sandbox Patch）**:
通过 --patch 参数注入 DSH 的配置，控制 AI 的执行权限。PAINT 采用 canvas-write 策略：允许画布写入操作，内核级拒绝危险系统命令。
_Avoid_: 权限控制、安全策略

**DSH_HOME**:
DSH 的数据目录（~/.paint/harness/），与 deepseek-desktop 共享。存放会话、配置、插件、技能、agent preset 等。
_Avoid_: 数据目录、配置目录

## 数据层

**快照（Snapshot）**:
画布完整状态的深拷贝，用于 Undo/Redo。每次操作后如满足快照条件则保存，包含操作者、动作类型、时间戳和完整状态。
_Avoid_: 备份、历史记录

**专有格式（.paint）**:
PAINT 的私有文件格式，JSON 序列化，包含完整画布状态（图层、历史、元数据）。工作时保存，导出时生成标准图片。
_Avoid_: 项目文件、源文件

## 通信

**Tauri IPC**:
Tauri 2 的原生进程间通信机制。前端通过 invoke 调用 Rust Command，通过 listen 订阅 Rust 事件。PAINT 统一采用此机制替代 SSE。
_Avoid_: 事件总线、消息通道
