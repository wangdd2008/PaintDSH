# PAINT 工程契约（实现者必读）

本文件锁定跨 ticket 的接口、目录、类型和测试约定。**任何实现者都不得单方面修改本文件的约定**；如确有必要，在 ticket 的 Comments 中说明理由，由集成者统一修订。

规格：`.scratch/paint-dsh-migration/spec.md`。架构：`docs/architecture-design.md`。术语：`GLOSSARY.md`。

---

## 1. 目录结构（固定）

```
package.json                # 前端 + @tauri-apps/cli
vite.config.ts  vitest.config.ts  tsconfig.json  index.html
src/
  main.tsx  App.tsx  app.css
  lib/types.ts              # 与 Rust DTO 一一对应的 TS 类型（手写，无生成器）
  lib/bridge.ts             # invoke/listen 的唯一封装层；组件里禁止直接 import @tauri-apps/api
  lib/errors.ts             # AppError 解析
  stores/paint.ts           # zustand store（画布状态 + 订阅 canvas://changed）
  features/canvas/{CanvasStage.tsx,engine.ts,filters.ts}
  features/toolbox/Toolbox.tsx
  features/layers/LayerPanel.tsx
  features/menu/MenuBar.tsx
  features/ai/{ChatPanel.tsx,HarnessView.tsx,EmployeePanel.tsx}
  features/knowledge/KnowledgePanel.tsx
  features/status/StatusBar.tsx
  test/setup.ts             # vitest setup（jsdom + @testing-library/jest-dom）
src-tauri/
  Cargo.toml  build.rs  tauri.conf.json  capabilities/default.json
  src/main.rs               # 仅调用 paint_dsh_lib::run()
  src/lib.rs                # 模块声明 + run() + 命令注册
  src/error.rs              # AppError / ErrorKind
  src/events.rs             # EventSink trait + CanvasEvent 等事件载荷
  src/paths.rs              # ~/.paint 目录解析（可用 PAINT_HOME 覆盖，测试用）
  src/paint/{mod.rs,document.rs,state_machine.rs,service.rs,command_dispatcher.rs}
  src/store.rs              # SQLite（含 MIGRATIONS）
  src/ai/{mod.rs,gateway.rs,context.rs}
  src/harness.rs            # DSH headless 子进程 + canvas-write patch
  src/harness_web.rs        # DSH web 子进程 + 品牌注入
  src/skills.rs             # DSH Skill 同步与互通
  src/employees.rs          # 数字员工 + preset 同步
  src/knowledge.rs          # 知识库 + 工作区注册
```

Rust crate 名 `paint-dsh`，lib 名 `paint_dsh_lib`（`crate-type = ["staticlib","cdylib","rlib"]`）。

---

## 2. 权威性划分

- **Rust 是画布状态的唯一真相源。** 前端不维护第二份可编辑状态；它把 `CanvasDocument` 渲染成 Fabric 对象，用户交互结束后 `invoke` 一个 action，等 `canvas://changed` 回来再重绘。
- **Fabric.js 只负责渲染与输入采集**，不负责持久化、不做历史管理。
- **所有写操作必须经 `dispatch`（Command Dispatcher）**，人类与 AI 的唯一入口，见第 5 节。

---

## 3. 数据模型（serde `rename_all = "camelCase"`，tagged enum 用 `type`）

```rust
pub enum Actor { User, Agent }                    // "user" | "agent"
pub enum LayerType { Raster, Vector, AiGenerated } // "raster" | "vector" | "aiGenerated"
pub enum BlendMode { Normal, Multiply, Screen, Overlay } // 小写
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LayerContent {
    Raster { data_url: String },   // {"type":"raster","dataUrl":"data:image/png;base64,..."}
    Vector { svg: String },        // {"type":"vector","svg":"<svg .../>"}
    AiCode { code: String },       // {"type":"aiCode","code":"<circle .../>"}
}
pub struct Transform { x: f64, y: f64, scale_x: f64, scale_y: f64, rotation: f64 }
pub struct LayerMetadata { created_by: Actor, created_at: i64, prompt: Option<String>, code: Option<String>, filters: Vec<FilterSpec> }
pub struct Layer {
    id, name, layer_type /* 序列化为 "type" */, visible, locked, opacity: f32,
    blend_mode, transform, content, metadata,
}
pub struct DocumentMetadata { created_at: i64, updated_at: i64, version: u32 }
pub struct CanvasDocument {
    id, name, width: u32, height: u32, background_color: String,
    layers: Vec<Layer>, active_layer_id: String, revision: u64, metadata: DocumentMetadata,
}
pub struct Snapshot { id, revision: u64, action: String, actor: Actor, timestamp: i64, state: CanvasDocument }
pub struct PaintFile { version: u32, canvas: CanvasDocument, history: Vec<Snapshot>, cursor: usize }
```

