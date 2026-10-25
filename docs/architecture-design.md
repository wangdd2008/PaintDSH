# AI 画图软件架构设计文档（基于 DSH 方案）

> 基于 DeepSeek Harness（DSH）智能底座的个人 AI 画图工具
>
> 参考项目：DSH Platform（DevStation）— 面向研发者的智能桌面工作台

---

## 一、项目定位

一款面向个人用户的 AI 辅助画图软件，以 **DeepSeek Harness（DSH）** 为智能底座，融合传统画图工具（笔刷、形状、文字）与 AI 辅助画图（代码生成式 AI 绘图）。

核心设计理念：**"人机同权"**——人类用户和 AI Agent 共用同一套命令接口和状态机。

---

## 二、架构决策汇总

| 编号 | 决策项 | 选择 | 说明 |
|------|--------|------|------|
| Q1 | 桌面壳 | Tauri 2 | 空载 ~95MB，远低于 Electron，与 DSH 生态对齐 |
| Q2 | 后端语言 | Rust | Tauri 2 原生后端，与 DSH headless 子进程管理天然对齐 |
| Q3 | AI 引擎 | DeepSeek Harness（DSH） | 提供 headless + Web UI 双模式，自带工具链 |
| Q4 | AI 交互模式 | 双引擎模式 | Harness（复杂任务）+ Direct API（简单对话） |
| Q5 | 渲染引擎 | Fabric.js | 基于成熟的 Canvas 2D 库，与 AI 引擎解耦 |
| Q6 | 文件格式 | 双格式导出 | 工作时保存专有格式（完整信息），导出时生成标准图片 |
| Q7 | AI 安全策略 | 自定义沙箱 | `canvas-write` 策略：允许画布写入，禁止危险命令 |
| Q8 | 存储策略 | SQLite WAL + 文件 | 复用 DSH Platform 的 `store.rs` 设计 |
| Q9 | 通信机制 | Tauri IPC | 统一 `invoke` + `listen`，替代 SSE |
| Q10 | AI 图层策略 | 强制新建 AI 图层 | 每次 AI 操作新建图层，用户可删除或合并 |
| Q11 | AI 上下文 | 完整 JSON 描述 + DSH 适配 | 画布状态转为 JSON，适配 DSH 工具调用协议 |
| Q12 | UI 布局 | 侧边栏极简布局 | 参考 Figma——左侧工具面板，右侧属性/图层面板，中间大面积画布 |
| Q13 | AI 工具集 | DSH Skill | 画布操作封装为 Harness skill，注册到技能目录 |
| Q14 | 数字员工 | 内置三类角色 | 图形设计师、滤镜专家、画布助手 |

---

## 三、整体架构概览

