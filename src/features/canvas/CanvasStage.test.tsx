import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, waitFor } from "@testing-library/react";

/**
 * jsdom 没有 canvas 2D 上下文，Fabric 必须整体替换。
 * 这里给引擎用到的 API 全部提供假实现（含 drawing mode 与 SVG 解析）。
 */
const fabric = vi.hoisted(() => {
  type Dims = { width: number; height: number };

  class FakeObject {
    left = 0;
    top = 0;
    scaleX = 1;
    scaleY = 1;
    angle = 0;
    opacity = 1;
    filters: unknown[] = [];
    readonly set = vi.fn();
    readonly applyFilters = vi.fn();
  }

  class FakeCanvas {
    static instances: FakeCanvas[] = [];
    backgroundColor: string | undefined;
    selection = false;
    isDrawingMode = false;
    freeDrawingBrush: unknown = null;
    viewportTransform = [1, 0, 0, 1, 0, 0];
    readonly element: unknown;
    readonly setDimensions = vi.fn<(dims: Dims) => void>();
    readonly requestRenderAll = vi.fn<() => void>();
    readonly renderAll = vi.fn<() => void>();
    readonly add = vi.fn();
    readonly remove = vi.fn();
    readonly clear = vi.fn();
    readonly destroy = vi.fn();
    readonly on = vi.fn();
    readonly off = vi.fn();
    readonly findTarget = vi.fn(() => null);
    readonly toDataURL = vi.fn(() => "data:image/png;base64,MOCK");
    readonly toSVG = vi.fn(() => "<svg></svg>");

    constructor(element: unknown, options: Record<string, unknown> = {}) {
      this.element = element;
      this.backgroundColor = options.backgroundColor as string | undefined;
      FakeCanvas.instances.push(this);
    }
  }

  class FakeImage extends FakeObject {
    static readonly fromURL = vi.fn(async () => new FakeImage());
  }

  class FakePencilBrush {
    color = "#000000";
    width = 1;
    constructor(_canvas: unknown) {}
  }

  class FakePoint {
    constructor(
      public x: number,
      public y: number,
    ) {}
  }

  return { FakeCanvas, FakeObject, FakeImage, FakePencilBrush, FakePoint };
});

vi.mock("fabric", () => ({
  Canvas: fabric.FakeCanvas,
  FabricObject: fabric.FakeObject,
  FabricImage: fabric.FakeImage,
  Group: fabric.FakeObject,
  PencilBrush: fabric.FakePencilBrush,
  Point: fabric.FakePoint,
  loadSVGFromString: vi.fn(async () => ({
    objects: [new fabric.FakeObject()],
    options: {},
  })),
  util: {
    invertTransform: () => [1, 0, 0, 1, 0, 0],
    transformPoint: (point: unknown) => point,
    groupSVGElements: () => new fabric.FakeObject(),
  },
}));

vi.mock("../../lib/bridge", () => ({
  createCanvas: vi.fn(),
  applyAction: vi.fn(),
  undo: vi.fn(),
  redo: vi.fn(),
  historyStatus: vi.fn(),
}));

import { CanvasStage } from "./CanvasStage";
import { initialPaintState, usePaintStore } from "../../stores/paint";

beforeEach(() => {
  fabric.FakeCanvas.instances.length = 0;
  usePaintStore.setState({ ...initialPaintState });
});

describe("CanvasStage", () => {
  it("挂载时创建白色背景、1280x800 的 Fabric 画布", () => {
    render(<CanvasStage />);

    expect(fabric.FakeCanvas.instances).toHaveLength(1);
    const canvas = fabric.FakeCanvas.instances[0]!;
    expect(canvas.backgroundColor).toBe("#ffffff");
    expect(canvas.setDimensions).toHaveBeenCalledWith({ width: 1280, height: 800 });
  });

  it("渲染带边框的画布舞台，且画布元素挂在其中", () => {
    const { getByTestId, container } = render(<CanvasStage />);

    const stage = getByTestId("canvas-stage");
    expect(stage).toBeInTheDocument();
    expect(stage).toContainElement(getByTestId("paint-canvas"));
    expect(container.querySelector("canvas")).toBe(getByTestId("paint-canvas"));
  });

  it("没有文档时清空画布并提示还没有图层", () => {
    const { getByTestId } = render(<CanvasStage />);
    const canvas = fabric.FakeCanvas.instances[0]!;

    expect(canvas.clear).toHaveBeenCalled();
    expect(getByTestId("canvas-hint")).toHaveTextContent("还没有图层");
  });

  it("卸载时销毁 Fabric 画布", () => {
    const { unmount } = render(<CanvasStage />);
    const canvas = fabric.FakeCanvas.instances[0]!;

    unmount();

    expect(canvas.destroy).toHaveBeenCalled();
  });

  it("画笔工具开启自由绘制，选择工具关闭", async () => {
    const { unmount } = render(<CanvasStage />);
    const canvas = fabric.FakeCanvas.instances[0]!;

    act(() => {
      usePaintStore.getState().setTool("brush");
    });
    await waitFor(() => expect(canvas.isDrawingMode).toBe(true));

    act(() => {
      usePaintStore.getState().setTool("select");
    });
    await waitFor(() => expect(canvas.isDrawingMode).toBe(false));

    unmount();
  });
});
