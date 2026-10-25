/**
 * 与 Rust DTO 一一对应的手写 TS 类型（无生成器）。
 * 字段名与 `src-tauri/src/paint/document.rs` 等保持 camelCase 一致。
 */

export interface PingResponse {
  pong: boolean;
  version: string;
}

export interface Size {
  width: number;
  height: number;
}

export interface Region extends Size {
  x: number;
  y: number;
}

export type Actor = "user" | "agent";
export type LayerType = "raster" | "vector" | "aiGenerated";
export type BlendMode = "normal" | "multiply" | "screen" | "overlay";

export type LayerContent =
  | { type: "raster"; dataUrl: string }
  | { type: "vector"; svg: string }
  | { type: "aiCode"; code: string };

export interface Transform {
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
  rotation: number;
}

export type FilterSpec =
  | { kind: "blur"; radius: number }
  | { kind: "sharpen"; amount: number }
  | {
      kind: "adjustColor";
      brightness: number;
      contrast: number;
      saturation: number;
      hue: number;
    };

export interface LayerMetadata {
  createdBy: Actor;
  createdAt: number;
  prompt?: string;
  code?: string;
  filters: FilterSpec[];
}

export interface Layer {
  id: string;
  name: string;
  type: LayerType;
  visible: boolean;
  locked: boolean;
  opacity: number;
  blendMode: BlendMode;
  transform: Transform;
  content: LayerContent;
  metadata: LayerMetadata;
}

export interface DocumentMetadata {
  createdAt: number;
  updatedAt: number;
  version: number;
}

export interface CanvasDocument {
  id: string;
  name: string;
  width: number;
  height: number;
  backgroundColor: string;
  layers: Layer[];
  activeLayerId: string;
  selectedRegion?: Region | null;
  revision: number;
  metadata: DocumentMetadata;
}

export interface CanvasSummary {
  id: string;
  name: string;
  width: number;
  height: number;
  revision: number;
  layerCount: number;
  updatedAt: number;
}

export interface HistoryStatus {
  cursor: number;
  total: number;
  canUndo: boolean;
  canRedo: boolean;
}

export type AiEngine = "harness" | "direct";

export interface AiChatRequest {
  canvasId?: string;
  employeeId?: string;
  prompt: string;
  engine?: AiEngine;
  module?: string;
}

export interface AiChatStart {
  sessionId: string;
  engine: AiEngine;
  model: string;
}

export interface AiDelta {
  sessionId: string;
  content: string;
  kind: "content" | "reasoning";
}

export interface AiDone {
  sessionId: string;
  engine: AiEngine;
  model: string;
  tokensInput: number;
  tokensOutput: number;
  latencyMs: number;
  success: boolean;
}

export interface AiError {
  sessionId: string;
  kind: string;
  message: string;
}

export interface AiPendingOp {
  canvasId: string;
  action: string;
  input: Record<string, unknown>;
  reason: string;
}

export interface AiCall {
  id: number;
  sessionId?: string | null;
  canvasId?: string | null;
  module: string;
  employee?: string | null;
  task: string;
  engine: string;
  model: string;
  tokensInput: number;
  tokensOutput: number;
  latencyMs: number;
  success: boolean;
  error?: string | null;
  createdAt: number;
}

export interface AiUsageRow {
  date: string;
  calls: number;
  tokensInput: number;
  tokensOutput: number;
  failures: number;
}

export interface Employee {
  id: string;
  name: string;
  role: string;
  persona: string;
  skills: string[];
  knowledge: string[];
  model?: string | null;
  builtin: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface EmployeeInput {
  id?: string;
  name: string;
  role: string;
  persona: string;
  skills: string[];
  knowledge: string[];
  model?: string | null;
}

export type SkillSource = "paint" | "dsh";

export interface SkillInfo {
  id: string;
  name: string;
  description: string;
  source: SkillSource;
  path: string;
  enabled: boolean;
}

export interface SkillSyncReport {
  written: string[];
  unchanged: string[];
  removed: string[];
}

export type KnowledgeKind = "brushes" | "palettes" | "shapes" | "design";

export interface KnowledgeEntry {
  kind: string;
  name: string;
  path: string;
  bytes: number;
  content?: string;
}

export interface KnowledgeDirs {
  root: string;
  dirs: string[];
}

export interface WorkspaceRegistration {
  workspacePath: string;
  registered: boolean;
  entryCount: number;
}

export interface DshWebInfo {
  url: string;
  port: number;
  pid: number;
}

export interface DshWebStatus {
  running: boolean;
  url?: string;
  port?: number;
}

/** 前端错误对象（Rust `AppError` 的序列化形态）。 */
export interface AppErrorPayload {
  kind: string;
  message: string;
}

/** 导出格式。 */
export type ExportFormat = "png" | "jpg" | "svg";