```
┌─────────────────────────────────────────────────────────────┐
│  用户入口层（两种交互方式）                                    │
│  ┌──────────────┐    ┌──────────────┐                       │
│  │ 传统画图工具  │    │ AI 辅助画图   │                       │
│  │ (笔刷/形状)  │    │ (DSH Agent)  │                       │
│  └──────┬───────┘    └──────┬───────                       │
│         │                   │                               │
│         └─────────┬──────────                               │
│                   ▼                                          │
│         Tauri IPC（统一协议）                                │
└──────────────────┬──────────────────────────────────────────┘
                   │
┌──────────────────┼──────────────────────────────────────────┐
│  Tauri 2 后端（Rust）                                          │
│                  │                                           │
│  ┌───────────────▼─────────────────────────────┐             │
│  │         Tauri Command 路由层               │             │
│  │    invoke(command, payload)                │             │
│  └───────────────┬─────────────────────────────┘             │
│                  │                                           │
│  ┌───────────────▼─────────────────────────────┐             │
│  │      Command Dispatcher                     │             │
│  │  ┌─────────────────────────────────────────┐│             │
│  │  │  dispatch(action, input, actor)         ││             │
│  │  │  actor = 'user' | 'agent'               ││             │
│  │  └─────────────────────────────────────────┘│             │
│  └───────────────┬─────────────────────────────┘             │
│                  │                                           │
│  ┌───────────────▼─────────────────────────────┐             │
│  │       Paint Core（业务核心）               │             │
│  │  ┌───────── ┌─────────┐ ──────────┐     │             │
│  │  │ Service │ │ DSH     │ │Repository│     │             │
│  │  │ 状态机   │ │ Skill   │ │ 数据层    │     │             │
│  │  └─────────┘ └───────── └──────────┘     │             │
│  └─────────────────────────────────────────────┘             │
│                  │                                           │
│  ┌───────────────▼─────────────────────────────┐             │
│  │          AI 网关（AI Gateway）               │             │
│  │  ┌────────── ┌──────────┐ ──────────┐   │             │
│  │  │ Harness  │ │ Direct   │ │ Token    │   │             │
│  │  │ Engine   │ │ Engine   │ │ Meter    │   │             │
│  │  └──────────┘ └──────────┘ └──────────┘   │             │
│  └─────────────────────────────────────────────┘             │
│                  │                                           │
│  ┌───────────────▼─────────────────────────────┐             │
│  │       DSH 集成层（Harness）                  │             │
│  │  ┌────────── ┌────────── ──────────┐   │             │
│  │  │ Headless │ │ Web UI   │ │ Sandbox  │   │             │
│  │  │ Process  │ │ Process  │ │ Patch    │   │             │
│  │  └──────────┘ └──────────┘ └──────────┘   │             │
│  └─────────────────────────────────────────────┘             │
└──────────────────┬──────────────────────────────────────────┘
                   │
┌────────────────────────────────────────────────────────────┐
│  数据层           │                                          │
│  ┌───────────────▼───────────────┐                           │
│  │  SQLite (状态、图层、历史)    │                           │
│  │  文件系统 (作品文件、导出图片) │                           │
│  └───────────────────────────────┘                           │
└──────────────────────────────────────────────────────────────┘
```

---

## 四、核心模块设计

### 4.1 Command Dispatcher（人机同权入口）

```rust
// command_dispatcher.rs
use tauri::Emitter;

pub fn dispatch(action: &str, input: serde_json::Value, actor: &str, app: &AppHandle) -> Result<CanvasState, PaintError> {
    // actor = 'user' → 人类操作（笔刷、点击等）
    // actor = 'agent' → AI 操作（DSH skill 调用）
    
    let state = service::apply_action(action, input, actor)?;
    
    // 通过 Tauri IPC 广播变更
    app.emit("canvas://changed", &state)?;
    
    Ok(state)
}
```

**关键设计**：
- 不区分"人类 API"和"AI API"，只有**一个 `applyAction` 接口**
- 状态机不区分 actor，只关心"这个操作在当前状态下是否合法"
- 每次操作后触发快照保存（用于 Undo）
- 变更通过 Tauri IPC 实时推送到前端

### 4.2 画布状态机（Service）

```rust
// paint_service.rs
#[derive(Debug, Clone)]
pub struct CanvasState {
    pub id: String,
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub layers: Vec<Layer>,
    pub selected_region: Option<Region>,
    pub active_layer_id: String,
    pub history: Vec<Snapshot>,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub layer_type: LayerType, // 'raster' | 'vector' | 'ai-generated'
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub content: LayerContent,
    pub metadata: LayerMetadata,
}

#[derive(Debug, Clone)]
pub struct LayerMetadata {
    pub created_by: Actor, // 'user' | 'agent'
    pub created_at: i64,
    pub prompt: Option<String>, // AI 生成时的提示词
    pub code: Option<String>,   // AI 生成的原始代码
}

pub fn apply_action(canvas_id: &str, request: ActionRequest) -> Result<CanvasState, PaintError> {
    let current = get_canvas(canvas_id)?;
    
    // 乐观锁检查
    if current.revision != request.expected_revision {
        return Err(PaintError::RevisionConflict);
    }
    
    // 执行状态转移
    let next_state = canvas_reducer(current, request)?;
    
    // 保存快照（用于 Undo）
    if is_snapshot_worthy(&request.action) {
        save_snapshot(canvas_id, &next_state)?;
    }
    
    // 记录操作历史
    log_activity(&request.actor, &request.action, next_state.revision)?;
    
    Ok(next_state)
}
```

