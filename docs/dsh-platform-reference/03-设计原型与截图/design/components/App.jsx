// 应用主框架
function App() {
  const [activeNav, setActiveNav] = React.useState('dashboard');
  const [rightPanelOpen, setRightPanelOpen] = React.useState(true);
  const [rightPanelMode, setRightPanelMode] = React.useState('ai'); // 'ai' | 'monitor'
  const [commandPaletteOpen, setCommandPaletteOpen] = React.useState(false);
  const [currentTime, setCurrentTime] = React.useState(new Date());
  const [metrics, setMetrics] = React.useState({ cpu: 38.5, mem: 62.3, disk: 45, net: 28 });

  // 全局时间更新
  React.useEffect(() => {
    const t = setInterval(() => setCurrentTime(new Date()), 1000);
    return () => clearInterval(t);
  }, []);

  // 底部状态栏指标波动
  React.useEffect(() => {
    const t = setInterval(() => {
      setMetrics(prev => ({
        cpu: Math.max(10, Math.min(90, prev.cpu + (Math.random() - 0.5) * 5)),
        mem: Math.max(45, Math.min(80, prev.mem + (Math.random() - 0.5) * 2)),
        disk: Math.max(20, Math.min(70, prev.disk + (Math.random() - 0.5) * 8)),
        net: Math.max(5, Math.min(80, prev.net + (Math.random() - 0.5) * 10)),
      }));
    }, 2000);
    return () => clearInterval(t);
  }, []);

  // 快捷键
  React.useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const navItems = [
    { id: 'dashboard', name: '工作台', icon: 'dashboard' },
    { id: 'code', name: '代码', icon: 'code' },
    { id: 'terminal', name: '终端', icon: 'terminal' },
    { id: 'ai', name: 'AI助手', icon: 'ai' },
    { id: 'monitor', name: '监控', icon: 'monitor' },
    { id: 'project', name: '项目', icon: 'project' },
    { id: 'knowledge', name: '知识库', icon: 'knowledge' },
    { id: 'api', name: 'API调试', icon: 'api' },
    { id: 'settings', name: '设置', icon: 'settings' },
  ];

  const renderContent = () => {
    switch (activeNav) {
      case 'dashboard': return <Dashboard onNavigate={setActiveNav} />;
      case 'code': return <CodeEditor />;
      case 'terminal': return <TerminalModule />;
      case 'ai': return <AIAssistant />;
      case 'monitor': return <SystemMonitor />;
      case 'project': return <ProjectGit />;
      case 'api': return <APIDebug />;
      case 'knowledge': return <KnowledgeBase />;
      case 'settings': return <SettingsPage />;
      default: return <Dashboard onNavigate={setActiveNav} />;
    }
  };

  const timeStr = currentTime.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  const dateStr = currentTime.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', weekday: 'short' });

  return (
    <div style={{
      width: '100%', height: '100%',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      padding: 20,
    }}>
      {/* 主窗口 */}
      <div className="glass" style={{
        width: '100%', height: '100%', maxWidth: 1600, maxHeight: 900,
        borderRadius: 14, overflow: 'hidden',
        display: 'flex', flexDirection: 'column',
        boxShadow: '0 24px 80px rgba(0,0,0,0.5), 0 0 0 1px rgba(255,255,255,0.05)',
      }}>
        {/* 标题栏 */}
        <div style={{
          height: 38, flexShrink: 0,
          display: 'flex', alignItems: 'center',
          background: 'rgba(15,17,28,0.8)',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          padding: '0 12px',
          WebkitAppRegion: 'drag',
        }}>
          {/* 红绿灯 */}
          <div style={{ display: 'flex', gap: 8, width: 60, WebkitAppRegion: 'no-drag' }}>
            <div style={trafficLightStyle('#ff5f57', '#e0443e')} />
            <div style={trafficLightStyle('#ffbd2e', '#dea123')} />
            <div style={trafficLightStyle('#28c940', '#1aab29')} />
          </div>

          {/* 全局搜索栏 */}
          <div style={{ flex: 1, display: 'flex', justifyContent: 'center', padding: '0 20px' }}>
            <div
              onClick={() => setCommandPaletteOpen(true)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                width: '100%', maxWidth: 480,
                padding: '5px 14px',
                background: 'rgba(255,255,255,0.06)',
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: 8,
                cursor: 'pointer',
                WebkitAppRegion: 'no-drag',
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.09)';
                e.currentTarget.style.borderColor = 'rgba(255,255,255,0.12)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.06)';
                e.currentTarget.style.borderColor = 'rgba(255,255,255,0.08)';
              }}
            >
              <Icon name="search" size={13} color="rgba(230,232,239,0.4)" />
              <span style={{ fontSize: 12, color: 'rgba(230,232,239,0.4)', flex: 1 }}>搜索命令、文件、AI 对话... (⌘K)</span>
              <div style={{
                padding: '2px 6px', fontSize: 10,
                background: 'rgba(255,255,255,0.08)',
                borderRadius: 4, color: 'rgba(230,232,239,0.35)',
              }}>⌘K</div>
            </div>
          </div>

          {/* 当前项目 + 快捷操作 */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, WebkitAppRegion: 'no-drag' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '0 10px' }}>
              <div style={{
                width: 18, height: 18, borderRadius: 4,
                background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
              }} />
              <span style={{ fontSize: 12, fontWeight: 500 }}>auth-service</span>
              <div style={{
                display: 'flex', alignItems: 'center', gap: 3,
                padding: '2px 8px', fontSize: 10,
                background: 'rgba(231,111,81,0.15)',
                borderRadius: 10, color: '#E76F51',
              }}>
                <Icon name="gitBranch" size={10} />
                feature/oauth2
              </div>
            </div>
            <div style={{ width: 1, height: 16, background: 'rgba(255,255,255,0.08)' }} />
            <div style={{ display: 'flex', gap: 2 }}>
              <HeaderBtn icon="bell" />
              <HeaderBtn icon="maximize" />
              <HeaderBtn icon="user" />
            </div>
          </div>
        </div>

        {/* 主体区 */}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {/* 左侧导航 */}
          <div style={{
            width: 60, flexShrink: 0,
            background: 'rgba(15,17,28,0.6)',
            borderRight: '1px solid rgba(255,255,255,0.06)',
            display: 'flex', flexDirection: 'column',
            alignItems: 'center',
            padding: '14px 0',
            gap: 4,
          }}>
            {/* Logo */}
            <div style={{
              width: 36, height: 36, borderRadius: 10, marginBottom: 10,
              background: 'linear-gradient(135deg, #0A84FF 0%, #BF5AF2 100%)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              boxShadow: '0 4px 12px rgba(10,132,255,0.4)',
            }}>
              <Icon name="zap" size={18} color="#fff" />
            </div>

            {navItems.map((item, i) => {
              if (i === 5 || i === 8) {
                return (
                  <React.Fragment key={item.id}>
                    <div style={{ width: 24, height: 1, background: 'rgba(255,255,255,0.06)', margin: '8px 0' }} />
                    <NavItem item={item} active={activeNav === item.id} onClick={() => setActiveNav(item.id)} />
                  </React.Fragment>
                );
              }
              return <NavItem key={item.id} item={item} active={activeNav === item.id} onClick={() => setActiveNav(item.id)} />;
            })}

            <div style={{ flex: 1 }} />

            {/* 底部快捷按钮 */}
            <div style={{
              width: 36, height: 36, borderRadius: 10,
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              cursor: 'pointer',
              color: 'rgba(230,232,239,0.5)',
            }}>
              <div style={{
                width: 8, height: 8, borderRadius: '50%',
                background: '#30D158',
                boxShadow: '0 0 8px rgba(48,209,88,0.5)',
              }} />
            </div>
          </div>

          {/* 中央主工作区 */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', position: 'relative' }}>
            <div style={{ flex: 1, overflow: 'hidden' }}>
              {renderContent()}
            </div>
          </div>

          {/* 右侧面板 */}
          {rightPanelOpen && (
            <div style={{
              width: 320, flexShrink: 0,
              background: 'rgba(15,17,28,0.5)',
              borderLeft: '1px solid rgba(255,255,255,0.06)',
              display: 'flex', flexDirection: 'column',
            }} className="fade-in">
              {/* 面板头部 + 切换 */}
              <div style={{
                padding: '8px 12px',
                borderBottom: '1px solid rgba(255,255,255,0.06)',
                display: 'flex', alignItems: 'center', gap: 6,
              }}>
                <div style={{
                  flex: 1, display: 'flex', gap: 4,
                  background: 'rgba(255,255,255,0.06)',
                  borderRadius: 8, padding: 3,
                }}>
                  <button
                    onClick={() => setRightPanelMode('ai')}
                    style={{
                      flex: 1, padding: '5px 0',
                      fontSize: 11, fontWeight: 500,
                      background: rightPanelMode === 'ai' ? 'rgba(255,255,255,0.1)' : 'transparent',
                      border: 'none', borderRadius: 6,
                      color: rightPanelMode === 'ai' ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                      cursor: 'pointer',
                      display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 5,
                    }}
                  >
                    <Icon name="sparkles" size={12} />
                    AI
                  </button>
                  <button
                    onClick={() => setRightPanelMode('monitor')}
                    style={{
                      flex: 1, padding: '5px 0',
                      fontSize: 11, fontWeight: 500,
                      background: rightPanelMode === 'monitor' ? 'rgba(255,255,255,0.1)' : 'transparent',
                      border: 'none', borderRadius: 6,
                      color: rightPanelMode === 'monitor' ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                      cursor: 'pointer',
                      display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 5,
                    }}
                  >
                    <Icon name="activity" size={12} />
                    监控
                  </button>
                </div>
                <button
                  onClick={() => setRightPanelOpen(false)}
                  style={{
                    width: 26, height: 26,
                    background: 'rgba(255,255,255,0.06)',
                    border: 'none', borderRadius: 6,
                    cursor: 'pointer',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    color: 'rgba(230,232,239,0.5)',
                  }}
                >
                  <Icon name="close" size={12} />
                </button>
              </div>

              {/* 面板内容 */}
              <div style={{ flex: 1, overflow: 'hidden', padding: 10 }}>
                {rightPanelMode === 'ai' ? (
                  <AIAssistant compact />
                ) : (
                  <SystemMonitor compact />
                )}
              </div>
            </div>
          )}

          {/* 右侧面板展开按钮（收起时显示） */}
          {!rightPanelOpen && (
            <button
              onClick={() => setRightPanelOpen(true)}
              style={{
                position: 'absolute', right: 8, top: 52,
                width: 28, height: 28,
                background: 'rgba(25,28,48,0.8)',
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: 6,
                cursor: 'pointer',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                color: 'rgba(230,232,239,0.5)',
                zIndex: 10,
              }}
            >
              <Icon name="chevronLeft" size={14} />
            </button>
          )}
        </div>

        {/* 底部状态栏 */}
        <div style={{
          height: 24, flexShrink: 0,
          background: 'rgba(10,12,22,0.9)',
          borderTop: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', alignItems: 'center',
          padding: '0 12px',
          fontSize: 10.5,
          color: 'rgba(230,232,239,0.5)',
        }}>
          {/* 左侧：系统指标 */}
          <div style={{ display: 'flex', gap: 16, alignItems: 'center' }}>
            <StatusMetric icon="cpu" label="CPU" value={`${metrics.cpu.toFixed(1)}%`} color="#0A84FF" />
            <StatusMetric icon="memory" label="MEM" value={`${metrics.mem.toFixed(1)}%`} color="#BF5AF2" />
            <StatusMetric icon="hardDrive" label="DISK" value={`${metrics.disk.toFixed(0)} MB/s`} color="#30D158" />
            <StatusMetric icon="wifi" label="NET" value={`${metrics.net.toFixed(1)} MB/s`} color="#FF9F0A" />
          </div>

          <div style={{ flex: 1 }} />

          {/* 中间：Git 信息 */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <Icon name="gitBranch" size={11} color="#E76F51" />
            <span>feature/oauth2</span>
            <span style={{ color: 'rgba(230,232,239,0.25)' }}>·</span>
            <span style={{ color: '#30D158' }}>↑1</span>
            <span style={{ color: '#FF6B6B' }}>↓0</span>
            <span style={{ color: 'rgba(230,232,239,0.25)' }}>·</span>
            <span>0 errors, 2 warnings</span>
          </div>

          <div style={{ flex: 1 }} />

          {/* 右侧：时间 + 电池 */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
              <Icon name="battery" size={13} color="#30D158" />
              <span>85%</span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
              <Icon name="zap" size={11} color="#FFD60A" />
              <span>低功耗</span>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', lineHeight: 1.1 }}>
              <span style={{ fontWeight: 500, color: 'rgba(230,232,239,0.7)' }}>{timeStr}</span>
              <span style={{ fontSize: 9.5 }}>{dateStr}</span>
            </div>
          </div>
        </div>
      </div>

      {/* 命令面板 */}
      <CommandPalette
        isOpen={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onNavigate={setActiveNav}
      />

      {/* spin animation */}
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

function NavItem({ item, active, onClick }) {
  return (
    <div
      onClick={onClick}
      title={item.name}
      style={{
        width: 42, height: 42, borderRadius: 10,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        cursor: 'pointer',
        position: 'relative',
        background: active ? 'rgba(10,132,255,0.2)' : 'transparent',
        color: active ? '#0A84FF' : 'rgba(230,232,239,0.5)',
        transition: 'all 0.2s',
      }}
      onMouseEnter={(e) => {
        if (!active) {
          e.currentTarget.style.background = 'rgba(255,255,255,0.06)';
          e.currentTarget.style.color = 'rgba(230,232,239,0.8)';
        }
      }}
      onMouseLeave={(e) => {
        if (!active) {
          e.currentTarget.style.background = 'transparent';
          e.currentTarget.style.color = 'rgba(230,232,239,0.5)';
        }
      }}
    >
      {active && (
        <div style={{
          position: 'absolute', left: -9, top: '50%',
          transform: 'translateY(-50%)',
          width: 3, height: 20, borderRadius: '0 2px 2px 0',
          background: '#0A84FF',
        }} />
      )}
      <Icon name={item.icon} size={18} />
    </div>
  );
}

function HeaderBtn({ icon }) {
  const [hover, setHover] = React.useState(false);
  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      style={{
        width: 28, height: 28, borderRadius: 6,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        cursor: 'pointer',
        background: hover ? 'rgba(255,255,255,0.08)' : 'transparent',
        color: hover ? 'rgba(230,232,239,0.8)' : 'rgba(230,232,239,0.4)',
        transition: 'all 0.15s',
      }}
    >
      <Icon name={icon} size={15} />
    </div>
  );
}

function StatusMetric({ icon, label, value, color }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
      <Icon name={icon} size={11} color={color} />
      <span style={{ color: 'rgba(230,232,239,0.35)' }}>{label}</span>
      <span className="mono" style={{ color, fontWeight: 500 }}>{value}</span>
    </div>
  );
}

const trafficLightStyle = (color, shadowColor) => ({
  width: 12, height: 12, borderRadius: '50%',
  background: color,
  boxShadow: `inset 0 0.5px 0 rgba(255,255,255,0.3), 0 0.5px 1px ${shadowColor}50`,
  cursor: 'pointer',
  position: 'relative',
});

// 设置页（简单实现）
function SettingsPage() {
  return (
    <div style={{ padding: 32, height: '100%', overflow: 'auto' }}>
      <h2 style={{ fontSize: 20, fontWeight: 700, marginBottom: 20 }}>设置</h2>
      <div style={{ display: 'grid', gridTemplateColumns: '200px 1fr', gap: 24 }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {['通用', '外观', '快捷键', 'AI 助手', '终端', 'Git', '插件', '关于'].map((item, i) => (
            <div key={i} style={{
              padding: '8px 12px', borderRadius: 6,
              fontSize: 13,
              background: i === 0 ? 'rgba(10,132,255,0.15)' : 'transparent',
              color: i === 0 ? '#0A84FF' : 'rgba(230,232,239,0.7)',
              cursor: 'pointer',
            }}>{item}</div>
          ))}
        </div>
        <div className="glass-light" style={{ borderRadius: 12, padding: 20 }}>
          <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 16 }}>通用设置</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            <SettingRow label="随系统启动" desc="登录时自动启动 DevStation" />
            <SettingRow label="最小化到菜单栏" desc="关闭窗口时最小化到菜单栏" />
            <SettingRow label="自动更新" desc="自动检查并安装更新" />
            <SettingRow label="启用硬件加速" desc="使用 Metal GPU 加速渲染" />
            <SettingRow label="低功耗模式" desc="降低性能以节省电量（推荐 Mac 笔记本使用）" defaultOn />
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingRow({ label, desc, defaultOn }) {
  const [on, setOn] = React.useState(defaultOn || false);
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 0' }}>
      <div>
        <div style={{ fontSize: 13, fontWeight: 500, marginBottom: 2 }}>{label}</div>
        <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.45)' }}>{desc}</div>
      </div>
      <div
        onClick={() => setOn(!on)}
        style={{
          width: 36, height: 20, borderRadius: 10,
          background: on ? '#30D158' : 'rgba(255,255,255,0.15)',
          cursor: 'pointer',
          position: 'relative',
          transition: 'background 0.2s',
        }}
      >
        <div style={{
          position: 'absolute', top: 2,
          left: on ? 18 : 2,
          width: 16, height: 16, borderRadius: '50%',
          background: '#fff',
          boxShadow: '0 1px 3px rgba(0,0,0,0.3)',
          transition: 'left 0.2s',
        }} />
      </div>
    </div>
  );
}

Object.assign(window, { App });
