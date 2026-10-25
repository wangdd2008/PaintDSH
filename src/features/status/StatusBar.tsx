import { usePaintStore } from "../../stores/paint";

/** 底部状态栏：后端连通性 + 画布尺寸 + 缩放 + 当前工具。 */
export function StatusBar() {
  const backend = usePaintStore((state) => state.backend);

  const backendLabel =
    backend.state === "connected"
      ? `后端已连接 v${backend.version ?? "?"}`
      : backend.state === "disconnected"
        ? "后端未连接"
        : "正在连接后端…";

  return (
    <footer className="status-bar" data-testid="status-bar">
      <span
        className={`status-bar__dot status-bar__dot--${backend.state}`}
        aria-hidden="true"
      />
      <span className="status-bar__item">{backendLabel}</span>
      <span className="status-bar__spacer" />
      <span className="status-bar__item">1280 × 800 px</span>
      <span className="status-bar__item">缩放 100%</span>
      <span className="status-bar__item">工具：选择</span>
    </footer>
  );
}
