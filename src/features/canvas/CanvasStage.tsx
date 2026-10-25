import { useEffect, useRef, useState } from "react";

import type { Layer } from "../../lib/types";
import { activeLayer, usePaintStore } from "../../stores/paint";
import {
  CANVAS_VIEWPORT_PADDING,
  CanvasEngine,
  DEFAULT_DOCUMENT_SIZE,
  fitCanvasToViewport,
  layerSvg,
  setCurrentEngine,
  type Size,
} from "./engine";
import { appendElement, shapeElement, textElement } from "./svg";

/**
 * 画布舞台：把非 React 的 CanvasEngine 挂到 DOM 上，跟随容器尺寸变化，
 * 并把「用户交互结果」翻译成 dispatch（人类与 AI 同一条写入口）。
 */
export function CanvasStage() {
  const hostRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<CanvasEngine | null>(null);
  const [viewport, setViewport] = useState<Size>({ width: 0, height: 0 });

  const document = usePaintStore((state) => state.document);
  const tool = usePaintStore((state) => state.tool);
  const shapeKind = usePaintStore((state) => state.shapeKind);
  const brushColor = usePaintStore((state) => state.brushColor);
  const brushWidth = usePaintStore((state) => state.brushWidth);
  const dispatchAction = usePaintStore((state) => state.dispatchAction);
  const setError = usePaintStore((state) => state.setError);

  useEffect(() => {
    const element = canvasRef.current;
    if (!element) {
      return;
    }
    const engine = new CanvasEngine(element, { documentSize: DEFAULT_DOCUMENT_SIZE });
    engine.mount();
    engineRef.current = engine;
    setCurrentEngine(engine);

    return () => {
      setCurrentEngine(null);
      engine.destroy();
      engineRef.current = null;
    };
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }

    const measure = () => {
      setViewport({ width: host.clientWidth, height: host.clientHeight });
    };
    measure();

    if (typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", measure);
      return () => window.removeEventListener("resize", measure);
    }

    const observer = new ResizeObserver(measure);
    observer.observe(host);
    return () => observer.disconnect();
  }, []);

  const fit = fitCanvasToViewport(
    viewport,
    document ? { width: document.width, height: document.height } : DEFAULT_DOCUMENT_SIZE,
    CANVAS_VIEWPORT_PADDING,
  );

  useEffect(() => {
    engineRef.current?.resize({ width: fit.width, height: fit.height });
  }, [fit.width, fit.height]);

  // 文档变化 → 重绘
  useEffect(() => {
    if (!document) {
      engineRef.current?.renderBlank();
      return;
    }
    void engineRef.current?.renderDocument(document);
  }, [document]);

  // 工具切换：画笔/橡皮进绘制模式，其余可选中
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) {
      return;
    }
    const drawing = tool === "brush" || tool === "eraser";
    engine.setDrawingMode(drawing, {
      // 橡皮用画布背景色覆盖（矢量图层无法真正擦除像素，见 ticket-03 备注）
      color: tool === "eraser" ? (document?.backgroundColor ?? "#ffffff") : brushColor,
      width: brushWidth,
    });
    engine.setInteractive(tool === "select");
  }, [tool, brushColor, brushWidth, document?.backgroundColor, document?.revision]);

  // 画笔落笔完成 → 追加到当前图层
  useEffect(() => {
    const canvas = engineRef.current?.getCanvas();
    if (!canvas) {
      return;
    }
    const handler = (event: { path?: { toSVG: () => string } }) => {
      const layer = activeLayer(usePaintStore.getState().document);
      const path = event.path;
      if (!layer || !path) {
        return;
      }
      const element = path.toSVG();
      void dispatchAction("set_layer_content", {
        layerId: layer.id,
        content: { type: "vector", svg: appendElement(layerSvg(layer), element) },
      });
    };
    canvas.on("path:created", handler as never);
    return () => {
      canvas.off("path:created", handler as never);
    };
  }, [dispatchAction, document?.revision]);

  // 选择工具拖动 → 提交 transform
  useEffect(() => {
    const engine = engineRef.current;
    const canvas = engine?.getCanvas();
    if (!engine || !canvas) {
      return;
    }
    const handler = (event: { target?: unknown }) => {
      const layerId = engine.layerIdOf(event.target as never);
      const target = event.target as
        | { left?: number; top?: number; scaleX?: number; scaleY?: number; angle?: number }
        | undefined;
      if (!layerId || !target) {
        return;
      }
      void dispatchAction("set_layer_transform", {
        layerId,
        transform: {
          x: target.left ?? 0,
          y: target.top ?? 0,
          scaleX: target.scaleX ?? 1,
          scaleY: target.scaleY ?? 1,
          rotation: target.angle ?? 0,
        },
      });
    };
    canvas.on("object:modified", handler as never);
    return () => {
      canvas.off("object:modified", handler as never);
    };
  }, [dispatchAction, document?.revision]);

  // 形状/文字：按下与抬起的位置决定元素
  const downPoint = useRef<{ x: number; y: number } | null>(null);

  const handlePointerDown = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const engine = engineRef.current;
    if (!engine || !document) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const scaleX = document.width / Math.max(rect.width, 1);
    const scaleY = document.height / Math.max(rect.height, 1);
    downPoint.current = {
      x: (event.clientX - rect.left) * scaleX,
      y: (event.clientY - rect.top) * scaleY,
    };

    if (tool === "text") {
      const layer = activeLayer(document);
      const text = window.prompt("输入文字");
      if (!layer || !text) {
        return;
      }
      void dispatchAction("set_layer_content", {
        layerId: layer.id,
        content: {
          type: "vector",
          svg: appendElement(
            layerSvg(layer),
            textElement(downPoint.current.x, downPoint.current.y, text),
          ),
        },
      });
    }
  };

  const handlePointerUp = (event: React.PointerEvent<HTMLCanvasElement>) => {
    const engine = engineRef.current;
    const start = downPoint.current;
    downPoint.current = null;
    if (!engine || !document || !start || tool !== "shape") {
      return;
    }
    const layer = activeLayer(document);
    if (!layer) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    const scaleX = document.width / Math.max(rect.width, 1);
    const scaleY = document.height / Math.max(rect.height, 1);
    const end = {
      x: (event.clientX - rect.left) * scaleX,
      y: (event.clientY - rect.top) * scaleY,
    };
    const width = Math.abs(end.x - start.x);
    const height = Math.abs(end.y - start.y);
    if (width < 2 || height < 2) {
      setError("形状太小了，拖动出更大的范围");
      return;
    }

    void dispatchAction("set_layer_content", {
      layerId: layer.id,
      content: {
        type: "vector",
        svg: appendElement(
          layerSvg(layer),
          shapeElement(shapeKind, start.x, start.y, end.x - start.x, end.y - start.y, {
            fill: brushColor,
          }),
        ),
      },
    });
  };

  const active = activeLayer(document);
  const activeNote = layerNote(active);

  return (
    <div className="canvas-stage" data-testid="canvas-stage" ref={hostRef}>
      <div
        className="canvas-stage__frame"
        style={{ width: `${fit.width}px`, height: `${fit.height}px` }}
      >
        <canvas
          ref={canvasRef}
          data-testid="paint-canvas"
          aria-label="画布"
          onPointerDown={handlePointerDown}
          onPointerUp={handlePointerUp}
          style={{ touchAction: "none" }}
        />
      </div>
      <p className="canvas-stage__hint" data-testid="canvas-hint">
        {activeNote}
      </p>
    </div>
  );
}

function layerNote(layer: Layer | null): string {
  if (!layer) {
    return "还没有图层";
  }
  if (layer.locked) {
    return `图层「${layer.name}」已锁定`;
  }
  return `当前图层：${layer.name}`;
}
