import { useEffect, useState } from "react";

import { ChatPanel } from "./features/ai/ChatPanel";
import { EmployeePanel } from "./features/ai/EmployeePanel";
import { HarnessView } from "./features/ai/HarnessView";
import { CanvasStage } from "./features/canvas/CanvasStage";
import { FilterPanel } from "./features/filters/FilterPanel";
import { KnowledgePanel } from "./features/knowledge/KnowledgePanel";
import { LayerPanel } from "./features/layers/LayerPanel";
import { MenuBar } from "./features/menu/MenuBar";
import { StatusBar } from "./features/status/StatusBar";
import { Toolbox } from "./features/toolbox/Toolbox";
import { ping } from "./lib/bridge";
import { usePaintStore } from "./stores/paint";
import "./app.css";

type SidePanelId = "layers" | "filters" | "ai" | "employees" | "knowledge" | "harness";

const SIDE_PANELS: { id: SidePanelId; label: string }[] = [
  { id: "layers", label: "图层" },
  { id: "filters", label: "滤镜" },
  { id: "ai", label: "AI" },
  { id: "employees", label: "员工" },
  { id: "knowledge", label: "知识库" },
  { id: "harness", label: "工作台" },
];

/** 三栏骨架：菜单栏 / 工具栏 + 画布 + 右侧面板（图层/滤镜/AI/员工/知识库/工作台）/ 状态栏。 */
export function App() {
  const setBackendConnected = usePaintStore((state) => state.setBackendConnected);
  const setBackendDisconnected = usePaintStore((state) => state.setBackendDisconnected);
  const bootstrap = usePaintStore((state) => state.bootstrap);
  const setDocument = usePaintStore((state) => state.setDocument);
  const [panel, setPanel] = useState<SidePanelId>("layers");

  useEffect(() => {
    let cancelled = false;

    ping()
      .then((response) => {
        if (cancelled) {
          return undefined;
        }
        setBackendConnected(response.version);
        return bootstrap();
      })
      .catch(() => {
        if (!cancelled) {
          setBackendDisconnected();
          // 没有后端时给一个本地空白文档，界面仍可浏览（保存等操作会报错）。
          setDocument(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [setBackendConnected, setBackendDisconnected, bootstrap, setDocument]);

  return (
    <div className="app-shell">
      <MenuBar />
      <div className="app-body">
        <Toolbox />
        <main className="canvas-area" data-testid="canvas-area">
          <CanvasStage />
        </main>
        <aside className="side-panel" data-testid="side-panel">
          <nav className="side-panel__tabs" aria-label="右侧面板切换">
            {SIDE_PANELS.map((item) => (
              <button
                key={item.id}
                type="button"
                className={panel === item.id ? "is-active" : undefined}
                aria-pressed={panel === item.id}
                onClick={() => setPanel(item.id)}
              >
                {item.label}
              </button>
            ))}
          </nav>
          <div className="side-panel__body">
            {panel === "layers" ? <LayerPanel /> : null}
            {panel === "filters" ? <FilterPanel /> : null}
            {panel === "ai" ? <ChatPanel /> : null}
            {panel === "employees" ? <EmployeePanel /> : null}
            {panel === "knowledge" ? <KnowledgePanel /> : null}
            {panel === "harness" ? <HarnessView /> : null}
          </div>
        </aside>
      </div>
      <StatusBar />
    </div>
  );
}