### 4.3 AI 层：DSH 集成

#### 4.3.1 AI 网关（AI Gateway）

```rust
// ai_gateway.rs
pub async fn ai_chat_start(
    app: &AppHandle,
    request: AiChatRequest,
) -> Result<AiSessionId, AiError> {
    // 1. 上下文装配
    let context = build_context(request.canvas_id)?;
    
    // 2. 引擎选择
    let engine = select_engine(&request);
    
    // 3. 并发控制（Semaphore，最多 2 个在途请求）
    let _permit = AI_SEMAPHORE.acquire().await?;
    
    match engine {
        AiEngine::Harness => {
            // headless 子进程执行
            let outcome = run_headless(&request.task, &context)?;
            // 实时 emit ai://delta 事件到前端
            for line in outcome.stdout.lines() {
                app.emit("ai://delta", AiDeltaEvent { content: line })?;
            }
        }
        AiEngine::Direct => {
            // SSE 流式请求 api.deepseek.com
            let stream = stream_chat(&request, &context).await?;
            for chunk in stream {
                app.emit("ai://delta", AiDeltaEvent { content: chunk.content })?;
            }
        }
    }
    
    // 4. Token 计量入库
    record_ai_call(&request, &outcome)?;
    
    // 5. 完成 emit
    app.emit("ai://done", ())?;
    
    Ok(outcome.session_id)
}
```

#### 4.3.2 DSH Headless 子进程管理

```rust
// harness.rs
pub fn run_headless(
    paths: &HarnessPaths,
    api_key: &str,
    task: &str,
    cwd: Option<&Path>,
    on_line: impl Fn(&str),
) -> Result<HarnessOutcome, HarnessError> {
    // 1. 全局锁：同一时间只跑一个任务
    let _guard = RUN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    
    // 2. 写入 headless 安全补丁（canvas-write 策略）
    let sandbox_patch = write_canvas_sandbox_patch();
    
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

#### 4.3.3 DSH Web UI 子进程管理

```rust
// harness_web.rs
pub fn start_web_ui(paths: &HarnessPaths, port: u16) -> Result<Child, HarnessError> {
    // 1. 启动本地 Web 服务
    let child = Command::new(&paths.node_executable)
        .arg(&paths.dsh_entry)
        .arg("web")
        .arg("--host").arg("127.0.0.1")
        .arg("--port").arg(port.to_string())
        .env("DSH_HOME", &paths.dsh_home)
        .spawn()?;
    
    // 2. 应用侧集成
    //    - 深色主题同步
    //    - 知识库工作区注册
    //    - 数字员工 preset 同步
    //    - 技能同步
    //    - 管控补丁（禁用外部网络检索）
    
    // 3. 品牌定制脚本注入
    //    - 替换 DeepSeek 品牌为「AI 助手」
    //    - 深色主题色阶映射
    
    Ok(child)
}
```

### 4.4 DSH Skill 设计

AI 通过 DSH Skill 操作画布。每个 skill 为独立的 `SKILL.md` 文件：

```
~/.paint/skills/
├── canvas-get-context/
│   └── SKILL.md
├── canvas-create-layer/
│   └── SKILL.md
├── canvas-apply-filter/
│   └── SKILL.md
├── canvas-select-region/
│   └── SKILL.md
├── canvas-draw-shape/
│   └── SKILL.md
├── canvas-import-image/
│   └── SKILL.md
└── canvas-export-image/
    └── SKILL.md
```

**Skill 示例**（`canvas-create-layer/SKILL.md`）：

```markdown
# canvas-create-layer

在指定画布上创建一个新图层。

