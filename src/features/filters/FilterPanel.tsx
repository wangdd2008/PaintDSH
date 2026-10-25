import { activeLayer, usePaintStore } from "../../stores/paint";
import { describeFilter, DEFAULT_SPECS } from "../canvas/filters";
import type { FilterSpec } from "../../lib/types";
import { emptySvg } from "../canvas/svg";

/**
 * 滤镜面板：只对 raster 图层直接生效；矢量图层会先提示栅格化。
 *
 * 滤镜是声明式元数据，真正的像素效果在 `engine.renderDocument` 里按 spec 应用。
 */
export function FilterPanel() {
  const document = usePaintStore((state) => state.document);
  const dispatchAction = usePaintStore((state) => state.dispatchAction);
  const setError = usePaintStore((state) => state.setError);
  const layer = activeLayer(document);

  if (!layer) {
    return (
      <section className="filter-panel" data-testid="filter-panel" aria-label="滤镜">
        <header className="panel-section__header">
          <h2>滤镜</h2>
        </header>
        <p className="panel-section__empty">先选中一个图层</p>
      </section>
    );
  }

  const apply = (spec: FilterSpec) => {
    if (layer.content.type !== "raster") {
      setError("滤镜只能应用于位图图层：请先对该图层执行栅格化（导出再导入）");
      return;
    }
    setError(null);
    void dispatchAction("apply_filter", { layerId: layer.id, filter: spec });
  };

  return (
    <section className="filter-panel" data-testid="filter-panel" aria-label="滤镜">
      <header className="panel-section__header">
        <h2>滤镜</h2>
        <button
          type="button"
          onClick={() => void dispatchAction("clear_filters", { layerId: layer.id })}
          disabled={layer.metadata.filters.length === 0}
        >
          清空
        </button>
      </header>

      <div className="filter-panel__presets">
        <button type="button" onClick={() => apply(DEFAULT_SPECS.blur)}>
          模糊
        </button>
        <button type="button" onClick={() => apply(DEFAULT_SPECS.sharpen)}>
          锐化
        </button>
        <button type="button" onClick={() => apply(DEFAULT_SPECS.adjustColor)}>
          调色
        </button>
      </div>

      <ul className="filter-panel__stack" data-testid="filter-stack">
        {layer.metadata.filters.length === 0 ? (
          <li className="panel-section__empty">还没有滤镜</li>
        ) : (
          layer.metadata.filters.map((spec, index) => (
            <li key={`${spec.kind}-${index}`}>{describeFilter(spec)}</li>
          ))
        )}
      </ul>

      <p className="filter-panel__hint">
        图层内容：{layer.content.type === "raster" ? "位图" : "矢量（需栅格化）"}
        {layer.content.type === "vector" || layer.content.type === "aiCode"
          ? ` · ${emptySvg().length} 字节空模板`
          : ""}
      </p>
    </section>
  );
}
