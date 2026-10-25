interface Tool {
  id: string;
  label: string;
  glyph: string;
}

/** 工具表。工单 05 起接线到画布交互。 */
const TOOLS: readonly Tool[] = [
  { id: "select", label: "选择", glyph: "↖" },
  { id: "brush", label: "画笔", glyph: "✎" },
  { id: "eraser", label: "橡皮", glyph: "⌫" },
  { id: "shape", label: "形状", glyph: "▭" },
  { id: "text", label: "文字", glyph: "T" },
  { id: "fill", label: "填充", glyph: "◧" },
];

/** 左侧工具栏（图标竖排，Figma 风格）。 */
export function Toolbox() {
  return (
    <aside className="toolbox" data-testid="toolbox" aria-label="工具栏">
      <ul className="toolbox__list">
        {TOOLS.map((tool) => (
          <li key={tool.id}>
            <button
              type="button"
              className="toolbox__button"
              aria-label={tool.label}
              title={tool.label}
              aria-pressed={tool.id === "select"}
            >
              <span aria-hidden="true">{tool.glyph}</span>
            </button>
          </li>
        ))}
      </ul>
    </aside>
  );
}