## Parameters

- `canvasId`: string — 画布 ID
- `name`: string — 图层名称
- `content`: SVGString | null — 初始内容（可选）

## Example

```json
{
  "canvasId": "canvas-123",
  "name": "AI-红色圆",
  "content": "<circle cx=\"100\" cy=\"100\" r=\"50\" fill=\"red\" />"
}
```
```

启用后，skill 自动同步到 Harness 技能目录，AI 可直接调用。

### 4.5 AI 代码生成模式

由于选择**纯 DSH 代码生成**模式，AI 生成内容的方式是：

```
1. 用户说："帮我画一个红色的圆"
2. DSH 推理后生成 Canvas/SVG 代码：
   const svg = `<circle cx="100" cy="100" r="50" fill="red" />`;
3. DSH 调用 canvas-create-layer skill：
   { action: "canvas-create-layer", name: "AI-红色圆", content: svg }
4. 系统创建新图层，将 SVG 渲染到画布上
5. 用户可以在画布上继续编辑这个图层
```

**优势**：
- AI 生成的是可编辑的矢量代码，而非静态像素图
- 用户可以进一步修改 AI 生成的图形（改颜色、改大小、改形状）
- 不需要调用外部图像生成 API，完全离线可用

**局限**：
- 不适合照片级真实感图像
- 复杂场景需要多步代码生成

---

## 五、前端架构

### 5.1 UI 布局（侧边栏极简布局）

```
┌─────────────────────────────────────────────────────────────┐
│  [菜单栏: 文件 编辑 视图 帮助]                                │
├──────────┬────────────────────────────────────┬─────────────┤
│          │                                    │             │
│  工具面板 │         画 布 区 域               │ 属性/图层   │
│  ─────── │                                    │ 面板        │
│  [选择]  │    ┌──────────────────────┐         │ ─────────── │
│  [画笔]  │    │                      │         │ 图层列表    │
│  [橡皮]  │    │    画布内容          │         │ - Layer 1   │
│  [形状]  │    │                      │         │ - Layer 2 ⬅ │
│  [文字]  │    │                      │         │ - AI-圆    │
│  [填充]  │    │                      │         │             │
│  ─────── │    └──────────────────────┘         │ 属性面板    │
│          │                                    │ - 颜色      │
│          │                                    │ - 透明度    │
│          │                                    │ - 混合模式  │
├──────────────────────────────────────────────┴─────────────┤
│  [状态栏: 画布尺寸 缩放比例 当前工具]                         │
└─────────────────────────────────────────────────────────────┘
```

**AI 对话面板**：右侧属性面板下方可折叠抽屉，默认收起，点击 AI 图标展开。

**DSH Web UI 嵌入**：通过 iframe 嵌入 Harness 完整界面，提供类 ChatGPT 的 AI 交互体验。

### 5.2 画布渲染（基于 Fabric.js）

```typescript
// canvas-engine.ts
import { Canvas } from 'fabric';

class PaintCanvasEngine {
  private canvas: Canvas;
  private layers: Map<string, fabric.Object>;
  
  constructor(element: HTMLCanvasElement) {
    this.canvas = new Canvas(element);
  }
  
  // 添加图层
  addLayer(layer: Layer): fabric.Object {
    const fabricObject = this.createFabricObject(layer);
    this.canvas.add(fabricObject);
    this.layers.set(layer.id, fabricObject);
    return fabricObject;
  }
  
  // 应用滤镜
  applyFilter(layerId: string, filter: FilterConfig) {
    const obj = this.layers.get(layerId);
    // 应用 Fabric.js 滤镜
  }
  
  // 导出图片
  export(format: 'png' | 'jpg'): Blob {
    return this.canvas.toDataURL({ format });
  }
}
```

### 5.3 状态同步（Tauri IPC）

```typescript
// paint-ipc.ts
import { invoke, listen } from '@tauri-apps/api/core';

