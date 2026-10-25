import {
  Canvas,
  FabricImage,
  FabricObject,
  Group,
  loadSVGFromString,
  PencilBrush,
  Point,
  util,
} from "fabric";

import type { CanvasDocument, Layer } from "../../lib/types";
import { toFabricFilters } from "./filters";
import { normalizeSvg } from "./svg";

/**
 * 画布渲染引擎：React 无关的 Fabric.js 封装。
 *
 * 只做「渲染与输入采集」：把 Rust 下发的 `CanvasDocument` 渲染成 Fabric 对象，
 * 不持有文档状态、不做历史管理——状态真相源在 Rust 侧（engineering-contract 第 2 节）。
 */

export interface Size {
  width: number;
  height: number;
}

/** 默认画布文档尺寸（像素）。 */
export const DEFAULT_DOCUMENT_SIZE: Size = { width: 1280, height: 800 };

/** 默认画纸颜色：不透明的白，保证空画布有可见的纸面。 */
export const DEFAULT_BACKGROUND_COLOR = "#ffffff";

/** 画布在视口里的留白（Figma 风格的深色工作区）。 */
export const CANVAS_VIEWPORT_PADDING = 48;

/** 缩放下限，避免窗口极小时把画布压成 0 像素。 */
const MIN_ZOOM = 0.05;

export interface ViewportFit extends Size {
  /** 显示缩放比例（0 < zoom <= 1，不放大）。 */
  zoom: number;
}

/**
 * 计算画布在视口中的显示尺寸：等比缩放、留内边距、不放大。
 * 视口或文档尺寸无效时退回文档原始尺寸。
 */
export function fitCanvasToViewport(
  viewport: Size,
  documentSize: Size = DEFAULT_DOCUMENT_SIZE,
  padding: number = CANVAS_VIEWPORT_PADDING,
): ViewportFit {
  const viewportValid =
    Number.isFinite(viewport.width) &&
    Number.isFinite(viewport.height) &&
    viewport.width > 0 &&
    viewport.height > 0;
  const documentValid = documentSize.width > 0 && documentSize.height > 0;

  if (!viewportValid || !documentValid) {
    return { width: documentSize.width, height: documentSize.height, zoom: 1 };
  }

  const usableWidth = Math.max(viewport.width - padding * 2, 1);
  const usableHeight = Math.max(viewport.height - padding * 2, 1);
  const zoom = Math.max(
    Math.min(usableWidth / documentSize.width, usableHeight / documentSize.height, 1),
    MIN_ZOOM,
  );

  return {
    width: Math.round(documentSize.width * zoom),
    height: Math.round(documentSize.height * zoom),
    zoom,
  };
}

/**
 * 图层绘制顺序（自底向上）：数组下标 0 在最底层，隐藏图层不参与渲染。
 * Fabric 后加入的对象在上层，所以顺序即 add 顺序。
 */
export function orderedLayerIds(
  layers: ReadonlyArray<{ id: string; visible: boolean }>,
): string[] {
  return layers.filter((layer) => layer.visible).map((layer) => layer.id);
}

/** 图层内容的 SVG 源码（位图返回空串）。 */
export function layerSvg(layer: Layer): string {
  switch (layer.content.type) {
    case "vector":
      return normalizeSvg(layer.content.svg);
    case "aiCode":
      return normalizeSvg(layer.content.code);
    default:
      return "";
  }
}

/**
 * 当前挂载的引擎实例。菜单栏导出需要拿到它，用注册表避免把 engine 一路透传。
 * `CanvasStage` 挂载/卸载时维护。
 */
let currentEngine: CanvasEngine | null = null;

export function setCurrentEngine(engine: CanvasEngine | null): void {
  currentEngine = engine;
}

export function getCurrentEngine(): CanvasEngine | null {
  return currentEngine;
}

export interface CanvasEngineOptions {
  documentSize?: Size;
  backgroundColor?: string;
}

export class CanvasEngine {
  private readonly element: HTMLCanvasElement;
  private documentSize: Size;
  private backgroundColor: string;
  private canvas: Canvas | null = null;
  private renderedRevision = -1;
  private readonly layerObjects = new Map<string, FabricObject>();

  constructor(element: HTMLCanvasElement, options: CanvasEngineOptions = {}) {
    this.element = element;
    this.documentSize = options.documentSize ?? DEFAULT_DOCUMENT_SIZE;
    this.backgroundColor = options.backgroundColor ?? DEFAULT_BACKGROUND_COLOR;
  }

  /** 创建 Fabric Canvas 并按文档尺寸设置画布；重复调用会先销毁旧实例。 */
  mount(): Canvas {
    if (this.canvas) {
      this.destroy();
    }

    const canvas = new Canvas(this.element, {
      backgroundColor: this.backgroundColor,
      preserveObjectStacking: true,
      selection: false,
      stopContextMenu: true,
    });
    canvas.setDimensions({
      width: this.documentSize.width,
      height: this.documentSize.height,
    });
    canvas.requestRenderAll();

    this.canvas = canvas;
    return canvas;
  }

  /** 更新画布显示尺寸（窗口尺寸变化时调用）。 */
  resize(size: Size): void {
    if (!this.canvas) {
      return;
    }
    this.canvas.setDimensions({
      width: Math.max(1, Math.round(size.width)),
      height: Math.max(1, Math.round(size.height)),
    });
    this.canvas.requestRenderAll();
  }

  /** 清空内容并恢复白纸背景。 */
  renderBlank(): void {
    if (!this.canvas) {
      return;
    }
    this.canvas.clear();
    this.canvas.backgroundColor = this.backgroundColor;
    this.canvas.requestRenderAll();
    // 清空后必须让渲染缓存失效，否则同一 revision 的文档会被当成"已渲染"而跳过。
    this.renderedRevision = -1;
    this.layerObjects.clear();
  }

