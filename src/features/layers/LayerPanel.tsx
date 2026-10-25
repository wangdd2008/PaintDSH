/** 右侧图层面板 + 属性面板占位；图层数据随 ticket 02/03 接入。 */
export function LayerPanel() {
  return (
    <aside className="layer-panel" data-testid="layer-panel" aria-label="图层与属性面板">
      <section className="panel-section">
        <header className="panel-section__header">
          <h2>图层</h2>
          <span className="panel-section__badge">0</span>
        </header>
        <p className="panel-section__empty">暂无图层</p>
      </section>
      <section className="panel-section panel-section--grow">
        <header className="panel-section__header">
          <h2>属性</h2>
        </header>
        <p className="panel-section__empty">未选中图层</p>
      </section>
    </aside>
  );
}
