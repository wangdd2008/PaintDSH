// 全局命令面板（Raycast/Alfred 风格）
function CommandPalette({ isOpen, onClose, onNavigate }) {
  const [query, setQuery] = React.useState('');
  const [selectedIndex, setSelectedIndex] = React.useState(0);
  const inputRef = React.useRef(null);

  const commands = [
    { id: 'dashboard', name: '工作台首页', desc: '查看系统概览和今日任务', icon: 'dashboard', category: '导航' },
    { id: 'code', name: '代码编辑器', desc: '打开代码编辑器', icon: 'code', category: '导航' },
    { id: 'terminal', name: '终端', desc: '打开命令行终端', icon: 'terminal', category: '导航' },
    { id: 'ai', name: 'AI 助手', desc: '智能代码助手', icon: 'ai', category: '导航' },
    { id: 'monitor', name: '系统监控', desc: '查看 CPU、内存、网络等指标', icon: 'monitor', category: '导航' },
    { id: 'project', name: '项目与 Git', desc: '管理项目和版本控制', icon: 'project', category: '导航' },
    { id: 'api', name: 'API 调试', desc: 'HTTP 请求调试工具', icon: 'api', category: '导航' },
    { id: 'knowledge', name: '知识库', desc: '笔记和代码片段', icon: 'knowledge', category: '导航' },
    { id: 'settings', name: '设置', desc: '应用偏好配置', icon: 'settings', category: '导航' },
    { id: 'gen-code', name: 'AI: 生成代码', desc: '根据描述生成代码', icon: 'sparkles', category: 'AI 指令' },
    { id: 'explain-code', name: 'AI: 解释代码', desc: '解释当前代码的含义', icon: 'doc', category: 'AI 指令' },
    { id: 'fix-bug', name: 'AI: 修复 Bug', desc: '检测并修复代码问题', icon: 'bug', category: 'AI 指令' },
    { id: 'optimize', name: 'AI: 优化代码', desc: '性能优化建议', icon: 'zap', category: 'AI 指令' },
    { id: 'write-test', name: 'AI: 写测试', desc: '生成单元测试用例', icon: 'test', category: 'AI 指令' },
    { id: 'new-terminal', name: '新建终端标签', desc: '打开一个新的终端窗口', icon: 'plus', category: '操作' },
    { id: 'split-screen', name: '分屏显示', desc: '左右分屏显示两个编辑器', icon: 'split', category: '操作' },
    { id: 'toggle-sidebar', name: '切换侧边栏', desc: '显示或隐藏左侧边栏', icon: 'folder', category: '操作' },
    { id: 'git-commit', name: 'Git 提交', desc: '提交当前更改', icon: 'gitBranch', category: 'Git' },
    { id: 'git-pull', name: 'Git 拉取', desc: '从远程仓库拉取更新', icon: 'cloud', category: 'Git' },
    { id: 'git-push', name: 'Git 推送', desc: '推送到远程仓库', icon: 'cloud', category: 'Git' },
  ];

  const filtered = commands.filter(c =>
    c.name.toLowerCase().includes(query.toLowerCase()) ||
    c.desc.toLowerCase().includes(query.toLowerCase())
  );

  React.useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  React.useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = (e) => {
    if (e.key === 'Escape') {
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter') {
      if (filtered[selectedIndex]) {
        handleSelect(filtered[selectedIndex]);
      }
    }
  };

  const handleSelect = (cmd) => {
    if (['dashboard', 'code', 'terminal', 'ai', 'monitor', 'project', 'api', 'knowledge'].includes(cmd.id)) {
      onNavigate(cmd.id);
    }
    onClose();
  };

  if (!isOpen) return null;

  // 按类别分组
  const grouped = {};
  filtered.forEach(cmd => {
    if (!grouped[cmd.category]) grouped[cmd.category] = [];
    grouped[cmd.category].push(cmd);
  });

  let globalIndex = -1;

  return (
    <div
      style={{
        position: 'fixed', inset: 0, zIndex: 1000,
        background: 'rgba(0,0,0,0.4)',
        backdropFilter: 'blur(8px)',
        WebkitBackdropFilter: 'blur(8px)',
        display: 'flex', alignItems: 'flex-start', justifyContent: 'center',
        paddingTop: '15vh',
      }}
      onClick={onClose}
      className="fade-in"
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          width: 600, maxWidth: '90vw',
          background: 'rgba(25,28,48,0.95)',
          backdropFilter: 'blur(40px) saturate(180%)',
          WebkitBackdropFilter: 'blur(40px) saturate(180%)',
          border: '1px solid rgba(255,255,255,0.1)',
          borderRadius: 14,
          boxShadow: '0 24px 80px rgba(0,0,0,0.5)',
          overflow: 'hidden',
        }}
      >
        {/* 搜索输入 */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 12,
          padding: '14px 16px',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
        }}>
          <Icon name="search" size={18} color="rgba(230,232,239,0.4)" />
          <input
            ref={inputRef}
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入命令或搜索..."
            style={{
              flex: 1, fontSize: 15,
              background: 'transparent', border: 'none', outline: 'none',
              color: '#e6e8ef',
            }}
          />
          <div style={{
            padding: '3px 8px', fontSize: 10,
            background: 'rgba(255,255,255,0.08)',
            borderRadius: 4, color: 'rgba(230,232,239,0.4)',
          }}>ESC</div>
        </div>

        {/* 命令列表 */}
        <div style={{ maxHeight: '50vh', overflow: 'auto', padding: '8px 0' }}>
          {Object.keys(grouped).length === 0 && (
            <div style={{
              padding: 40, textAlign: 'center',
              color: 'rgba(230,232,239,0.4)', fontSize: 13,
            }}>
              没有找到匹配的命令
            </div>
          )}
          {Object.entries(grouped).map(([category, cmds]) => (
            <div key={category}>
              <div style={{
                padding: '8px 16px 4px',
                fontSize: 10, fontWeight: 600,
                color: 'rgba(230,232,239,0.35)',
                textTransform: 'uppercase', letterSpacing: 0.5,
              }}>{category}</div>
              {cmds.map(cmd => {
                globalIndex++;
                const isSel = globalIndex === selectedIndex;
                return (
                  <div
                    key={cmd.id}
                    onClick={() => handleSelect(cmd)}
                    style={{
                      display: 'flex', alignItems: 'center', gap: 12,
                      padding: '8px 16px',
                      cursor: 'pointer',
                      background: isSel ? 'rgba(10,132,255,0.2)' : 'transparent',
                    }}
                    onMouseEnter={() => setSelectedIndex(globalIndex)}
                  >
                    <div style={{
                      width: 32, height: 32, borderRadius: 8,
                      background: isSel ? 'rgba(10,132,255,0.3)' : 'rgba(255,255,255,0.06)',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                      flexShrink: 0,
                    }}>
                      <Icon name={cmd.icon} size={16} color={isSel ? '#0A84FF' : 'rgba(230,232,239,0.5)'} />
                    </div>
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: 13, fontWeight: 500 }}>{cmd.name}</div>
                      <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.4)' }}>{cmd.desc}</div>
                    </div>
                    {isSel && (
                      <div style={{
                        display: 'flex', gap: 4, alignItems: 'center',
                        fontSize: 10, color: 'rgba(230,232,239,0.4)',
                      }}>
                        <span style={{
                          padding: '2px 6px', borderRadius: 4,
                          background: 'rgba(255,255,255,0.08)',
                        }}>↵</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
        </div>

        {/* 底部提示 */}
        <div style={{
          padding: '8px 16px', fontSize: 11,
          borderTop: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', gap: 16,
          color: 'rgba(230,232,239,0.35)',
          background: 'rgba(0,0,0,0.15)',
        }}>
          <span>↑↓ 选择</span>
          <span>↵ 执行</span>
          <span>esc 关闭</span>
          <div style={{ flex: 1 }} />
          <span style={{ color: 'rgba(230,232,239,0.25)' }}>⌘K 打开</span>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { CommandPalette });
