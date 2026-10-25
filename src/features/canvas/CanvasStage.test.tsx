import { beforeEach, describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/react";

const fabric = vi.hoisted(() => {
  type Dims = { width: number; height: number };

  class FakeCanvas {
    static instances: FakeCanvas[] = [];
    backgroundColor: string | undefined;
    readonly element: unknown;
    readonly setDimensions = vi.fn<(dims: Dims) => void>();
    readonly requestRenderAll = vi.fn<() => void>();
    readonly renderAll = vi.fn<() => void>();
    readonly add = vi.fn();
    readonly remove = vi.fn();
    readonly destroy = vi.fn();
    readonly toDataURL = vi.fn(() => "data:image/png;base64,MOCK");

    constructor(element: unknown, options: Record<string, unknown> = {}) {
      this.element = element;
      this.backgroundColor = options.backgroundColor as string | undefined;
      FakeCanvas.instances.push(this);
    }
  }

  return { FakeCanvas };
});

vi.mock("fabric", () => ({ Canvas: fabric.FakeCanvas }));

import { CanvasStage } from "./CanvasStage";

beforeEach(() => {
  fabric.FakeCanvas.instances.length = 0;
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

  it("卸载时销毁 Fabric 画布", () => {
    const { unmount } = render(<CanvasStage />);
    const canvas = fabric.FakeCanvas.instances[0]!;

    unmount();

    expect(canvas.destroy).toHaveBeenCalled();
  });
});
