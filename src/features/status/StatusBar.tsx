import { usePaintStore } from "../../stores/paint";

/** 底部状态栏：后端连通性 + 画布尺寸 + 修订 + 当前工具 + 错误提示。 */
export function StatusBar() {
  const backend = usePaintStore((state) => state.backend);
  const document = usePaintStore((state) => state.document);
  const tool = usePaintStore((state) => state.tool);
  const error = usePaintStore((state) => state.error);

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
      {error ? (
        <span className="status-bar__error" role="alert" data-testid="status-error">
          {error}
        </span>
      ) : null}
      <span className="status-bar__spacer" />
      <span className="status-bar__item">
        {document ? `${document.width} × ${document.height} px` : "—"}
      </span>
      <span className="status-bar__item">
        {document ? `修订 ${document.revision}` : "无画布"}
      </span>
      <span className="status-bar__item">工具：{toolLabel(tool)}</span>
    </footer>
  );
}

function toolLabel(tool: string): string {
  switch (tool) {
    case "brush":
      return "画笔";
    case "eraser":
      return "橡皮";
    case "shape":
      return "形状";
    case "text":
      return "文字";
    default:
      return "选择";
  }
}
