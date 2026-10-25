import { usePaintStore, type ShapeKind, type ToolId } from "../../stores/paint";

interface Tool {
  id: ToolId;
  label: string;
  glyph: string;
  hint: string;
}

/** 工具表：全部走 dispatch，人类与 AI 共用同一套 action。 */
const TOOLS: readonly Tool[] = [
  { id: "select", label: "选择", glyph: "↖", hint: "拖动图层、点选" },
  { id: "brush", label: "画笔", glyph: "✎", hint: "自由绘制" },
  { id: "eraser", label: "橡皮", glyph: "⌫", hint: "用背景色覆盖" },
  { id: "shape", label: "形状", glyph: "▭", hint: "拖出矩形/椭圆" },
  { id: "text", label: "文字", glyph: "T", hint: "点击输入文字" },
];

const SHAPES: readonly { id: ShapeKind; label: string }[] = [
  { id: "rect", label: "矩形" },
  { id: "ellipse", label: "椭圆" },
];

const COLORS = ["#1c2024", "#e5484d", "#2f6feb", "#30a46c", "#f5a524", "#8e4ec6"];

/** 左侧工具栏（图标竖排，Figma 风格）。 */
export function Toolbox() {
  const tool = usePaintStore((state) => state.tool);
  const setTool = usePaintStore((state) => state.setTool);
  const shapeKind = usePaintStore((state) => state.shapeKind);
  const setShapeKind = usePaintStore((state) => state.setShapeKind);
  const brushColor = usePaintStore((state) => state.brushColor);
  const setBrushColor = usePaintStore((state) => state.setBrushColor);
  const brushWidth = usePaintStore((state) => state.brushWidth);
  const setBrushWidth = usePaintStore((state) => state.setBrushWidth);
  const undo = usePaintStore((state) => state.undo);
  const redo = usePaintStore((state) => state.redo);
  const history = usePaintStore((state) => state.history);

  return (
    <aside className="toolbox" data-testid="toolbox" aria-label="工具栏">
      <ul className="toolbox__list">
        {TOOLS.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              className={`toolbox__button${tool === item.id ? " toolbox__button--active" : ""}`}
              aria-label={item.label}
              title={`${item.label} — ${item.hint}`}
              aria-pressed={tool === item.id}
              onClick={() => setTool(item.id)}
            >
              <span aria-hidden="true">{item.glyph}</span>
            </button>
          </li>
        ))}
      </ul>

      {tool === "shape" ? (
        <div className="toolbox__group" aria-label="形状类型">
          {SHAPES.map((shape) => (
            <button
              key={shape.id}
              type="button"
              className={`toolbox__chip${shapeKind === shape.id ? " toolbox__chip--active" : ""}`}
              onClick={() => setShapeKind(shape.id)}
            >
              {shape.label}
            </button>
          ))}
        </div>
      ) : null}

      <div className="toolbox__group" aria-label="颜色">
        {COLORS.map((color) => (
          <button
            key={color}
            type="button"
            className={`toolbox__swatch${brushColor === color ? " toolbox__swatch--active" : ""}`}
            style={{ backgroundColor: color }}
            aria-label={`颜色 ${color}`}
            onClick={() => setBrushColor(color)}
          />
        ))}
      </div>

      <label className="toolbox__width">
        笔宽
        <input
          type="range"
          min={1}
          max={32}
          value={brushWidth}
          onChange={(event) => setBrushWidth(Number(event.target.value))}
        />
        <span>{brushWidth}</span>
      </label>

      <div className="toolbox__group toolbox__group--history">
        <button type="button" onClick={() => void undo()} disabled={!history?.canUndo}>
          ↶ 撤销
        </button>
        <button type="button" onClick={() => void redo()} disabled={!history?.canRedo}>
          ↷ 重做
        </button>
      </div>
    </aside>
  );
}