// 订阅画布变更
export function subscribeCanvasChanges(canvasId: string, callback: (state: CanvasState) => void) {
  const unlisten = listen<CanvasState>('canvas://changed', (event) => {
    if (event.payload.id === canvasId) {
      callback(event.payload);
    }
  });
  return unlisten;
}

// 订阅 AI 输出
export function subscribeAiOutput(callback: (delta: AiDeltaEvent) => void) {
  const unlisten = listen<AiDeltaEvent>('ai://delta', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

// 调用后端 Command
export async function applyAction(action: string, input: unknown, actor: 'user' | 'agent') {
  return invoke<CanvasState>('apply_action', { action, input, actor });
}
```

---

## 六、数据模型

### 6.1 核心实体

```rust
// 画布（单文件模式）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanvasDocument {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub layers: Vec<Layer>,
    pub active_layer_id: String,
    pub history: Vec<Snapshot>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentMetadata {
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u32,
}

// 图层
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub layer_type: LayerType, // 'raster' | 'vector' | 'ai-generated'
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub transform: Transform,
    pub content: LayerContent,
    pub metadata: LayerMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LayerMetadata {
    pub created_by: Actor, // 'user' | 'agent'
    pub created_at: i64,
    pub prompt: Option<String>,
    pub code: Option<String>, // AI 生成的原始代码
}

// 图层内容（支持多种格式）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum LayerContent {
    Raster { data_url: String }, // base64 图片
    Vector { svg: String },      // SVG 代码
    AiCode { code: String },     // AI 生成的 Canvas/SVG 代码
}

// 快照（用于 Undo/Redo）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub revision: u64,
    pub action: String,
    pub actor: Actor,
    pub timestamp: i64,
    pub state: CanvasDocument, // 完整状态深拷贝
}
```

### 6.2 文件格式

**专有格式（`.paint`）**：

```json
{
  "version": 1,
  "canvas": {
    "id": "canvas-123",
    "name": "未命名作品",
    "width": 1920,
    "height": 1080,
    "backgroundColor": "#ffffff",
    "layers": [
      {
        "id": "layer-1",
        "name": "背景",
        "type": "raster",
        "visible": true,
        "opacity": 1.0,
        "content": { "type": "raster", "dataUrl": "data:image/png;base64,..." }
      },
      {
        "id": "layer-ai-1",
        "name": "AI-红色圆",
        "type": "ai-generated",
        "visible": true,
        "opacity": 1.0,
        "content": { "type": "ai-code", "code": "<circle cx=\"100\" cy=\"100\" r=\"50\" fill=\"red\" />" },
        "metadata": {
          "createdBy": "agent",
          "createdAt": 1698765432000,
          "prompt": "画一个红色的圆"
        }
      }
    ]
  },
  "history": [ /* 快照数组 */ ]
}
```

**导出格式**：
- `PNG`（透明背景）
- `JPG`（白色背景）
- `SVG`（矢量图层保留可编辑性）

---

## 七、DSH 集成详细设计

### 7.1 AI 操作流程

```
用户输入自然语言指令
         │
         ▼
┌─────────────────────┐
│  Context Builder    │  ← 构建画布上下文（图层列表、尺寸、选中区域）
└──────────┬──────────
           │
           ▼
┌─────────────────────┐
│  AI 网关            │  ← 选择引擎（harness/direct）
└──────────┬──────────┘
           │
     ┌─────┴─────┐
     ▼           ▼
┌────────┐  ┌──────────┐
│Harness │  │ Direct   │
│Engine  │  │ Engine   │
└───┬────┘  └────┬─────┘
     │            │
     ▼            ▼
┌────────┐  ┌──────────┐
│Headless│  │ SSE 流式 │
│子进程  │  │请求      │
└───┬────┘  └────┬─────┘
     │            │
     └─────┬──────┘
           ▼
┌─────────────────────┐
│  Tauri IPC Event    │  ← ai://delta / ai://done
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Command Dispatcher │  ← 与人类操作走同一入口
│  applyAction()      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Service 状态机     │  ← 执行状态转移，保存快照
└──────────┬──────────
           │
           ▼
