import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../lib/bridge", () => ({
  createCanvas: vi.fn(),
  applyAction: vi.fn(),
  undo: vi.fn(),
  redo: vi.fn(),
  historyStatus: vi.fn(),
}));

import * as bridge from "../lib/bridge";
import type { CanvasDocument } from "../lib/types";
import { activeLayer, initialPaintState, toErrorMessage, usePaintStore } from "./paint";

function document(): CanvasDocument {
  return {
    id: "canvas-1",
    name: "测试",
    width: 800,
    height: 600,
    backgroundColor: "#ffffff",
    layers: [
      {
        id: "layer-1",
        name: "背景",
        type: "vector",
        visible: true,
        locked: false,
        opacity: 1,
        blendMode: "normal",
        transform: { x: 0, y: 0, scaleX: 1, scaleY: 1, rotation: 0 },
        content: { type: "vector", svg: "<svg></svg>" },
        metadata: { createdBy: "user", createdAt: 1, filters: [] },
      },
    ],
    activeLayerId: "layer-1",
    revision: 0,
    metadata: { createdAt: 1, updatedAt: 1, version: 1 },
  };
}

describe("paint store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    usePaintStore.setState({ ...initialPaintState });
  });

  it("初始状态没有文档、工具为选择", () => {
    const state = usePaintStore.getState();
    expect(state.document).toBeNull();
    expect(state.tool).toBe("select");
    expect(state.backend.state).toBe("connecting");
  });

  it("bootstrap 创建画布并刷新历史", async () => {
    const doc = document();
    vi.mocked(bridge.createCanvas).mockResolvedValue(doc);
    vi.mocked(bridge.historyStatus).mockResolvedValue({
      cursor: 0,
      total: 1,
      canUndo: false,
      canRedo: false,
    });

    await usePaintStore.getState().bootstrap({ name: "草稿", width: 800, height: 600 });

    expect(bridge.createCanvas).toHaveBeenCalledWith("草稿", 800, 600);
    expect(usePaintStore.getState().document?.id).toBe("canvas-1");
    expect(usePaintStore.getState().history?.total).toBe(1);
  });

  it("dispatchAction 带上 revision 做乐观锁", async () => {
    const doc = document();
    usePaintStore.setState({ document: doc });
    vi.mocked(bridge.applyAction).mockResolvedValue({ ...doc, revision: 1 });
    vi.mocked(bridge.historyStatus).mockResolvedValue({
      cursor: 1,
      total: 2,
      canUndo: true,
      canRedo: false,
    });

    await usePaintStore.getState().dispatchAction("create_layer", { name: "新层" });

    expect(bridge.applyAction).toHaveBeenCalledWith(
      "canvas-1",
      "create_layer",
      { name: "新层" },
      "user",
      0,
    );
    expect(usePaintStore.getState().document?.revision).toBe(1);
    expect(usePaintStore.getState().history?.canUndo).toBe(true);
  });

  it("后端报错时写入 error 且不覆盖文档", async () => {
    const doc = document();
    usePaintStore.setState({ document: doc });
    vi.mocked(bridge.applyAction).mockRejectedValue({
      kind: "revisionConflict",
      message: "画布已被修改",
    });

    await usePaintStore.getState().dispatchAction("create_layer", {});

    const state = usePaintStore.getState();
    expect(state.document?.revision).toBe(0);
    expect(state.error).toContain("画布已被修改");
  });

  it("没有文档时 dispatch 直接返回", async () => {
    await usePaintStore.getState().dispatchAction("create_layer", {});
    expect(bridge.applyAction).not.toHaveBeenCalled();
  });

  it("undo/redo 走专用命令", async () => {
    const doc = document();
    usePaintStore.setState({ document: doc });
    vi.mocked(bridge.undo).mockResolvedValue({ ...doc, revision: 2 });
    vi.mocked(bridge.redo).mockResolvedValue({ ...doc, revision: 3 });
    vi.mocked(bridge.historyStatus).mockResolvedValue({
      cursor: 0,
      total: 1,
      canUndo: false,
      canRedo: false,
    });

    await usePaintStore.getState().undo();
    expect(bridge.undo).toHaveBeenCalledWith("canvas-1");

    await usePaintStore.getState().redo();
    expect(bridge.redo).toHaveBeenCalledWith("canvas-1");
  });

  it("activeLayer 回落到顶层图层", () => {
    const doc = document();
    doc.activeLayerId = "layer-missing";
    expect(activeLayer(doc)?.id).toBe("layer-1");
    expect(activeLayer(null)).toBeNull();
  });

  it("toErrorMessage 兼容字符串、载荷对象与未知值", () => {
    expect(toErrorMessage("boom")).toBe("boom");
    expect(toErrorMessage({ kind: "io", message: "失败" })).toBe("io: 失败");
    expect(toErrorMessage(42)).toBe("未知错误");
  });
});
