import { useEffect } from "react";

import { CanvasStage } from "./features/canvas/CanvasStage";
import { LayerPanel } from "./features/layers/LayerPanel";
import { MenuBar } from "./features/menu/MenuBar";
import { StatusBar } from "./features/status/StatusBar";
import { Toolbox } from "./features/toolbox/Toolbox";
import { ping } from "./lib/bridge";
import { usePaintStore } from "./stores/paint";
import "./app.css";

/** 三栏骨架：菜单栏 / 工具栏 + 画布 + 图层属性面板 / 状态栏。 */
export function App() {
  const setBackendConnected = usePaintStore((state) => state.setBackendConnected);
  const setBackendDisconnected = usePaintStore((state) => state.setBackendDisconnected);

  useEffect(() => {
    let cancelled = false;

    ping()
      .then((response) => {
        if (!cancelled) {
          setBackendConnected(response.version);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setBackendDisconnected();
        }
      });

    return () => {
      cancelled = true;
    };
  }, [setBackendConnected, setBackendDisconnected]);

  return (
    <div className="app-shell">
      <MenuBar />
      <div className="app-body">
        <Toolbox />
        <main className="canvas-area" data-testid="canvas-area">
          <CanvasStage />
        </main>
        <LayerPanel />
      </div>
      <StatusBar />
    </div>
  );
}