- `Layer.layer_type` 必须序列化为 `"type"`（`#[serde(rename = "type")]`）。
- 时间戳统一 `i64` 毫秒（Unix epoch ms）。
- 枚举大小写：`Actor` 小写、`BlendMode` 小写、`LayerType` camelCase（`aiGenerated`）。
- 所有面向 JS 的类型都要有 `src/lib/types.ts` 的对应声明，字段名保持 camelCase 一致。

### 滤镜（ticket 11）

```rust
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FilterSpec {
    Blur { radius: f32 },
    Sharpen { amount: f32 },
    AdjustColor { brightness: f32, contrast: f32, saturation: f32, hue: f32 },
}
```

滤镜是**声明式元数据**：`apply_filter` 只把 `FilterSpec` 追加到 `layer.metadata.filters` 并落快照，实际像素效果由前端按 spec 调 Fabric.js filter 重绘；导出时也由前端应用。滤镜只作用于 `raster` 图层，对 `vector`/`aiCode` 图层前端先栅格化（`set_layer_content` 换成 raster）再应用。

---

## 4. Action 词表（固定）

`apply_action` 的 `action` 字符串，一律 snake_case。

| action | input | 说明 |
| --- | --- | --- |
| `create_layer` | `{name?, type?, content?, index?}` | 新建图层并设为 active |
| `delete_layer` | `{layerId}` | 不允许删掉最后一个图层 |
| `reorder_layer` | `{layerId, toIndex}` | |
| `set_layer_visibility` | `{layerId, visible}` | |
| `set_layer_property` | `{layerId, name?, opacity?, blendMode?, locked?}` | |
| `set_layer_transform` | `{layerId, transform}` | |
| `set_layer_content` | `{layerId, content}` | 笔刷/形状/文字提交都走这里 |
| `select_layer` | `{layerId}` | 不落快照 |
| `select_region` | `{region: {x,y,width,height} \| null}` | 不落快照 |
| `apply_filter` | `{layerId, filter}` | ticket 11 |
| `clear_filters` | `{layerId}` | ticket 11 |
| `undo` | `{}` | 移动历史游标，不新增快照 |
| `redo` | `{}` | 同上 |

**快照规则**：除 `select_layer`、`select_region`、`undo`、`redo` 外的 action 都落快照。`undo`/`redo` 恢复历史快照内容但 `revision` 仍然自增（保证乐观锁单调）。

**乐观锁**：`expectedRevision` 为 `Some` 且与当前 `revision` 不符 → `ErrorKind::RevisionConflict`。

---

## 5. Command Dispatcher

```rust
pub fn dispatch(
    store: &Store,
    sink: &dyn EventSink,
    canvas_id: &str,
    request: ActionRequest,   // { action, input: Value, actor, expected_revision: Option<u64> }
) -> Result<CanvasDocument, AppError>;
```

- 状态机不区分 actor，只有合法性检查。
- 成功后 `sink.emit("canvas://changed", document)`。
- 服务层（`service.rs`）负责：读文档 → 乐观锁 → reducer → 落快照 → 持久化 → 发事件。

---

## 6. Tauri IPC 契约（命令 + 事件）

命令（Rust 侧 snake_case 参数，JS 侧用 camelCase 传，Tauri 自动转换）：

```
ping() -> { pong: bool, version: string }
create_canvas(name: string, width: number, height: number) -> CanvasDocument
get_canvas(canvas_id: string) -> CanvasDocument
apply_action(canvas_id, action, input, actor, expected_revision?: number) -> CanvasDocument
undo(canvas_id) / redo(canvas_id) -> CanvasDocument        # 便捷包装，仍走 dispatch

pick_save_path(default_name: string) -> string | null       # rfd 原生对话框
pick_open_path() -> string | null
save_document(canvas_id: string, path: string) -> void      # 写 .paint
open_document(path: string) -> CanvasDocument
export_image(canvas_id: string, path: string, format: "png"|"jpg"|"svg", data_url?: string) -> void

ai_chat_start(request: { canvasId?, employeeId?, prompt, engine?: "harness"|"direct", module? }) -> { sessionId, engine }
ai_cancel(session_id: string) -> void
ai_list_calls(limit: number) -> AiCall[]
ai_usage_summary(days: number) -> { date, calls, tokensInput, tokensOutput, failures }[]

list_employees() -> Employee[]
save_employee(input: EmployeeInput) -> Employee
delete_employee(id: string) -> void
sync_employees() -> string[]

list_skills() -> SkillInfo[]
set_skill_enabled(id: string, enabled: boolean) -> void
sync_skills() -> { synced: string[], removed: string[] }
import_dsh_skill(id: string) -> SkillInfo

knowledge_dirs() -> { root: string, dirs: string[] }
list_knowledge(kind: "brushes"|"palettes"|"shapes"|"design") -> KnowledgeEntry[]
save_knowledge_entry(kind, name: string, content: string) -> KnowledgeEntry
register_knowledge_workspace() -> { workspacePath: string, registered: boolean }

start_dsh_web() -> { url: string, port: number, pid: number }
stop_dsh_web() -> void
dsh_web_status() -> { url, port, pid } | null
```

事件（payload 均为 camelCase）：

