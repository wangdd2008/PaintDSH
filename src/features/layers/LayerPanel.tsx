import { activeLayer, usePaintStore } from "../../stores/paint";

/** 右侧图层面板：创建/删除/排序/可见性/透明度，全部经 dispatch。 */
export function LayerPanel() {
  const document = usePaintStore((state) => state.document);
  const dispatchAction = usePaintStore((state) => state.dispatchAction);
  const active = activeLayer(document);

  if (!document) {
    return (
      <aside className="layer-panel" data-testid="layer-panel" aria-label="图层与属性面板">
        <section className="panel-section">
          <header className="panel-section__header">
            <h2>图层</h2>
            <span className="panel-section__badge">0</span>
          </header>
          <p className="panel-section__empty">暂无图层</p>
        </section>
      </aside>
    );
  }

  // 展示自上而下：数组末尾是最上层
  const layers = [...document.layers].reverse();

  return (
    <aside className="layer-panel" data-testid="layer-panel" aria-label="图层与属性面板">
      <section className="panel-section">
        <header className="panel-section__header">
          <h2>图层</h2>
          <span className="panel-section__badge">{document.layers.length}</span>
          <button
            type="button"
            className="panel-section__action"
            onClick={() =>
              void dispatchAction("create_layer", {
                name: `图层 ${document.layers.length + 1}`,
              })
            }
          >
            ＋ 新建
          </button>
        </header>

        <ul className="layer-list" data-testid="layer-list">
          {layers.map((layer) => {
            const index = document.layers.findIndex((item) => item.id === layer.id);
            const isActive = layer.id === active?.id;
            return (
              <li
                key={layer.id}
                className={`layer-item${isActive ? " layer-item--active" : ""}`}
                data-testid="layer-item"
              >
                <button
                  type="button"
                  className="layer-item__visibility"
                  aria-label={layer.visible ? "隐藏图层" : "显示图层"}
                  title={layer.visible ? "隐藏" : "显示"}
                  onClick={() =>
                    void dispatchAction("set_layer_visibility", {
                      layerId: layer.id,
                      visible: !layer.visible,
                    })
                  }
                >
                  {layer.visible ? "●" : "○"}
                </button>
                <button
                  type="button"
                  className="layer-item__name"
                  onClick={() => void dispatchAction("select_layer", { layerId: layer.id })}
                  title={layer.type}
                >
                  {layer.name}
                  {layer.metadata.createdBy === "agent" ? (
                    <span className="layer-item__badge">AI</span>
                  ) : null}
                </button>
                <span className="layer-item__actions">
                  <button
                    type="button"
                    aria-label="上移"
                    disabled={index === document.layers.length - 1}
                    onClick={() =>
                      void dispatchAction("reorder_layer", {
                        layerId: layer.id,
                        toIndex: index + 1,
                      })
                    }
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    aria-label="下移"
                    disabled={index === 0}
                    onClick={() =>
                      void dispatchAction("reorder_layer", {
                        layerId: layer.id,
                        toIndex: index - 1,
                      })
                    }
                  >
                    ↓
                  </button>
                  <button
                    type="button"
                    aria-label="重命名"
                    onClick={() => {
                      const next = window.prompt("图层名", layer.name);
                      if (next?.trim()) {
                        void dispatchAction("set_layer_property", {
                          layerId: layer.id,
                          name: next.trim(),
                        });
                      }
                    }}
                  >
                    ✎
                  </button>
                  <button
                    type="button"
                    aria-label="删除图层"
                    disabled={document.layers.length <= 1}
                    onClick={() =>
                      void dispatchAction("delete_layer", { layerId: layer.id })
                    }
                  >
                    ✕
                  </button>
                </span>
              </li>
            );
          })}
        </ul>
      </section>

      <section className="panel-section panel-section--grow">
        <header className="panel-section__header">
          <h2>属性</h2>
        </header>
        {active ? (
          <div className="property-grid">
            <label>
              透明度
              <input
                type="range"
                min={0}
                max={1}
                step={0.05}
                value={active.opacity}
                onChange={(event) =>
                  void dispatchAction("set_layer_property", {
                    layerId: active.id,
                    opacity: Number(event.target.value),
                  })
                }
              />
              <span>{Math.round(active.opacity * 100)}%</span>
            </label>
            <label>
              混合模式
              <select
                value={active.blendMode}
                onChange={(event) =>
                  void dispatchAction("set_layer_property", {
                    layerId: active.id,
                    blendMode: event.target.value,
                  })
                }
              >
                <option value="normal">正常</option>
                <option value="multiply">正片叠底</option>
                <option value="screen">滤色</option>
                <option value="overlay">叠加</option>
              </select>
            </label>
            <label className="property-grid__inline">
              <input
                type="checkbox"
                checked={active.locked}
                onChange={(event) =>
                  void dispatchAction("set_layer_property", {
                    layerId: active.id,
                    locked: event.target.checked,
                  })
                }
              />
              锁定图层
            </label>
            <p className="property-grid__meta">
              类型 {active.type} · 修订 {document.revision}
            </p>
          </div>
        ) : (
          <p className="panel-section__empty">未选中图层</p>
        )}
      </section>
    </aside>
  );
}