  /**
   * 声明式渲染：把文档内容同步到 Fabric。
   *
   * 简化实现：revision 变化时整画布重建（百图层以内足够快，且避免增量 diff 的边界 bug）。
   */
  async renderDocument(document: CanvasDocument): Promise<void> {
    const canvas = this.canvas;
    if (!canvas) {
      return;
    }
    if (this.renderedRevision === document.revision && this.layerObjects.size > 0) {
      return;
    }

    this.documentSize = { width: document.width, height: document.height };
    this.backgroundColor = document.backgroundColor;
    canvas.clear();
    canvas.backgroundColor = document.backgroundColor;
    canvas.setDimensions({ width: document.width, height: document.height });

    this.layerObjects.clear();
    for (const layer of document.layers) {
      if (!layer.visible) {
        continue;
      }
      const object = await this.buildLayerObject(layer);
      if (!object) {
        continue;
      }
      this.layerObjects.set(layer.id, object);
      canvas.add(object);
    }

    this.renderedRevision = document.revision;
    canvas.requestRenderAll();
  }

  private async buildLayerObject(layer: Layer): Promise<FabricObject | null> {
    if (layer.content.type === "raster") {
      try {
        const image = await FabricImage.fromURL(layer.content.dataUrl);
        image.set({
          left: layer.transform.x,
          top: layer.transform.y,
          scaleX: layer.transform.scaleX,
          scaleY: layer.transform.scaleY,
          angle: layer.transform.rotation,
          opacity: layer.opacity,
          selectable: false,
          evented: false,
          globalCompositeOperation: layer.blendMode,
        });
        if (layer.metadata.filters.length > 0) {
          image.filters = toFabricFilters(layer.metadata.filters);
          image.applyFilters();
        }
        return image;
      } catch {
        return null;
      }
    }

    const svg = layerSvg(layer);
    try {
      const { objects, options } = await loadSVGFromString(svg);
      const valid = objects.filter((object): object is FabricObject => Boolean(object));
      const group = util.groupSVGElements(valid, options) as Group;
      group.set({
        left: layer.transform.x,
        top: layer.transform.y,
        scaleX: layer.transform.scaleX,
        scaleY: layer.transform.scaleY,
        angle: layer.transform.rotation,
        opacity: layer.opacity,
        selectable: false,
        evented: false,
        globalCompositeOperation: layer.blendMode,
      });
      return group;
    } catch {
      return null;
    }
  }

  /** 打开/关闭自由绘制，并配置画笔。 */
  setDrawingMode(enabled: boolean, options: { color?: string; width?: number } = {}): void {
    const canvas = this.canvas;
    if (!canvas) {
      return;
    }
    canvas.isDrawingMode = enabled;
    if (!enabled) {
      return;
    }
    const brush = new PencilBrush(canvas);
    brush.color = options.color ?? "#1c2024";
    brush.width = options.width ?? 4;
    canvas.freeDrawingBrush = brush;
  }

  /** 屏幕坐标 → 画布坐标。 */
  pointerToCanvas(point: { x: number; y: number }): Point {
    const canvas = this.canvas;
    if (!canvas) {
      return new Point(point.x, point.y);
    }
    const inverse = util.invertTransform(canvas.viewportTransform ?? [1, 0, 0, 1, 0, 0]);
    return util.transformPoint(new Point(point.x, point.y), inverse);
  }

  /** 切换对象是否可交互（选择工具开启，画笔类工具关闭）。 */
  setInteractive(enabled: boolean): void {
    const canvas = this.canvas;
    if (!canvas) {
      return;
    }
    for (const object of this.layerObjects.values()) {
      object.set({ selectable: enabled, evented: enabled });
    }
    canvas.selection = enabled;
    canvas.requestRenderAll();
  }

  /** 查对象属于哪个图层。 */
  layerIdOf(object: FabricObject | null | undefined): string | null {
    if (!object) {
      return null;
    }
    for (const [layerId, candidate] of this.layerObjects) {
      if (candidate === object) {
        return layerId;
      }
    }
    return null;
  }

  /** 取鼠标下方最上层对象所属的图层 id（橡皮/选择用）。 */
  layerIdAt(point: Point): string | null {
    const canvas = this.canvas;
    if (!canvas) {
      return null;
    }
    const target = canvas.findTarget({ clientX: point.x, clientY: point.y } as never);
    return this.layerIdOf(target as FabricObject | null);
  }

  /** 导出 PNG/JPG data URL。 */
  toDataURL(format: "png" | "jpg" = "png"): string {
    if (!this.canvas) {
      throw new Error("画布尚未初始化，无法导出");
    }
    // Fabric 的 ImageFormat 里 JPG 写作 "jpeg"。
    const imageFormat = format === "jpg" ? "jpeg" : "png";
    return this.canvas.toDataURL({ format: imageFormat, multiplier: 1 });
  }

  /** 导出 SVG（Fabric 生成，包含矢量与内嵌位图）。 */
  toSvg(): string {
    if (!this.canvas) {
      throw new Error("画布尚未初始化，无法导出");
    }
    return this.canvas.toSVG();
  }

  isMounted(): boolean {
    return this.canvas !== null;
  }

  getCanvas(): Canvas | null {
    return this.canvas;
  }

  /** 释放 Fabric 实例（React 组件卸载时调用）。 */
  destroy(): void {
    this.canvas?.destroy();
    this.canvas = null;
    this.renderedRevision = -1;
    this.layerObjects.clear();
  }
}
