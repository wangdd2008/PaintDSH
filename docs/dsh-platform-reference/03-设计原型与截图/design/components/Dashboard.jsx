// 工作台首页 Dashboard
function Dashboard({ onNavigate }) {
  const [tasks, setTasks] = React.useState([
    { id: 1, text: '修复用户登录接口的并发问题', done: false, priority: 'high', project: 'auth-service' },
    { id: 2, text: '完成订单模块的单元测试覆盖', done: false, priority: 'med', project: 'order-service' },
    { id: 3, text: 'Code Review: PR #247 用户中心重构', done: true, priority: 'med', project: 'user-center' },
    { id: 4, text: '优化首页加载性能 < 2s', done: false, priority: 'high', project: 'web-app' },
    { id: 5, text: '部署 v2.3.1 到生产环境', done: false, priority: 'low', project: 'devops' },
  ]);

  const toggleTask = (id) => {
    setTasks(prev => prev.map(t => t.id === id ? { ...t, done: !t.done } : t));
  };

  const recentProjects = [
    { name: 'auth-service', branch: 'feature/oauth2', lastMod: '2 小时前', lang: 'Java', color: '#E76F51' },
    { name: 'web-dashboard', branch: 'main', lastMod: '昨天', lang: 'TypeScript', color: '#3178C6' },
    { name: 'order-service', branch: 'develop', lastMod: '3 天前', lang: 'Go', color: '#00ADD8' },
    { name: 'ml-pipeline', branch: 'feature/cv-model', lastMod: '1 周前', lang: 'Python', color: '#3776AB' },
  ];

  const quickActions = [
    { name: '终端', icon: 'terminal', color: '#30D158', action: 'terminal' },
    { name: '代码', icon: 'code', color: '#0A84FF', action: 'code' },
    { name: 'API调试', icon: 'api', color: '#BF5AF2', action: 'api' },
    { name: '笔记', icon: 'knowledge', color: '#FF9F0A', action: 'knowledge' },
    { name: 'Git', icon: 'gitBranch', color: '#E76F51', action: 'project' },
    { name: 'Docker', icon: 'database', color: '#2496ED', action: 'project' },
    { name: '数据库', icon: 'hardDrive', color: '#34A853', action: 'project' },
    { name: '浏览器', icon: 'globe', color: '#FBBC05', action: 'knowledge' },
  ];

  const aiQuickActions = [
    { name: '生成代码', icon: 'sparkles', desc: '快速生成模板代码', color: '#0A84FF' },
    { name: '解释代码', icon: 'doc', desc: '解释当前代码逻辑', color: '#30D158' },
    { name: '优化代码', icon: 'zap', desc: '性能与可读性优化', color: '#FFD60A' },
    { name: '写测试', icon: 'test', desc: '生成单元测试用例', color: '#BF5AF2' },
  ];

  // 实时资源指标
  const [metrics, setMetrics] = React.useState({ cpu: 38, mem: 62, disk: 45, net: 28 });
  React.useEffect(() => {
    const t = setInterval(() => {
      setMetrics(prev => ({
        cpu: Math.max(10, Math.min(90, prev.cpu + (Math.random() - 0.5) * 8)),
        mem: Math.max(40, Math.min(85, prev.mem + (Math.random() - 0.5) * 3)),
        disk: Math.max(20, Math.min(70, prev.disk + (Math.random() - 0.5) * 5)),
        net: Math.max(5, Math.min(80, prev.net + (Math.random() - 0.5) * 10)),
      }));
    }, 2000);
    return () => clearInterval(t);
  }, []);

  const metricCards = [
    { label: 'CPU 使用率', value: `${metrics.cpu.toFixed(1)}%`, sub: '8核活跃', icon: 'cpu', color: '#0A84FF', bgColor: 'rgba(10,132,255,0.1)' },
    { label: '内存压力', value: `${metrics.mem.toFixed(1)}%`, sub: '10.2 GB / 16 GB', icon: 'memory', color: '#BF5AF2', bgColor: 'rgba(191,90,242,0.1)' },
    { label: '磁盘读写', value: `${metrics.disk.toFixed(0)} MB/s`, sub: 'SSD 剩余 245 GB', icon: 'hardDrive', color: '#30D158', bgColor: 'rgba(48,209,88,0.1)' },
    { label: '网络流量', value: `${metrics.net.toFixed(1)} MB/s`, sub: '↓ 22.4 ↑ 5.6', icon: 'wifi', color: '#FF9F0A', bgColor: 'rgba(255,159,10,0.1)' },
  ];

  return (
    <div style={{ padding: 20, overflow: 'auto', height: '100%', display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* 欢迎区 */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div>
          <h1 style={{ fontSize: 22, fontWeight: 700, marginBottom: 4 }}>早上好，开发者 👋</h1>
          <p style={{ fontSize: 13, color: 'rgba(230,232,239,0.5)' }}>今天是个写代码的好日子 · Apple M1 已为你就绪</p>
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <div className="glass-light" style={{ padding: '8px 14px', borderRadius: 10, display: 'flex', alignItems: 'center', gap: 6 }}>
            <div style={{ width: 6, height: 6, borderRadius: '50%', background: '#30D158', animation: 'pulse-soft 2s infinite' }} />
            <span style={{ fontSize: 12, color: 'rgba(230,232,239,0.7)' }}>所有服务正常</span>
          </div>
        </div>
      </div>

      {/* 系统资源概览卡片 */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 12 }}>
        {metricCards.map((card, i) => (
          <div key={i} className="glass-light hover-lift" style={{
            borderRadius: 14, padding: 16, position: 'relative', overflow: 'hidden',
            cursor: 'pointer',
          }} onClick={() => onNavigate && onNavigate('monitor')}>
            <div style={{
              position: 'absolute', top: 0, right: 0, width: 80, height: 80,
              background: card.bgColor, borderRadius: '50%',
              filter: 'blur(30px)', transform: 'translate(20%, -20%)',
            }} />
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 12, position: 'relative' }}>
              <div style={{
                width: 36, height: 36, borderRadius: 10,
                background: card.bgColor, display: 'flex',
                alignItems: 'center', justifyContent: 'center',
              }}>
                <Icon name={card.icon} size={18} color={card.color} />
              </div>
              <span style={{ fontSize: 12, color: 'rgba(230,232,239,0.6)' }}>{card.label}</span>
            </div>
            <div className="mono" style={{ fontSize: 24, fontWeight: 700, marginBottom: 4, position: 'relative' }}>{card.value}</div>
            <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.4)', position: 'relative' }}>{card.sub}</div>
          </div>
        ))}
      </div>

      {/* 中间区：任务 + 最近项目 */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
        {/* 今日任务 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{
                width: 28, height: 28, borderRadius: 8,
                background: 'rgba(10,132,255,0.15)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <Icon name="check" size={14} color="#0A84FF" />
              </div>
              <span style={{ fontSize: 14, fontWeight: 600 }}>今日任务</span>
              <span style={{
                fontSize: 11, padding: '2px 8px', borderRadius: 10,
                background: 'rgba(255,255,255,0.08)',
                color: 'rgba(230,232,239,0.5)',
              }}>
                {tasks.filter(t => t.done).length}/{tasks.length}
              </span>
            </div>
            <button style={{
              padding: '4px 10px', fontSize: 11, borderRadius: 6,
              border: '1px solid rgba(255,255,255,0.1)', background: 'transparent',
              color: 'rgba(230,232,239,0.6)', cursor: 'pointer',
            }}>
              + 新建
            </button>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {tasks.map(task => (
              <div key={task.id}
                onClick={() => toggleTask(task.id)}
                style={{
                  display: 'flex', alignItems: 'center', gap: 10, padding: '8px 10px',
                  borderRadius: 8, cursor: 'pointer',
                  background: task.done ? 'rgba(255,255,255,0.02)' : 'transparent',
                }}
                className="hover-lift"
              >
                <div style={{
                  width: 18, height: 18, borderRadius: 5,
                  border: task.done ? '1.5px solid #30D158' : '1.5px solid rgba(255,255,255,0.2)',
                  background: task.done ? '#30D158' : 'transparent',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  flexShrink: 0, transition: 'all 0.2s',
                }}>
                  {task.done && <Icon name="check" size={12} color="#fff" strokeWidth={2.5} />}
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{
                    fontSize: 12.5, textDecoration: task.done ? 'line-through' : 'none',
                    color: task.done ? 'rgba(230,232,239,0.35)' : 'rgba(230,232,239,0.9)',
                  }}>{task.text}</div>
                  <div style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)', marginTop: 2 }}>
                    {task.project}
                  </div>
                </div>
                <div style={{
                  width: 6, height: 6, borderRadius: '50%', flexShrink: 0,
                  background: task.priority === 'high' ? '#FF6B6B' : task.priority === 'med' ? '#FF9F0A' : 'rgba(230,232,239,0.2)',
                }} />
              </div>
            ))}
          </div>
        </div>

        {/* 最近项目 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <div style={{
                width: 28, height: 28, borderRadius: 8,
                background: 'rgba(191,90,242,0.15)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <Icon name="folder" size={14} color="#BF5AF2" />
              </div>
              <span style={{ fontSize: 14, fontWeight: 600 }}>最近项目</span>
            </div>
            <button style={{
              padding: '4px 10px', fontSize: 11, borderRadius: 6,
              border: '1px solid rgba(255,255,255,0.1)', background: 'transparent',
              color: 'rgba(230,232,239,0.6)', cursor: 'pointer',
            }}>
              查看全部
            </button>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            {recentProjects.map((p, i) => (
              <div key={i}
                className="hover-lift"
                style={{
                  display: 'flex', alignItems: 'center', gap: 10, padding: '10px 12px',
                  borderRadius: 10, background: 'rgba(255,255,255,0.03)',
                  cursor: 'pointer',
                }}
                onClick={() => onNavigate && onNavigate('project')}
              >
                <div style={{
                  width: 32, height: 32, borderRadius: 8,
                  background: p.color + '20',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  flexShrink: 0,
                }}>
                  <Icon name="folder" size={16} color={p.color} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 13, fontWeight: 500, marginBottom: 2 }}>{p.name}</div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 10, color: 'rgba(230,232,239,0.4)' }}>
                    <span style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
                      <Icon name="gitBranch" size={10} />
                      {p.branch}
                    </span>
                    <span>·</span>
                    <span>{p.lastMod}</span>
                  </div>
                </div>
                <span style={{
                  fontSize: 10, padding: '2px 8px', borderRadius: 6,
                  background: p.color + '18', color: p.color,
                  flexShrink: 0,
                }}>{p.lang}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* 底部区：快捷入口 + AI 快捷指令 */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.3fr 1fr', gap: 12 }}>
        {/* 快捷入口网格 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14 }}>
            <div style={{
              width: 28, height: 28, borderRadius: 8,
              background: 'rgba(48,209,88,0.15)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <Icon name="zap" size={14} color="#30D158" />
            </div>
            <span style={{ fontSize: 14, fontWeight: 600 }}>快捷入口</span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 10 }}>
            {quickActions.map((a, i) => (
              <div key={i}
                className="hover-lift"
                style={{
                  display: 'flex', flexDirection: 'column', alignItems: 'center',
                  gap: 8, padding: '14px 8px', borderRadius: 10,
                  background: 'rgba(255,255,255,0.03)',
                  cursor: 'pointer', border: '1px solid transparent',
                  transition: 'all 0.2s',
                }}
                onClick={() => onNavigate && onNavigate(a.action)}
                onMouseEnter={(e) => { e.currentTarget.style.borderColor = a.color + '50'; }}
                onMouseLeave={(e) => { e.currentTarget.style.borderColor = 'transparent'; }}
              >
                <div style={{
                  width: 36, height: 36, borderRadius: 10,
                  background: a.color + '18',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                }}>
                  <Icon name={a.icon} size={18} color={a.color} />
                </div>
                <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.75)' }}>{a.name}</span>
              </div>
            ))}
          </div>
        </div>

        {/* AI 快捷指令 */}
        <div className="glass-light" style={{ borderRadius: 14, padding: 16, position: 'relative', overflow: 'hidden' }}>
          <div style={{
            position: 'absolute', top: -20, right: -20, width: 120, height: 120,
            background: 'radial-gradient(circle, rgba(10,132,255,0.2) 0%, transparent 70%)',
          }} />
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14, position: 'relative' }}>
            <div style={{
              width: 28, height: 28, borderRadius: 8,
              background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <Icon name="sparkles" size={14} color="#fff" />
            </div>
            <span style={{ fontSize: 14, fontWeight: 600 }}>AI 快捷指令</span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, position: 'relative' }}>
            {aiQuickActions.map((a, i) => (
              <div key={i}
                className="hover-lift"
                style={{
                  display: 'flex', flexDirection: 'column', gap: 4,
                  padding: '10px 12px', borderRadius: 10,
                  background: 'rgba(255,255,255,0.04)',
                  cursor: 'pointer',
                }}
                onClick={() => onNavigate && onNavigate('ai')}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Icon name={a.icon} size={14} color={a.color} />
                  <span style={{ fontSize: 12, fontWeight: 500 }}>{a.name}</span>
                </div>
                <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.4)' }}>{a.desc}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { Dashboard });