| 事件 | payload |
| --- | --- |
| `canvas://changed` | `CanvasDocument` |
| `ai://delta` | `{ sessionId, content, kind: "content" \| "reasoning" }` |
| `ai://done` | `{ sessionId, engine, model, tokensInput, tokensOutput, latencyMs, success }` |
| `ai://error` | `{ sessionId, kind, message }` |
| `dsh://web-status` | `{ running: boolean, url?: string, port?: number }` |

**不要用 `ai://done` 承载错误**；错误单独发 `ai://error`。

---

## 7. 存储

- 连接：`store.rs::Store::open(path)`；测试用 `Store::open_in_memory()` 或临时文件。
- `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;`
- schema 通过 `pub const MIGRATIONS: &[&str]` 顺序执行，全部幂等（`CREATE TABLE IF NOT EXISTS`）。**新 ticket 只能往数组尾部追加一条字符串常量**，不要改动已有条目。
- 表：`canvases`、`layers`、`snapshots`、`ai_sessions`、`ai_messages`、`ai_calls`、`employees`。具体 SQL 由负责该表的 ticket 定义，`canvases`/`layers`/`snapshots` 由 ticket 02 定义。
- 生产路径：`paths::paint_home()/paint.db`；`paint_home()` 默认 `~/.paint`，环境变量 `PAINT_HOME` 可覆盖（测试用）。

---

## 8. 可测试性

- **禁止**在业务逻辑里直接调用 `tauri::AppHandle`。需要发事件的代码依赖 `&dyn EventSink`（`src/events.rs`）：
  ```rust
  pub trait EventSink: Send + Sync {
      fn emit(&self, event: &str, payload: serde_json::Value);
  }
  ```
  `AppHandleSink` 是生产实现；测试用 `RecordingSink`（记录 `(event, payload)`）。
- **禁止**在业务逻辑里直接读进程环境或用户目录；用 `paths::PaintHome`（可注入）。
- 子进程相关逻辑必须能把「可执行文件 + 参数 + 环境变量」当作数据构造出来（`HarnessCommand::build(...)` 返回结构体），测试断言参数内容，不真的启动 DSH。真正启动的函数用这个结构体执行。
- 前端：`lib/bridge.ts` 里所有 `invoke` 都要能被 `vi.mock` 替换；组件测试必须 mock `fabric`（jsdom 没有 canvas 2D）。
- Rust 单测放在各模块内 `#[cfg(test)] mod tests`。

---

## 9. 命令与脚本（固定）

`package.json` scripts：

```
dev        vite
build      tsc --noEmit && vite build
typecheck  tsc --noEmit
test       vitest run
test:watch vitest
tauri      tauri
```

Rust：`cargo test`（在 `src-tauri/`），`cargo fmt --check`，`cargo clippy -- -D warnings`（若太吵可降为默认）。

---

## 10. 依赖基线（固定，勿自行替换）

前端：`react@19`、`react-dom@19`、`fabric@6`、`zustand@5`、`vite@6`、`typescript@5`、`vitest@3`、`@testing-library/react`、`jsdom`、`@vitejs/plugin-react`、`@tauri-apps/api@2`、`@tauri-apps/cli@2`。

Rust：`tauri@2`、`tauri-build@2`、`serde`、`serde_json`、`rusqlite`（`bundled`）、`thiserror@2`、`uuid`（`v4`）、`chrono@0.4`、`rfd@0.15`、`tokio`、`tokio-util`、`futures-util`、`reqwest`（`json,stream,rustls-tls`，`default-features=false`）、`base64@0.22`、`dirs@5`。

新增依赖必须在 ticket Comments 里说明理由。

---

## 11. 本机环境注意事项（Windows）

- cargo 不在 PATH：用 `set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"` 或 `"%USERPROFILE%\.cargo\bin\cargo.exe"`。
- **共享 target 目录**，避免每个 worktree 一份：`set "CARGO_TARGET_DIR=E:\data\.cache\paint-dsh\cargo-target"`。
- npm 装包用 `npm install --cache E:\data\.cache\paint-dsh\npm-cache --prefer-offline`。
- 本机已装真实 `dsh` 0.2.1-rc.3（`%APPDATA%\npm\node_modules\@deepseek-ai\dsh\lib\bin.js`，DSH_HOME=`~/.dsh`）。探测 dsh 时必须重定向输出到文件，禁止运行交互式 `dsh` / `dsh web` / `more`。
- 参考笔记（DSH 实测结论）：`C:\Users\hzb\.qwen\tmp\paint-dsh-notes\dsh-reference-notes.md`。

---

## 12. 分支与集成

- 集成分支：`feat/paint-dsh-migration`（所有 ticket 的基线）。
- 每个 ticket：`ticket/NN-<slug>` 分支 + 独立 worktree（`.qwen/worktrees/NN-<slug>`）。
- 完成前必须把集成分支 tip `git merge` 进自己的分支并解决冲突，再报告完成。
- 提交信息：中文，`feat(ticket-NN): ...` / `fix(ticket-NN): ...` / `test(ticket-NN): ...`。
