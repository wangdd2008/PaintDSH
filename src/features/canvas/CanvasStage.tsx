import { useEffect, useRef, useState } from "react";

import {
  CANVAS_VIEWPORT_PADDING,
  CanvasEngine,
  DEFAULT_DOCUMENT_SIZE,
  fitCanvasToViewport,
  type Size,
} from "./engine";

/**
 * 画布舞台：把非 React 的 CanvasEngine 挂到 DOM 上，
 * 跟随容器尺寸变化调整显示大小，卸载时销毁 Fabric 实例。
 */
export function CanvasStage() {
  const hostRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<CanvasEngine | null>(null);
  const [viewport, setViewport] = useState<Size>({ width: 0, height: 0 });

  useEffect(() => {
    const element = canvasRef.current;
    if (!element) {
      return;
    }
    const engine = new CanvasEngine(element, { documentSize: DEFAULT_DOCUMENT_SIZE });
    engine.mount();
    engineRef.current = engine;

    return () => {
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

  const fit = fitCanvasToViewport(viewport, DEFAULT_DOCUMENT_SIZE, CANVAS_VIEWPORT_PADDING);

  useEffect(() => {
    engineRef.current?.resize({ width: fit.width, height: fit.height });
  }, [fit.width, fit.height]);

  return (
    <div className="canvas-stage" data-testid="canvas-stage" ref={hostRef}>
      <div
        className="canvas-stage__frame"
        style={{ width: `${fit.width}px`, height: `${fit.height}px` }}
      >
        <canvas ref={canvasRef} data-testid="paint-canvas" aria-label="画布" />
      </div>
    </div>
  );
}
