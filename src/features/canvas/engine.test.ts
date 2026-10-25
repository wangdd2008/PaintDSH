import { beforeEach, describe, expect, it, vi } from "vitest";

const fabric = vi.hoisted(() => {
  type Dims = { width: number; height: number };

  class FakeCanvas {
    static instances: FakeCanvas[] = [];
    backgroundColor: string | undefined;
    readonly element: unknown;
    readonly options: Record<string, unknown>;
    readonly setDimensions = vi.fn<(dims: Dims) => void>();
    readonly requestRenderAll = vi.fn<() => void>();
    readonly renderAll = vi.fn<() => void>();
    readonly add = vi.fn();
    readonly remove = vi.fn();
    readonly clear = vi.fn();
    readonly destroy = vi.fn();
    readonly toDataURL = vi.fn(() => "data:image/png;base64,MOCK");

    constructor(element: unknown, options: Record<string, unknown> = {}) {
      this.element = element;
      this.options = options;
      this.backgroundColor = options.backgroundColor as string | undefined;
      FakeCanvas.instances.push(this);
    }
  }

  return { FakeCanvas };
});

vi.mock("fabric", () => ({ Canvas: fabric.FakeCanvas }));

import {
  CanvasEngine,
  DEFAULT_BACKGROUND_COLOR,
  DEFAULT_DOCUMENT_SIZE,
  fitCanvasToViewport,
  orderedLayerIds,
} from "./engine";

beforeEach(() => {
  fabric.FakeCanvas.instances.length = 0;
});

describe("fitCanvasToViewport", () => {
  it("按窗口可用空间等比缩放，并扣除内边距", () => {
    const fit = fitCanvasToViewport({ width: 1000, height: 800 }, { width: 1280, height: 800 }, 48);

    expect(fit.zoom).toBeCloseTo(0.70625, 5);
    expect(fit).toMatchObject({ width: 904, height: 565 });
  });

  it("窗口足够大时保持 1:1，不做放大", () => {
    const fit = fitCanvasToViewport({ width: 2000, height: 2000 }, { width: 1280, height: 800 });

    expect(fit).toEqual({ width: 1280, height: 800, zoom: 1 });
  });

  it("窗口尺寸无效（0 或负）时退回文档原始尺寸", () => {
    const fit = fitCanvasToViewport({ width: 0, height: -10 });

    expect(fit).toEqual({ width: 1280, height: 800, zoom: 1 });
  });

  it("窗口很窄时宽度是瓶颈", () => {
    const fit = fitCanvasToViewport({ width: 400, height: 300 }, { width: 1280, height: 800 }, 48);

    expect(fit).toEqual({ width: 304, height: 190, zoom: 0.2375 });
  });

  it("默认文档尺寸为 1280x800、背景为白色", () => {
    expect(DEFAULT_DOCUMENT_SIZE).toEqual({ width: 1280, height: 800 });
    expect(DEFAULT_BACKGROUND_COLOR).toBe("#ffffff");
  });
});

describe("orderedLayerIds", () => {
  it("返回自底向上的绘制顺序，并跳过隐藏图层", () => {
    const order = orderedLayerIds([
      { id: "background", visible: true },
      { id: "ink", visible: false },
      { id: "ai-circle", visible: true },
    ]);

    expect(order).toEqual(["background", "ai-circle"]);
  });

  it("空图层列表返回空数组", () => {
    expect(orderedLayerIds([])).toEqual([]);
  });
});

describe("CanvasEngine", () => {
  it("mount 时创建 Fabric Canvas、设置白色背景与文档尺寸", () => {
    const element = document.createElement("canvas");
    const engine = new CanvasEngine(element);

    engine.mount();

    expect(fabric.FakeCanvas.instances).toHaveLength(1);
    const canvas = fabric.FakeCanvas.instances[0]!;
    expect(canvas.element).toBe(element);
    expect(canvas.backgroundColor).toBe("#ffffff");
    expect(canvas.setDimensions).toHaveBeenCalledWith({ width: 1280, height: 800 });
    expect(engine.isMounted()).toBe(true);
  });

  it("可以通过选项覆盖文档尺寸与背景色", () => {
    const engine = new CanvasEngine(document.createElement("canvas"), {
      documentSize: { width: 640, height: 480 },
      backgroundColor: "#101010",
    });

    engine.mount();

    const canvas = fabric.FakeCanvas.instances[0]!;
    expect(canvas.backgroundColor).toBe("#101010");
    expect(canvas.setDimensions).toHaveBeenCalledWith({ width: 640, height: 480 });
  });

  it("resize 会更新画布显示尺寸并重绘", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));
    engine.mount();
    const canvas = fabric.FakeCanvas.instances[0]!;

    engine.resize({ width: 400, height: 250 });

    expect(canvas.setDimensions).toHaveBeenLastCalledWith({ width: 400, height: 250 });
    expect(canvas.requestRenderAll).toHaveBeenCalled();
  });

  it("renderBlank 清空内容并恢复白色纸面", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));
    engine.mount();
    const canvas = fabric.FakeCanvas.instances[0]!;
    canvas.backgroundColor = "#000000";

    engine.renderBlank();

    expect(canvas.clear).toHaveBeenCalled();
    expect(canvas.backgroundColor).toBe("#ffffff");
    expect(canvas.requestRenderAll).toHaveBeenCalled();
  });

  it("toDataURL 以 PNG 导出当前画布", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));
    engine.mount();

    expect(engine.toDataURL()).toBe("data:image/png;base64,MOCK");
    expect(fabric.FakeCanvas.instances[0]!.toDataURL).toHaveBeenCalledWith({
      format: "png",
      multiplier: 1,
    });
  });

  it("未挂载时调用 toDataURL 抛错", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));

    expect(() => engine.toDataURL()).toThrow(/尚未初始化/);
  });

  it("destroy 释放 Fabric Canvas", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));
    engine.mount();

    engine.destroy();

    expect(fabric.FakeCanvas.instances[0]!.destroy).toHaveBeenCalled();
    expect(engine.isMounted()).toBe(false);
  });

  it("重复 mount 会先销毁旧实例", () => {
    const engine = new CanvasEngine(document.createElement("canvas"));
    engine.mount();
    engine.mount();

    expect(fabric.FakeCanvas.instances).toHaveLength(2);
    expect(fabric.FakeCanvas.instances[0]!.destroy).toHaveBeenCalled();
  });
});
