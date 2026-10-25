import { Canvas } from "fabric";

/**
 * 画布渲染引擎：React 无关的 Fabric.js 封装。
 *
 * 只做「渲染与输入采集」的准备（创建/销毁/尺寸/空白画布/导出快照），
 * 不持有文档状态、不做历史管理——状态真相源在 Rust 侧（见 ADR-0002、工程契约第 2 节）。
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

export interface CanvasEngineOptions {
  documentSize?: Size;
  backgroundColor?: string;
}

export class CanvasEngine {
  private readonly element: HTMLCanvasElement;
  private readonly documentSize: Size;
  private readonly backgroundColor: string;
  private canvas: Canvas | null = null;

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
  }

  /** 导出当前画布为 PNG data URL。 */
  toDataURL(): string {
    if (!this.canvas) {
      throw new Error("画布尚未初始化，无法导出");
    }
    return this.canvas.toDataURL({ format: "png", multiplier: 1 });
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
  }
}