┌─────────────────────┐
│  Tauri IPC Event    │  ← canvas://changed
└─────────────────────┘
```

### 7.2 数字员工系统

PAINT 内置三类数字员工，同步为 Harness agent preset：

| 员工 | 角色设定 | 技能 | 知识库 |
|------|---------|------|--------|
| **图形设计师** | 专注于图形生成、布局建议 | create_layer, draw_shape, select_region | 画笔模板、配色方案 |
| **滤镜专家** | 专注于图像处理、滤镜参数调优 | apply_filter, adjust_color | 滤镜预设、调色板 |
| **画布助手** | 专注于图层管理、组织优化 | create_layer, delete_layer, merge_layer | 图层命名规范 |

```rust
// 将数字员工同步为 Harness agent preset
fn sync_employees_to_presets(employees: &[EmployeeDef], dsh_entry: &Path, dsh_home: &Path) {
    // 1. 读取 standard preset 模板
    // 2. 注入员工作为人设的 persona 配置
    // 3. 写入 $DSH_HOME/.agent-presets/<preset_id>/
    // 4. 清理已下线员工的 preset
}
```

### 7.3 知识库互通

PAINT 知识库路径：`~/.paint/knowledge/`

```
~/.paint/
├── knowledge/           # 知识库（注册为 Harness 工作区）
│   ├── brushes/          # 画笔模板
│   ├── palettes/         # 配色方案
│   ├── shapes/           # 形状模板
│   └── design-guide.md   # 设计规范
├── harness/             # DSH_HOME（与 deepseek-desktop 共享）
│   ├── settings.yaml
│   ├── storages/
│   ├── skills/           # 同步的技能
│   └── .agent-presets/   # 同步的数字员工 preset
└── skills/              # 平台技能主目录
    └── canvas-*/
        └── SKILL.md
