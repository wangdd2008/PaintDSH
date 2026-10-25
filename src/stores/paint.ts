import { create } from "zustand";

import * as bridge from "../lib/bridge";
import type { CanvasDocument, HistoryStatus } from "../lib/types";

/**
 * 全局画布状态（zustand）。
 *
 * 注意：画布文档的真相源在 Rust 侧，这里只放「渲染所需的只读快照」和 UI 状态；
 * 不要在这里维护可编辑的第二份文档（engineering-contract 第 2 节）。
 */

export type BackendState = "connecting" | "connected" | "disconnected";

export interface BackendStatus {
  state: BackendState;
  version: string | null;
}

export type ToolId = "select" | "brush" | "eraser" | "shape" | "text";
export type ShapeKind = "rect" | "ellipse";

export interface PaintState {
  backend: BackendStatus;
  document: CanvasDocument | null;
  history: HistoryStatus | null;
  error: string | null;
  tool: ToolId;
  shapeKind: ShapeKind;
  brushColor: string;
  brushWidth: number;

  setBackendConnected: (version: string) => void;
  setBackendDisconnected: () => void;
  setTool: (tool: ToolId) => void;
  setShapeKind: (kind: ShapeKind) => void;
  setBrushColor: (color: string) => void;
  setBrushWidth: (width: number) => void;
  setError: (message: string | null) => void;
  setDocument: (document: CanvasDocument | null) => void;

  bootstrap: (options?: { width?: number; height?: number; name?: string }) => Promise<void>;
  createCanvas: (name: string, width: number, height: number) => Promise<void>;
  dispatchAction: (action: string, input?: Record<string, unknown>) => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  refreshHistory: () => Promise<void>;
}

export const initialPaintState: Pick<
  PaintState,
  | "backend"
  | "document"
  | "history"
  | "error"
  | "tool"
  | "shapeKind"
  | "brushColor"
  | "brushWidth"
> = {
  backend: { state: "connecting", version: null },
  document: null,
  history: null,
  error: null,
  tool: "select",
  shapeKind: "rect",
  brushColor: "#1c2024",
  brushWidth: 4,
};

/** 把任意错误对象转成可展示的中文消息。 */
export function toErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "message" in error) {
    const payload = error as { kind?: string; message?: unknown };
    const kind = payload.kind ? `${payload.kind}: ` : "";
    return `${kind}${String(payload.message)}`;
  }
  return "未知错误";
}

export const usePaintStore = create<PaintState>((set, get) => ({
  ...initialPaintState,

  setBackendConnected: (version) => set({ backend: { state: "connected", version } }),
  setBackendDisconnected: () => set({ backend: { state: "disconnected", version: null } }),
  setTool: (tool) => set({ tool }),
  setShapeKind: (shapeKind) => set({ shapeKind }),
  setBrushColor: (brushColor) => set({ brushColor }),
  setBrushWidth: (brushWidth) => set({ brushWidth }),
  setError: (error) => set({ error }),
  setDocument: (document) => set({ document }),

  bootstrap: async (options = {}) => {
    try {
      // 先接着上次的作品，避免每次启动都堆一个空画布在库里。
      const existing = await bridge.listCanvases();
      if (existing.length > 0) {
        const document = await bridge.getCanvas(existing[0].id);
        set({ document, error: null });
        await get().refreshHistory();
        return;
      }
    } catch {
      // 列表失败（例如首启动）就继续走新建。
    }

    try {
      const document = await bridge.createCanvas(
        options.name ?? "未命名作品",
        options.width ?? 1280,
        options.height ?? 800,
      );
      set({ document, error: null });
      await get().refreshHistory();
    } catch (error) {
      set({ error: toErrorMessage(error) });
    }
  },

  createCanvas: async (name, width, height) => {
    try {
      const document = await bridge.createCanvas(name, width, height);
      set({ document, error: null });
      await get().refreshHistory();
    } catch (error) {
      set({ error: toErrorMessage(error) });
    }
  },

  dispatchAction: async (action, input = {}) => {
    const { document } = get();
    if (!document) {
      return;
    }
    try {
      const next = await bridge.applyAction(
        document.id,
        action,
        input,
        "user",
        document.revision,
      );
      set({ document: next, error: null });
      await get().refreshHistory();
    } catch (error) {
      set({ error: toErrorMessage(error) });
    }
  },

  undo: async () => {
    const { document } = get();
    if (!document) {
      return;
    }
    try {
      const next = await bridge.undo(document.id);
      set({ document: next, error: null });
      await get().refreshHistory();
    } catch (error) {
      set({ error: toErrorMessage(error) });
    }
  },

  redo: async () => {
    const { document } = get();
    if (!document) {
      return;
    }
    try {
      const next = await bridge.redo(document.id);
      set({ document: next, error: null });
      await get().refreshHistory();
    } catch (error) {
      set({ error: toErrorMessage(error) });
    }
  },

  refreshHistory: async () => {
    const { document } = get();
    if (!document) {
      return;
    }
    try {
      set({ history: await bridge.historyStatus(document.id) });
    } catch {
      // 历史状态只影响按钮态，失败不打断主流程。
    }
  },
}));

/** 当前激活图层（不存在时回落到顶层）。 */
export function activeLayer(document: CanvasDocument | null) {
  if (!document) {
    return null;
  }
  return (
    document.layers.find((layer) => layer.id === document.activeLayerId) ??
    document.layers[document.layers.length - 1] ??
    null
  );
}
