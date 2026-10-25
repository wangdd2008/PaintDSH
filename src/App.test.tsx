import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

const fabric = vi.hoisted(() => {
  class FakeCanvas {
    static instances: FakeCanvas[] = [];
    backgroundColor: string | undefined;
    readonly setDimensions = vi.fn();
    readonly requestRenderAll = vi.fn();
    readonly renderAll = vi.fn();
    readonly add = vi.fn();
    readonly remove = vi.fn();
    readonly destroy = vi.fn();
    readonly toDataURL = vi.fn(() => "data:image/png;base64,MOCK");

    constructor(_element: unknown, options: Record<string, unknown> = {}) {
      this.backgroundColor = options.backgroundColor as string | undefined;
      FakeCanvas.instances.push(this);
    }
  }

  return { FakeCanvas };
});

vi.mock("fabric", () => ({ Canvas: fabric.FakeCanvas }));

const bridge = vi.hoisted(() => ({
  ping: vi.fn(),
  invokeCommand: vi.fn(),
  onEvent: vi.fn(),
}));

vi.mock("./lib/bridge", () => bridge);

import { App } from "./App";
import { initialPaintState, usePaintStore } from "./stores/paint";

beforeEach(() => {
  fabric.FakeCanvas.instances.length = 0;
  bridge.ping.mockReset();
  usePaintStore.setState(initialPaintState);
});

describe("App 布局", () => {
  it("渲染三栏布局：工具栏 / 画布 / 图层面板 + 菜单栏 + 状态栏", async () => {
    bridge.ping.mockResolvedValue({ pong: true, version: "0.1.0" });

    render(<App />);

    expect(screen.getByTestId("menu-bar")).toBeInTheDocument();
    expect(screen.getByTestId("toolbox")).toBeInTheDocument();
    expect(screen.getByTestId("canvas-area")).toBeInTheDocument();
    expect(screen.getByTestId("layer-panel")).toBeInTheDocument();
    expect(screen.getByTestId("status-bar")).toBeInTheDocument();
    await screen.findByText("后端已连接 v0.1.0");
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
