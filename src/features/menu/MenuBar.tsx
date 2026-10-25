const MENUS = ["文件", "编辑", "视图", "帮助"] as const;

/** 顶部菜单栏。菜单项在后续工单接线（文件/编辑/视图命令）。 */
export function MenuBar() {
  return (
    <header className="menu-bar" data-testid="menu-bar">
      <div className="menu-bar__brand">PAINT</div>
      <nav className="menu-bar__menus" aria-label="主菜单">
        {MENUS.map((menu) => (
          <button key={menu} type="button" className="menu-bar__menu">
            {menu}
          </button>
        ))}
      </nav>
      <div className="menu-bar__meta">Tauri 2 · Fabric.js</div>
    </header>
  );
}
