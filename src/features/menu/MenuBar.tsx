import * as bridge from "../../lib/bridge";
import type { ExportFormat } from "../../lib/types";
import { usePaintStore } from "../../stores/paint";
import { getCurrentEngine } from "../canvas/engine";

/** 顶部菜单栏：新建/打开/保存/导出 + 撤销重做。 */
export function MenuBar() {
  const document = usePaintStore((state) => state.document);
  const createCanvas = usePaintStore((state) => state.createCanvas);
  const setDocument = usePaintStore((state) => state.setDocument);
  const setError = usePaintStore((state) => state.setError);
  const history = usePaintStore((state) => state.history);
  const undo = usePaintStore((state) => state.undo);
  const redo = usePaintStore((state) => state.redo);

  const handleNew = () => {
    void createCanvas("未命名作品", 1280, 800);
  };

  const handleOpen = async () => {
    try {
      const path = await bridge.pickOpenPath();
      if (!path) {
        return;
      }
      const opened = await bridge.openDocument(path);
      setDocument(opened);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleSave = async () => {
    if (!document) {
      return;
    }
    try {
      const path = await bridge.pickSavePath(`${document.name}.paint`);
      if (!path) {
        return;
      }
      const saved = await bridge.saveDocument(document.id, path);
      setError(null);
      window.alert(`已保存到 ${saved}`);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  const handleExport = async (format: ExportFormat) => {
    const engine = getCurrentEngine();
    if (!document || !engine) {
      return;
    }
    try {
      const path = await bridge.pickExportPath(`${document.name}.${format}`);
      if (!path) {
        return;
      }
      const dataUrl = format === "svg" ? null : engine.toDataURL(format);
      if (format === "svg") {
        const svg = engine.toSvg();
        const encoded = `data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(svg)))}`;
        await bridge.exportImage(path, "svg", encoded);
      } else {
        await bridge.exportImage(path, format, dataUrl ?? "");
      }
      window.alert(`已导出 ${format.toUpperCase()}`);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <header className="menu-bar" data-testid="menu-bar">
      <div className="menu-bar__brand">PAINT</div>
      <nav className="menu-bar__menus" aria-label="主菜单">
        <button type="button" className="menu-bar__menu" onClick={handleNew}>
          新建
        </button>
        <button type="button" className="menu-bar__menu" onClick={() => void handleOpen()}>
          打开
        </button>
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void handleSave()}
          disabled={!document}
        >
          保存
        </button>
        <span className="menu-bar__divider" aria-hidden="true" />
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void handleExport("png")}
          disabled={!document}
        >
          导出 PNG
        </button>
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void handleExport("jpg")}
          disabled={!document}
        >
          导出 JPG
        </button>
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void handleExport("svg")}
          disabled={!document}
        >
          导出 SVG
        </button>
        <span className="menu-bar__divider" aria-hidden="true" />
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void undo()}
          disabled={!history?.canUndo}
        >
          撤销
        </button>
        <button
          type="button"
          className="menu-bar__menu"
          onClick={() => void redo()}
          disabled={!history?.canRedo}
        >
          重做
        </button>
      </nav>
      <div className="menu-bar__meta">
        {document ? `${document.name} · ${document.layers.length} 图层` : "未打开画布"}
      </div>
    </header>
  );
}
