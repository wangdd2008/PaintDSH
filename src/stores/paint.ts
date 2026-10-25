import { create } from "zustand";

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

export interface PaintState {
  backend: BackendStatus;
  setBackendConnected: (version: string) => void;
  setBackendDisconnected: () => void;
}

export const initialPaintState: Pick<PaintState, "backend"> = {
  backend: { state: "connecting", version: null },
};

export const usePaintStore = create<PaintState>((set) => ({
  ...initialPaintState,
  setBackendConnected: (version) => set({ backend: { state: "connected", version } }),
  setBackendDisconnected: () => set({ backend: { state: "disconnected", version: null } }),
}));