```

### 7.4 Token 计量与成本控制

所有 AI 调用统一经 AI 网关计量，落 SQLite `ai_calls` 表：

```rust
// store.rs
pub struct AiCall {
    pub id: i64,
    pub module: String,      // "paint" | "chat" | "employee"
    pub employee: String,    // 数字员工名称（如果有）
    pub task: String,        // 任务描述
    pub engine: String,      // "harness" | "direct"
    pub model: String,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub latency_ms: i64,
    pub success: bool,
    pub created_at: i64,
}
```

---

## 八、项目目录结构

基于 Tauri 2 的标准结构：

```
paint/
├── src/                          # 前端 React 19 + TS + Vite
│   ├── features/
│   │   ├── canvas/               # 画布组件（Fabric.js）
│   │   ├── toolbox/              # 工具面板
│   │   ├── layer-panel/          # 图层面板
│   │   └── ai-chat/              # AI 对话面板
│   ├── ai/
│   │   ├── ChatPanel.tsx         # Direct 引擎对话 UI
│   │   └── HarnessView.tsx       # DSH Web UI 容器（iframe）
│   ├── stores/
│   │   └── paint.ts              # zustand 状态管理
│   ├── lib/
│   │   └── bridge.ts             # Tauri Command 封装
│   └── main.tsx
│
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── paint/                # 画图核心
│   │   │   ├── mod.rs
│   │   │   ├── command_dispatcher.rs
│   │   │   ├── service.rs
│   │   │   └── state_machine.rs
│   │   ├── ai/                   # AI 网关 + 数字员工
│   │   │   ├── mod.rs
│   │   │   ├── gateway.rs
│   │   │   └── employee.rs
│   │   ├── harness.rs            # DSH headless 子进程
│   │   ├── harness_web.rs       # DSH Web UI 子进程
│   │   ├── store.rs             # SQLite 数据层
│   │   └── lib.rs
│   └── Cargo.toml
│
├── docs/
│   ├── adr/                      # 架构决策记录
│   │   ├── 0001-tauri-2-migration.md
│   │   ├── 0002-dsh-replacement.md
│   │   ├── 0003-canvas-sandbox.md
│   │   └── 0004-unified-tauri-ipc.md
│   └── architecture-design.md    # 本设计文档
│
├── GLOSSARY.md                   # 术语表
├── tauri.conf.json
├── package.json
└── Cargo.toml
```

---

## 九、技术栈

| 层级 | 技术 |
|------|------|
| **前端** | React 19 + TypeScript + Vite + Fabric.js |
| **桌面壳** | Tauri 2（系统 WKWebView） |
| **后端** | Rust + Tauri 2 |
| **AI 引擎** | DeepSeek Harness（DSH） |
| **AI 网关** | Rust（统一调度、Token 计量、审计追踪） |
| **终端** | portable-pty + xterm.js（可选） |
| **数据库** | SQLite WAL |
| **构建** | pnpm + TypeScript + Rust Cargo |
| **测试** | Vitest + Playwright |

---

## 十、实现路线图

### Phase 1：基础画布（MVP）
- [ ] 初始化 Tauri 2 + React 19 项目结构
- [ ] 集成 Fabric.js，实现基础画布渲染
- [ ] 实现图层管理（创建、删除、排序、可见性）
- [ ] 实现基础工具（选择、画笔、橡皮擦、形状、文字）
- [ ] 实现文件保存/打开（专有格式 `.paint`）
- [ ] 实现导出功能（PNG/JPG/SVG）

### Phase 2：DSH 集成
- [ ] 集成 DSH headless 子进程（`harness.rs`）
- [ ] 实现 AI 网关（统一调度、Token 计量）
- [ ] 实现 Context Builder（画布状态 → JSON → DSH 适配）
- [ ] 实现 DSH Skill 封装（画布操作工具）
- [ ] 实现自定义安全沙箱（`canvas-write` 策略）
- [ ] 实现智能确认机制

### Phase 3：AI 生态
- [ ] 实现数字员工系统（图形设计师、滤镜专家、画布助手）
- [ ] 实现知识库（注册为 DSH 工作区）
- [ ] 实现技能生态互通（PAINT 技能 ↔ DSH 技能）
- [ ] 集成 DSH Web UI（iframe 嵌入）
- [ ] 实现品牌定制和主题同步

### Phase 4：高级功能
- [ ] 实现快照级 Undo/Redo
- [ ] 实现滤镜系统（模糊、锐化、调色等）
- [ ] 实现 AI 图层管理（合并、拆分、编辑 AI 代码）
- [ ] 性能优化（大画布、多图层）
- [ ] 插件系统（自定义工具、滤镜）

---

## 十一、与 DSH Platform 的对比

| 特性 | DSH Platform | PAINT |
|------|-------------|-------|
| **核心功能** | 研发工作台（IDE、终端、Git 等） | 传统画图 + AI 辅助画图 |
| **AI 角色** | 代码分析员、问题排查员等 | 图形设计师、滤镜专家、画布助手 |
| **AI 生成方式** | 代码分析、文档生成、建模 | Canvas/SVG 代码生成 |
| **渲染引擎** | CSS/SVG 自绘（无动画） | Fabric.js（Canvas 2D） |
| **数据模型** | 任务、会话、建模文档 | 画布、图层、快照 |
| **状态同步** | Tauri IPC | Tauri IPC |
| **架构模式** | 人机同权 Command Dispatcher | 人机同权 Command Dispatcher |
| **DSH 集成** | headless + Web UI 双模式 | headless + Web UI 双模式 |
| **安全策略** | read-only + approval=never | canvas-write（自定义） |

---

## 十二、一句话总结

> **这是一款基于 DeepSeek Harness（DSH）智能底座的 AI 辅助画图软件：DSH 作为贯穿全部模块的智能底座，AI 通过 DSH Skill 调用操作画布图层，生成可编辑的 Canvas/SVG 代码，与人类用户共用同一套 Command Dispatcher 和状态机。**
