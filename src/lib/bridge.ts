/**
 * Tauri IPC 的唯一封装层。
 *
 * 组件里禁止直接 import `@tauri-apps/api`——所有命令与事件都从这里进出，
 * 这样测试可以整体 `vi.mock('./lib/bridge')`（engineering-contract 第 1、8 节）。
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  Actor,
  AiCall,
  AiChatRequest,
  AiChatStart,
  AiUsageRow,
  CanvasDocument,
  CanvasSummary,
  DshWebInfo,
  Employee,
  EmployeeInput,
  ExportFormat,
  HistoryStatus,
  KnowledgeKind,
  KnowledgeDirs,
  KnowledgeEntry,
  PingResponse,
  SkillInfo,
  SkillSyncReport,
  WorkspaceRegistration,
} from "./types";

export type CommandArgs = Record<string, unknown>;

/** 通用 invoke 包装：返回类型由调用方声明，与 Rust 命令签名对齐。 */
export async function invokeCommand<T>(command: string, args?: CommandArgs): Promise<T> {
  return invoke<T>(command, args);
}

/** 探活：后端在线时返回 `{ pong: true, version }`。 */
export async function ping(): Promise<PingResponse> {
  return invokeCommand<PingResponse>("ping");
}

/** 订阅后端事件，返回取消订阅函数。 */
export async function onEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (event) => {
    handler(event.payload);
  });
}

// ---------------------------------------------------------------------------
// 画布
// ---------------------------------------------------------------------------

export function createCanvas(name: string, width: number, height: number) {
  return invokeCommand<CanvasDocument>("create_canvas", { name, width, height });
}

export function getCanvas(canvasId: string) {
  return invokeCommand<CanvasDocument>("get_canvas", { canvasId });
}

export function listCanvases() {
  return invokeCommand<CanvasSummary[]>("list_canvases");
}

export function applyAction(
  canvasId: string,
  action: string,
  input: CommandArgs,
  actor: Actor = "user",
  expectedRevision?: number,
) {
  return invokeCommand<CanvasDocument>("apply_action", {
    canvasId,
    action,
    input,
    actor,
    expectedRevision,
  });
}

export function undo(canvasId: string) {
  return invokeCommand<CanvasDocument>("undo", { canvasId });
}

export function redo(canvasId: string) {
  return invokeCommand<CanvasDocument>("redo", { canvasId });
}

export function historyStatus(canvasId: string) {
  return invokeCommand<HistoryStatus>("history_status", { canvasId });
}

// ---------------------------------------------------------------------------
// 文件
// ---------------------------------------------------------------------------

export function pickSavePath(defaultName: string) {
  return invokeCommand<string | null>("pick_save_path", { defaultName });
}

export function pickOpenPath() {
  return invokeCommand<string | null>("pick_open_path");
}

export function pickExportPath(defaultName: string) {
  return invokeCommand<string | null>("pick_export_path", { defaultName });
}

export function saveDocument(canvasId: string, path: string) {
  return invokeCommand<string>("save_document", { canvasId, path });
}

export function openDocument(path: string) {
  return invokeCommand<CanvasDocument>("open_document", { path });
}

export function exportImage(path: string, format: ExportFormat, dataUrl: string) {
  return invokeCommand<string>("export_image", { path, format, dataUrl });
}

// ---------------------------------------------------------------------------
// AI
// ---------------------------------------------------------------------------

export function aiChatStart(request: AiChatRequest) {
  return invokeCommand<AiChatStart>("ai_chat_start", { request });
}

export function aiCancel() {
  return invokeCommand<void>("ai_cancel");
}

export function aiListCalls(limit = 20) {
  return invokeCommand<AiCall[]>("ai_list_calls", { limit });
}

export function aiUsageSummary(days = 7) {
  return invokeCommand<AiUsageRow[]>("ai_usage_summary", { days });
}

// ---------------------------------------------------------------------------
// 数字员工
// ---------------------------------------------------------------------------

export function listEmployees() {
  return invokeCommand<Employee[]>("list_employees");
}

export function saveEmployee(employee: EmployeeInput) {
  return invokeCommand<Employee>("save_employee", { employee });
}

export function deleteEmployee(id: string) {
  return invokeCommand<void>("delete_employee", { id });
}

export function syncEmployees() {
  return invokeCommand<string[]>("sync_employees");
}

// ---------------------------------------------------------------------------
// 技能
// ---------------------------------------------------------------------------

export function listSkills() {
  return invokeCommand<SkillInfo[]>("list_skills");
}

export function setSkillEnabled(id: string, enabled: boolean) {
  return invokeCommand<void>("set_skill_enabled", { id, enabled });
}

export function syncSkills() {
  return invokeCommand<SkillSyncReport>("sync_skills");
}

export function importDshSkill(id: string) {
  return invokeCommand<SkillInfo>("import_dsh_skill", { id });
}

// ---------------------------------------------------------------------------
// 知识库
// ---------------------------------------------------------------------------

export function knowledgeDirs() {
  return invokeCommand<KnowledgeDirs>("knowledge_dirs");
}

export function listKnowledge(kind: KnowledgeKind) {
  return invokeCommand<KnowledgeEntry[]>("list_knowledge", { kind });
}

export function readKnowledge(kind: KnowledgeKind, name: string) {
  return invokeCommand<KnowledgeEntry>("read_knowledge", { kind, name });
}

export function saveKnowledgeEntry(kind: KnowledgeKind, name: string, content: string) {
  return invokeCommand<KnowledgeEntry>("save_knowledge_entry", { kind, name, content });
}

export function deleteKnowledgeEntry(kind: KnowledgeKind, name: string) {
  return invokeCommand<void>("delete_knowledge_entry", { kind, name });
}

export function registerKnowledgeWorkspace() {
  return invokeCommand<WorkspaceRegistration>("register_knowledge_workspace");
}

// ---------------------------------------------------------------------------
// DSH Web UI
// ---------------------------------------------------------------------------

export function startDshWeb() {
  return invokeCommand<DshWebInfo>("start_dsh_web");
}

export function stopDshWeb() {
  return invokeCommand<void>("stop_dsh_web");
}

export function dshWebStatus() {
  return invokeCommand<DshWebInfo | null>("dsh_web_status");
}

export function openDshWebWindow() {
  return invokeCommand<void>("open_dsh_web_window");
}
