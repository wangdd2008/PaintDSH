import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

const fabric = vi.hoisted(() => {
  class FakeObject {
    readonly set = vi.fn();
    readonly applyFilters = vi.fn();
  }

  class FakeCanvas {
    static instances: FakeCanvas[] = [];
    backgroundColor: string | undefined;
    isDrawingMode = false;
    selection = false;
    viewportTransform = [1, 0, 0, 1, 0, 0];
    readonly setDimensions = vi.fn();
    readonly requestRenderAll = vi.fn();
    readonly add = vi.fn();
    readonly clear = vi.fn();
    readonly destroy = vi.fn();
    readonly on = vi.fn();
    readonly off = vi.fn();
    readonly findTarget = vi.fn(() => null);
    readonly toDataURL = vi.fn(() => "data:image/png;base64,MOCK");
    readonly toSVG = vi.fn(() => "<svg></svg>");

    constructor(_element: unknown, options: Record<string, unknown> = {}) {
      this.backgroundColor = options.backgroundColor as string | undefined;
      FakeCanvas.instances.push(this);
    }
  }

  return { FakeCanvas, FakeObject };
});

vi.mock("fabric", () => ({
  Canvas: fabric.FakeCanvas,
  FabricObject: fabric.FakeObject,
  FabricImage: Object.assign(fabric.FakeObject, {
    fromURL: vi.fn(async () => new fabric.FakeObject()),
  }),
  Group: fabric.FakeObject,
  PencilBrush: class {
    color = "#000";
    width = 1;
  },
  Point: class {
    constructor(
      public x: number,
      public y: number,
    ) {}
  },
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

const bridge = vi.hoisted(() => ({
  ping: vi.fn(),
  invokeCommand: vi.fn(),
  onEvent: vi.fn(),
  createCanvas: vi.fn(),
  applyAction: vi.fn(),
  undo: vi.fn(),
  redo: vi.fn(),
  historyStatus: vi.fn(),
  listEmployees: vi.fn(),
  listKnowledge: vi.fn(),
  knowledgeDirs: vi.fn(),
  aiListCalls: vi.fn(),
  aiUsageSummary: vi.fn(),
}));

vi.mock("./lib/bridge", () => bridge);

import { App } from "./App";
import { initialPaintState, usePaintStore } from "./stores/paint";

beforeEach(() => {
  fabric.FakeCanvas.instances.length = 0;
  bridge.ping.mockReset();
  bridge.createCanvas.mockReset();
  bridge.createCanvas.mockResolvedValue({
    id: "canvas-1",
    name: "未命名作品",
    width: 1280,
    height: 800,
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
  });
  bridge.historyStatus.mockResolvedValue({
    cursor: 0,
    total: 1,
    canUndo: false,
    canRedo: false,
  });
  bridge.onEvent.mockResolvedValue(() => undefined);
  bridge.listEmployees.mockResolvedValue([]);
  bridge.listKnowledge.mockResolvedValue([]);
  bridge.knowledgeDirs.mockResolvedValue({ root: "C:/paint/knowledge", dirs: [] });
  bridge.aiListCalls.mockResolvedValue([]);
  bridge.aiUsageSummary.mockResolvedValue([]);
  usePaintStore.setState({ ...initialPaintState });
});

describe("App 布局", () => {
  it("渲染三栏布局：工具栏 / 画布 / 右侧面板 + 菜单栏 + 状态栏", async () => {
    bridge.ping.mockResolvedValue({ pong: true, version: "0.1.0" });

    render(<App />);

    expect(screen.getByTestId("menu-bar")).toBeInTheDocument();
    expect(screen.getByTestId("toolbox")).toBeInTheDocument();
    expect(screen.getByTestId("canvas-area")).toBeInTheDocument();
    expect(screen.getByTestId("side-panel")).toBeInTheDocument();
    expect(screen.getByTestId("layer-panel")).toBeInTheDocument();
    expect(screen.getByTestId("status-bar")).toBeInTheDocument();
    await screen.findByText("后端已连接 v0.1.0");
  });

  it("右侧可以在图层/滤镜/AI/员工/知识库/工作台之间切换", async () => {
    bridge.ping.mockResolvedValue({ pong: true, version: "0.1.0" });

    render(<App />);
    await screen.findByText("后端已连接 v0.1.0");

    expect(screen.getByTestId("layer-panel")).toBeInTheDocument();

    screen.getByRole("button", { name: "AI" }).click();
    expect(await screen.findByTestId("chat-panel")).toBeInTheDocument();

    screen.getByRole("button", { name: "知识库" }).click();
    expect(await screen.findByTestId("knowledge-panel")).toBeInTheDocument();
  });
});

describe("App 的后端连通性提示", () => {
  it("ping 成功时状态栏显示后端版本", async () => {
    bridge.ping.mockResolvedValue({ pong: true, version: "0.1.0" });

    render(<App />);

    expect(await screen.findByText("后端已连接 v0.1.0")).toBeInTheDocument();
    expect(bridge.ping).toHaveBeenCalledTimes(1);
  });

  it("ping 失败时状态栏显示后端未连接", async () => {
    bridge.ping.mockRejectedValue("ipc 不可用");

    render(<App />);

    expect(await screen.findByText("后端未连接")).toBeInTheDocument();
  });
});
