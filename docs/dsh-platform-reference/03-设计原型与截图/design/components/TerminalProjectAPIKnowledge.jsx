// ─── 终端模块 ───────────────────────────────────────
function TerminalModule() {
  const [activeTab, setActiveTab] = React.useState(0);
  const [input, setInput] = React.useState('');
  const [history, setHistory] = React.useState([
    { type: 'prompt', text: 'Last login: Mon Jan 15 10:00:23 on ttys000' },
    { type: 'prompt', text: '' },
    { type: 'cmd', cmd: 'cd ~/projects/auth-service', output: '' },
    { type: 'cmd', cmd: 'git status', output: `On branch feature/oauth2\nYour branch is up to date with 'origin/feature/oauth2'.\n\nChanges not staged for commit:\n  (use "git add <file>..." to update what will be committed)\n  (use "git restore <file>..." to discard changes in working directory)\n\tmodified:   src/main/java/com/example/service/UserService.java\n\tmodified:   src/main/java/com/example/controller/UserController.java\n\nUntracked files:\n  (use "git add <file>..." to include in what will be committed)\n\tsrc/main/java/com/example/dto/UserDTO.java\n\nno changes added to commit (use "git add" and/or "git commit -a")` },
  ]);

  const tabs = ['auth-service', 'docker', 'monitor', 'build'];
  const inputRef = React.useRef(null);

  const runCommand = (cmd) => {
    if (!cmd.trim()) return;
    let output = '';
    const c = cmd.trim().toLowerCase();

    if (c === 'ls' || c === 'ls -la') {
      output = `total 80\ndrwxr-xr-x  12 dev  staff   384 Jan 15 10:20 .\ndrwxr-xr-x   8 dev  staff   256 Jan 10 09:00 ..\n-rw-r--r--   1 dev  staff   340 Jan 10 09:00 .gitignore\ndrwxr-xr-x  13 dev  staff   416 Jan 15 10:18 .idea\n-rw-r--r--   1 dev  staff  1234 Jan 10 09:00 Dockerfile\n-rw-r--r--   1 dev  staff  2567 Jan 10 09:00 README.md\ndrwxr-xr-x   4 dev  staff   128 Jan 10 09:00 src\n-rw-r--r--   1 dev  staff  8765 Jan 10 09:00 pom.xml`;
    } else if (c.startsWith('git ')) {
      if (c === 'git status') {
        output = `On branch feature/oauth2\nYour branch is ahead of 'origin/feature/oauth2' by 1 commit.\n  (use "git push" to publish your local commits)\n\nChanges not staged for commit:\n\tmodified:   src/main/java/com/example/service/UserService.java\n\nno changes added to commit`;
      } else if (c === 'git log' || c === 'git log --oneline') {
        output = `a3f2c1d (HEAD -> feature/oauth2) Add OAuth2 token refresh support\nb7e8a4f Fix JWT token expiration bug\nc1d2e3f Implement user registration endpoint\nf4a5b6c Initial auth service setup\n9e8d7c6 Project initialization`;
      } else if (c === 'git branch') {
        output = `  develop\n* feature/oauth2\n  main\n  fix/password-reset`;
      } else {
        output = `执行: ${cmd}\n(模拟输出 - 命令已提交)`;
      }
    } else if (c === 'pwd') {
      output = '/Users/dev/projects/auth-service';
    } else if (c === 'whoami') {
      output = 'dev';
    } else if (c === 'date') {
      output = new Date().toString();
    } else if (c === 'clear') {
      setHistory([]);
      setInput('');
      return;
    } else if (c.startsWith('docker ')) {
      output = `CONTAINER ID   IMAGE          STATUS          PORTS                    NAMES\nabc123def456   postgres:15    Up 2 hours      0.0.0.0:5432->5432/tcp   postgres-dev\n789ghi012jkl   redis:7        Up 2 hours      0.0.0.0:6379->6379/tcp   redis-cache\nmno345pqr678   nginx:alpine   Up 1 hour       0.0.0.0:8080->80/tcp     nginx-proxy`;
    } else if (c.startsWith('npm ')) {
      output = `> auth-dashboard@1.0.0 build\n> vite build\n\nvite v5.0.12 building for production...\n✓ 203 modules transformed.\ndist/index.html                  0.87 kB\ndist/assets/index-abc123.js     156.34 kB\ndist/assets/index-def456.css     23.45 kB\n✓ built in 1.23s`;
    } else {
      output = `zsh: command not found: ${cmd.split(' ')[0]}\n(模拟环境 - 支持 ls, git, docker, npm, pwd, date, clear 等命令)`;
    }

    setHistory(prev => [...prev, { type: 'cmd', cmd, output }]);
    setInput('');
    setTimeout(() => {
      inputRef.current?.focus();
    }, 50);
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') {
      runCommand(input);
    }
  };

  const quickCmds = [
    { name: 'git status', icon: 'gitBranch' },
    { name: 'npm install', icon: 'refresh' },
    { name: 'docker ps', icon: 'database' },
    { name: 'git log --oneline', icon: 'clock' },
  ];

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* 终端主体 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', background: '#0d0f1a' }}>
        {/* 标签栏 */}
        <div style={{
          display: 'flex', alignItems: 'center',
          background: '#141726',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          padding: '0 6px', minHeight: 32,
        }}>
          {tabs.map((t, i) => (
            <div
              key={i}
              onClick={() => setActiveTab(i)}
              style={{
                display: 'flex', alignItems: 'center', gap: 6,
                padding: '6px 14px', cursor: 'pointer',
                fontSize: 11.5,
                color: activeTab === i ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                borderBottom: activeTab === i ? '2px solid #30D158' : '2px solid transparent',
                marginRight: 2,
                borderRadius: '4px 4px 0 0',
              }}
            >
              <span style={{ width: 6, height: 6, borderRadius: '50%', background: activeTab === i ? '#30D158' : 'rgba(230,232,239,0.2)' }} />
              {t}
            </div>
          ))}
          <div style={{
            width: 28, height: 24, display: 'flex', alignItems: 'center', justifyContent: 'center',
            cursor: 'pointer', borderRadius: 4,
            color: 'rgba(230,232,239,0.4)',
          }}>
            <Icon name="plus" size={13} />
          </div>
        </div>

        {/* 终端内容 */}
        <div
          className="mono"
          style={{
            flex: 1, padding: '12px 14px', overflow: 'auto',
            fontSize: 12, lineHeight: 1.65, color: '#e6e8ef',
          }}
          onClick={() => inputRef.current?.focus()}
        >
          {history.map((h, i) => (
            <div key={i} style={{ marginBottom: 2 }}>
              {h.type === 'prompt' && (
                <div style={{ color: 'rgba(230,232,239,0.5)', fontSize: 11 }}>{h.text}</div>
              )}
              {h.type === 'cmd' && (
                <>
                  <div style={{ display: 'flex', alignItems: 'center', flexWrap: 'wrap' }}>
                    <span style={{ color: '#30D158' }}>➜</span>
                    <span style={{ color: '#569CD6', marginLeft: 8 }}>auth-service</span>
                    <span style={{ color: '#E76F51', marginLeft: 6 }}>(feature/oauth2)</span>
                    <span style={{ color: '#e6e8ef', marginLeft: 8 }}>{h.cmd}</span>
                  </div>
                  {h.output && h.output.split('\n').map((line, li) => (
                    <div key={li} style={{
                      color: line.startsWith('\t') ? '#c3e88d'
                           : line.includes('(HEAD') || line.includes('*') ? '#FFD60A'
                           : 'rgba(230,232,239,0.75)',
                      paddingLeft: 2,
                    }}>{line}</div>
                  ))}
                </>
              )}
            </div>
          ))}
          {/* 当前输入行 */}
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <span style={{ color: '#30D158' }}>➜</span>
            <span style={{ color: '#569CD6', marginLeft: 8 }}>auth-service</span>
            <span style={{ color: '#E76F51', marginLeft: 6 }}>(feature/oauth2)</span>
            <input
              ref={inputRef}
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              autoFocus
              style={{
                flex: 1, marginLeft: 8,
                background: 'transparent', border: 'none', outline: 'none',
                color: '#e6e8ef', fontFamily: 'JetBrains Mono, monospace',
                fontSize: 12, caretColor: '#30D158',
              }}
            />
          </div>
        </div>
      </div>

      {/* 右侧快捷命令 */}
      <div style={{
        width: 200, flexShrink: 0,
        background: 'rgba(20,22,36,0.6)',
        borderLeft: '1px solid rgba(255,255,255,0.06)',
        display: 'flex', flexDirection: 'column',
      }}>
        <div style={{
          padding: '12px 14px', fontSize: 11, fontWeight: 600,
          color: 'rgba(230,232,239,0.5)',
          textTransform: 'uppercase', letterSpacing: 0.5,
          borderBottom: '1px solid rgba(255,255,255,0.06)',
        }}>
          快捷命令
        </div>
        <div style={{ padding: 8, flex: 1 }}>
          {quickCmds.map((c, i) => (
            <button
              key={i}
              onClick={() => runCommand(c.name)}
              style={{
                display: 'flex', alignItems: 'center', gap: 8,
                width: '100%', padding: '8px 10px', marginBottom: 4,
                background: 'rgba(255,255,255,0.04)',
                border: '1px solid rgba(255,255,255,0.06)',
                borderRadius: 8, cursor: 'pointer',
                color: 'rgba(230,232,239,0.75)',
                fontSize: 11.5, textAlign: 'left',
                transition: 'all 0.15s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(48,209,88,0.12)';
                e.currentTarget.style.borderColor = 'rgba(48,209,88,0.3)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.04)';
                e.currentTarget.style.borderColor = 'rgba(255,255,255,0.06)';
              }}
            >
              <Icon name={c.icon} size={13} color="#30D158" />
              <span className="mono">{c.name}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}


// ─── 项目与 Git 模块 ───────────────────────────────────────
function ProjectGit() {
  const [selectedFile, setSelectedFile] = React.useState(0);
  const [showDiff, setShowDiff] = React.useState(true);

  const projects = [
    { name: 'auth-service', active: true, branch: 'feature/oauth2', status: 'modified' },
    { name: 'web-dashboard', active: false, branch: 'main', status: 'clean' },
    { name: 'order-service', active: false, branch: 'develop', status: 'modified' },
    { name: 'ml-pipeline', active: false, branch: 'main', status: 'clean' },
    { name: 'api-gateway', active: false, branch: 'feature/rate-limit', status: 'staged' },
  ];

  const changedFiles = [
    { name: 'UserService.java', path: 'src/main/java/.../service/', status: 'modified', additions: 23, deletions: 8 },
    { name: 'UserController.java', path: 'src/main/java/.../controller/', status: 'modified', additions: 15, deletions: 3 },
    { name: 'UserDTO.java', path: 'src/main/java/.../dto/', status: 'new', additions: 42, deletions: 0 },
    { name: 'application.yml', path: 'src/main/resources/', status: 'modified', additions: 5, deletions: 2 },
  ];

  const stagedFiles = [
    { name: 'AuthInterceptor.java', path: 'src/main/java/.../interceptor/', status: 'staged', additions: 67, deletions: 0 },
  ];

  const commits = [
    { hash: 'a3f2c1d', message: 'Add OAuth2 token refresh support', author: 'dev', time: '2 小时前', branch: 'feature/oauth2' },
    { hash: 'b7e8a4f', message: 'Fix JWT token expiration bug', author: 'dev', time: '昨天', branch: 'feature/oauth2' },
    { hash: 'c1d2e3f', message: 'Implement user registration endpoint', author: 'dev', time: '2 天前', branch: 'feature/oauth2' },
    { hash: 'f4a5b6c', message: 'Initial auth service setup', author: 'dev', time: '3 天前', branch: 'develop' },
  ];

  const diffContent = [
    { type: 'context', text: '@Service' },
    { type: 'context', text: 'public class UserService {' },
    { type: 'context', text: '' },
    { type: 'del', text: '-    public Optional<User> findById(Long id) {' },
    { type: 'del', text: '-        return userRepository.findById(id);' },
    { type: 'del', text: '-    }' },
    { type: 'add', text: '+    @Cacheable(value = "users", key = "#id")' },
    { type: 'add', text: '+    public Optional<User> findById(Long id) {' },
    { type: 'add', text: '+        log.info("Finding user by id: {}", id);' },
    { type: 'add', text: '+        return userRepository.findById(id)' },
    { type: 'add', text: '+            .map(user -> {' },
    { type: 'add', text: '+                user.setPassword(null);' },
    { type: 'add', text: '+                return user;' },
    { type: 'add', text: '+            });' },
    { type: 'add', text: '+    }' },
    { type: 'context', text: '' },
    { type: 'context', text: '    // 新增：批量查询' },
    { type: 'add', text: '+    public List<User> findByIds(List<Long> ids) {' },
    { type: 'add', text: '+        return userRepository.findAllById(ids);' },
    { type: 'add', text: '+    }' },
  ];

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* 左侧项目列表 */}
      <div style={{
        width: 220, flexShrink: 0,
        background: 'rgba(20,22,36,0.4)',
        borderRight: '1px solid rgba(255,255,255,0.06)',
        display: 'flex', flexDirection: 'column',
      }}>
        <div style={{
          padding: '12px 14px', fontSize: 11, fontWeight: 600,
          color: 'rgba(230,232,239,0.5)',
          textTransform: 'uppercase', letterSpacing: 0.5,
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}>
          项目列表
          <Icon name="plus" size={13} color="rgba(230,232,239,0.4)" />
        </div>
        <div style={{ flex: 1, overflow: 'auto', padding: 4 }}>
          {projects.map((p, i) => (
            <div key={i} style={{
              padding: '10px 12px', borderRadius: 8, marginBottom: 2,
              cursor: 'pointer',
              background: p.active ? 'rgba(10,132,255,0.15)' : 'transparent',
              border: p.active ? '1px solid rgba(10,132,255,0.25)' : '1px solid transparent',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                <Icon name="folder" size={14} color={p.active ? '#0A84FF' : 'rgba(230,232,239,0.5)'} />
                <span style={{ fontSize: 12, fontWeight: p.active ? 600 : 400 }}>{p.name}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, paddingLeft: 22 }}>
                <Icon name="gitBranch" size={10} color="rgba(230,232,239,0.4)" />
                <span style={{ fontSize: 10.5, color: 'rgba(230,232,239,0.45)' }}>{p.branch}</span>
                <span style={{
                  width: 6, height: 6, borderRadius: '50%', marginLeft: 'auto',
                  background: p.status === 'clean' ? '#30D158' : p.status === 'staged' ? '#FFD60A' : '#FF9F0A',
                }} />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 右侧主内容 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* 顶部操作栏 */}
        <div style={{
          padding: '10px 16px',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', alignItems: 'center', gap: 8,
          background: 'rgba(20,22,36,0.4)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '0 10px' }}>
            <Icon name="gitBranch" size={16} color="#E76F51" />
            <span style={{ fontSize: 13, fontWeight: 600 }}>feature/oauth2</span>
            <span style={{ fontSize: 11, color: 'rgba(230,232,239,0.4)', marginLeft: 4 }}>· 修改 4 个文件</span>
          </div>
          <div style={{ flex: 1 }} />
          <button style={gitBtnStyle('secondary')}>拉取</button>
          <button style={gitBtnStyle('secondary')}>推送</button>
          <button style={gitBtnStyle('primary')}>提交</button>
          <button style={gitBtnStyle('secondary')}>切换分支</button>
        </div>

        {/* 内容区：变更 + 历史 */}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {/* 变更文件 */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', borderRight: '1px solid rgba(255,255,255,0.06)' }}>
            <div style={{ padding: '10px 16px', fontSize: 11, fontWeight: 600, color: 'rgba(230,232,239,0.5)' }}>
              暂存的更改 ({stagedFiles.length})
            </div>
            <div style={{ padding: '0 8px' }}>
              {stagedFiles.map((f, i) => (
                <FileRow key={i} file={f} staged onClick={() => setSelectedFile(i)} selected={selectedFile === i && showDiff} />
              ))}
            </div>
            <div style={{ padding: '12px 16px 8px', fontSize: 11, fontWeight: 600, color: 'rgba(230,232,239,0.5)' }}>
              更改 ({changedFiles.length})
            </div>
            <div style={{ padding: '0 8px', flex: 1, overflow: 'auto' }}>
              {changedFiles.map((f, i) => (
                <FileRow key={i} file={f} onClick={() => { setSelectedFile(i + 10); setShowDiff(true); }} selected={selectedFile === i + 10 && showDiff} />
              ))}
            </div>
          </div>

          {/* Diff 或 提交历史 */}
          <div style={{ width: '50%', display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
            <div style={{
              display: 'flex', borderBottom: '1px solid rgba(255,255,255,0.06)',
            }}>
              <div style={{
                padding: '10px 16px', fontSize: 12, cursor: 'pointer',
                color: showDiff ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                borderBottom: showDiff ? '2px solid #0A84FF' : '2px solid transparent',
              }} onClick={() => setShowDiff(true)}>
                差异对比
              </div>
              <div style={{
                padding: '10px 16px', fontSize: 12, cursor: 'pointer',
                color: !showDiff ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                borderBottom: !showDiff ? '2px solid #0A84FF' : '2px solid transparent',
              }} onClick={() => setShowDiff(false)}>
                提交历史
              </div>
            </div>

            {showDiff ? (
              <div style={{ flex: 1, overflow: 'auto', padding: 12 }} className="mono">
                <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.5)', marginBottom: 8 }}>
                  UserService.java · +23 -8
                </div>
                {diffContent.map((line, i) => (
                  <div key={i} style={{
                    fontSize: 11.5, lineHeight: 1.6, padding: '0 8px',
                    background: line.type === 'add' ? 'rgba(48,209,88,0.1)'
                              : line.type === 'del' ? 'rgba(255,107,107,0.1)'
                              : 'transparent',
                    color: line.type === 'add' ? '#30D158'
                         : line.type === 'del' ? '#FF6B6B'
                         : 'rgba(230,232,239,0.6)',
                    borderRadius: 2,
                  }}>
                    <span style={{ display: 'inline-block', width: 20, opacity: 0.5 }}>
                      {line.type === 'add' ? '+' : line.type === 'del' ? '-' : ' '}
                    </span>
                    {line.text}
                  </div>
                ))}
              </div>
            ) : (
              <div style={{ flex: 1, overflow: 'auto', padding: '4px 0' }}>
                {/* 分支可视化节点 */}
                <div style={{ padding: '12px 16px', display: 'flex', gap: 14, position: 'relative' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0 }}>
                    {commits.map((c, i) => (
                      <React.Fragment key={i}>
                        <div style={{
                          width: 10, height: 10, borderRadius: '50%',
                          background: i === 0 ? '#0A84FF' : '#E76F51',
                          border: '2px solid rgba(20,22,36,0.8)',
                          zIndex: 1, flexShrink: 0,
                          boxShadow: i === 0 ? '0 0 0 2px rgba(10,132,255,0.3)' : 'none',
                        }} />
                        {i < commits.length - 1 && (
                          <div style={{
                            width: 2, height: 44,
                            background: 'rgba(231,111,81,0.3)',
                            margin: '-2px 0',
                          }} />
                        )}
                      </React.Fragment>
                    ))}
                  </div>
                  <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 0 }}>
                    {commits.map((c, i) => (
                      <div key={i} style={{
                        padding: '4px 0',
                        height: 50,
                        display: 'flex', flexDirection: 'column', justifyContent: 'center',
                      }}>
                        <div style={{ fontSize: 12.5, fontWeight: 500, marginBottom: 2 }}>{c.message}</div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 10.5, color: 'rgba(230,232,239,0.45)' }}>
                          <span className="mono" style={{ color: '#E76F51' }}>{c.hash}</span>
                          <span>·</span>
                          <span>{c.author}</span>
                          <span>·</span>
                          <span>{c.time}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

const gitBtnStyle = (type) => ({
  padding: '6px 14px',
  fontSize: 11.5,
  borderRadius: 6,
  border: type === 'primary' ? 'none' : '1px solid rgba(255,255,255,0.1)',
  background: type === 'primary' ? 'linear-gradient(135deg, #0A84FF, #0060DF)' : 'rgba(255,255,255,0.06)',
  color: '#fff',
  cursor: 'pointer',
  fontWeight: type === 'primary' ? 600 : 400,
});

function FileRow({ file, selected, onClick, staged }) {
  const statusColor = file.status === 'new' ? '#30D158' : file.status === 'modified' ? '#FF9F0A' : '#FFD60A';
  return (
    <div
      onClick={onClick}
      style={{
        display: 'flex', alignItems: 'center', gap: 8,
        padding: '6px 10px', borderRadius: 6, cursor: 'pointer',
        background: selected ? 'rgba(10,132,255,0.15)' : 'transparent',
        marginBottom: 1,
      }}
    >
      <Icon name="file" size={13} color={staged ? '#FFD60A' : 'rgba(230,232,239,0.5)'} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 11.5, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{file.name}</div>
        <div style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)' }}>{file.path}</div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: 1 }}>
        {file.additions > 0 && <span style={{ fontSize: 10, color: '#30D158' }}>+{file.additions}</span>}
        {file.deletions > 0 && <span style={{ fontSize: 10, color: '#FF6B6B' }}>-{file.deletions}</span>}
      </div>
    </div>
  );
}


// ─── API 调试模块 ───────────────────────────────────────
function APIDebug() {
  const [method, setMethod] = React.useState('GET');
  const [url, setUrl] = React.useState('https://api.example.com/api/v1/users');
  const [activeTab, setActiveTab] = React.useState('params');
  const [response, setResponse] = React.useState(null);
  const [loading, setLoading] = React.useState(false);
  const [bodyTab, setBodyTab] = React.useState('json');

  const methods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH'];

  const history = [
    { method: 'GET', url: '/api/v1/users', status: 200, time: '10:30', name: '获取用户列表' },
    { method: 'POST', url: '/api/v1/auth/login', status: 200, time: '10:25', name: '用户登录' },
    { method: 'GET', url: '/api/v1/users/123', status: 200, time: '10:20', name: '获取用户详情' },
    { method: 'PUT', url: '/api/v1/users/123', status: 200, time: '10:15', name: '更新用户信息' },
    { method: 'DELETE', url: '/api/v1/users/456', status: 204, time: '昨天', name: '删除用户' },
    { method: 'POST', url: '/api/v1/users', status: 201, time: '昨天', name: '创建用户' },
  ];

  const sendRequest = () => {
    setLoading(true);
    setResponse(null);
    setTimeout(() => {
      const resp = {
        status: 200,
        statusText: 'OK',
        time: 234 + Math.floor(Math.random() * 100),
        size: '1.2 KB',
        headers: {
          'Content-Type': 'application/json',
          'X-Request-Id': 'req_' + Math.random().toString(36).substr(2, 10),
          'X-RateLimit-Remaining': '998',
          'Cache-Control': 'no-cache',
        },
        body: JSON.stringify({
          code: 0,
          message: 'success',
          data: {
            total: 156,
            page: 1,
            pageSize: 10,
            list: [
              { id: 1, username: 'alice', email: 'alice@example.com', role: 'admin', createdAt: '2024-01-10T08:00:00Z' },
              { id: 2, username: 'bob', email: 'bob@example.com', role: 'user', createdAt: '2024-01-11T09:30:00Z' },
              { id: 3, username: 'charlie', email: 'charlie@example.com', role: 'user', createdAt: '2024-01-12T14:20:00Z' },
            ],
          },
        }, null, 2),
      };
      setResponse(resp);
      setLoading(false);
    }, 800);
  };

  const methodColor = (m) => {
    switch (m) {
      case 'GET': return '#30D158';
      case 'POST': return '#FF9F0A';
      case 'PUT': return '#0A84FF';
      case 'DELETE': return '#FF6B6B';
      case 'PATCH': return '#BF5AF2';
      default: return '#fff';
    }
  };

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* 左侧历史记录 */}
      <div style={{
        width: 260, flexShrink: 0,
        background: 'rgba(20,22,36,0.4)',
        borderRight: '1px solid rgba(255,255,255,0.06)',
        display: 'flex', flexDirection: 'column',
      }}>
        <div style={{ padding: 12 }}>
          <button style={{
            width: '100%', padding: '8px 0',
            background: 'linear-gradient(135deg, #0A84FF, #0060DF)',
            border: 'none', borderRadius: 8, color: '#fff',
            fontSize: 12, fontWeight: 600, cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
          }}>
            <Icon name="plus" size={14} />
            新建请求
          </button>
        </div>
        <div style={{
          padding: '0 12px 8px', fontSize: 11, fontWeight: 600,
          color: 'rgba(230,232,239,0.5)',
        }}>
          历史记录
        </div>
        <div style={{ flex: 1, overflow: 'auto', padding: '0 8px' }}>
          {history.map((h, i) => (
            <div key={i} style={{
              padding: '10px 10px', borderRadius: 8,
              cursor: 'pointer', marginBottom: 2,
            }}
              onMouseEnter={(e) => { e.currentTarget.style.background = 'rgba(255,255,255,0.05)'; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = 'transparent'; }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                <span style={{
                  fontSize: 10, fontWeight: 700, color: methodColor(h.method),
                  minWidth: 36,
                }}>{h.method}</span>
                <span className="mono" style={{ fontSize: 11, color: 'rgba(230,232,239,0.6)', flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{h.url}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, paddingLeft: 44 }}>
                <span style={{ fontSize: 10, color: '#30D158' }}>{h.status}</span>
                <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)' }}>{h.time}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 主编辑区 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        {/* URL 栏 */}
        <div style={{
          padding: 12, borderBottom: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', gap: 8, alignItems: 'center',
          background: 'rgba(20,22,36,0.4)',
        }}>
          <div style={{ position: 'relative' }}>
            <select value={method} onChange={e => setMethod(e.target.value)}
              style={{
                padding: '8px 32px 8px 12px', fontSize: 12, fontWeight: 600,
                background: 'rgba(255,255,255,0.06)',
                border: '1px solid rgba(255,255,255,0.1)',
                borderRadius: 8, color: methodColor(method),
                appearance: 'none', cursor: 'pointer',
              }}
            >
              {methods.map(m => <option key={m} value={m}>{m}</option>)}
            </select>
            <Icon name="chevronDown" size={12} color="rgba(230,232,239,0.4)" style={{ position: 'absolute', right: 10, top: '50%', transform: 'translateY(-50%)', pointerEvents: 'none' }} />
          </div>
          <input
            value={url}
            onChange={e => setUrl(e.target.value)}
            style={{
              flex: 1, padding: '8px 12px', fontSize: 12,
              background: 'rgba(255,255,255,0.06)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: 8, color: '#e6e8ef', outline: 'none',
              fontFamily: 'JetBrains Mono, monospace',
            }}
            placeholder="输入请求 URL"
          />
          <button onClick={sendRequest} style={{
            padding: '8px 20px', fontSize: 12, fontWeight: 600,
            background: loading ? 'rgba(48,209,88,0.5)' : 'linear-gradient(135deg, #30D158, #28A745)',
            border: 'none', borderRadius: 8, color: '#fff',
            cursor: loading ? 'wait' : 'pointer',
            display: 'flex', alignItems: 'center', gap: 6,
          }}>
            {loading ? '发送中...' : (<><Icon name="play" size={12} /> 发送</>)}
          </button>
        </div>

        {/* 请求配置 Tab */}
        <div style={{
          display: 'flex', gap: 0,
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          padding: '0 12px',
        }}>
          {['params', 'headers', 'body', 'auth'].map(t => (
            <div key={t}
              onClick={() => setActiveTab(t)}
              style={{
                padding: '10px 16px', fontSize: 12, cursor: 'pointer',
                color: activeTab === t ? '#e6e8ef' : 'rgba(230,232,239,0.45)',
                borderBottom: activeTab === t ? '2px solid #0A84FF' : '2px solid transparent',
                marginBottom: -1,
              }}
            >
              {t === 'params' ? 'Query Params' : t === 'headers' ? 'Headers' : t === 'body' ? 'Body' : 'Auth'}
            </div>
          ))}
        </div>

        {/* 请求配置内容 + 响应 */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {/* 请求参数区 */}
          <div style={{ padding: 12, borderBottom: '1px solid rgba(255,255,255,0.06)', maxHeight: 200, overflow: 'auto' }}>
            {activeTab === 'params' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <ParamRow keyLabel="page" value="1" />
                <ParamRow keyLabel="pageSize" value="10" />
                <ParamRow keyLabel="keyword" value="" placeholder="搜索关键词" />
                <button style={{
                  padding: '4px 10px', fontSize: 11,
                  background: 'transparent', border: '1px dashed rgba(255,255,255,0.15)',
                  borderRadius: 6, color: 'rgba(230,232,239,0.4)',
                  cursor: 'pointer', width: 'fit-content',
                }}>+ 添加参数</button>
              </div>
            )}
            {activeTab === 'headers' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <ParamRow keyLabel="Content-Type" value="application/json" />
                <ParamRow keyLabel="Authorization" value="Bearer eyJhbGciOi..." />
                <ParamRow keyLabel="X-Request-Id" value="" placeholder="可选" />
              </div>
            )}
            {activeTab === 'body' && (
              <div>
                <div style={{ display: 'flex', gap: 4, marginBottom: 8 }}>
                  {['json', 'form', 'none'].map(t => (
                    <button key={t} onClick={() => setBodyTab(t)} style={{
                      padding: '4px 10px', fontSize: 11, borderRadius: 6,
                      background: bodyTab === t ? 'rgba(10,132,255,0.2)' : 'transparent',
                      border: bodyTab === t ? '1px solid rgba(10,132,255,0.3)' : '1px solid transparent',
                      color: bodyTab === t ? '#0A84FF' : 'rgba(230,232,239,0.5)',
                      cursor: 'pointer',
                    }}>
                      {t === 'json' ? 'JSON' : t === 'form' ? 'Form' : 'None'}
                    </button>
                  ))}
                </div>
                {bodyTab === 'json' && (
                  <pre className="mono" style={{
                    padding: 10, margin: 0, fontSize: 11.5,
                    background: 'rgba(0,0,0,0.2)', borderRadius: 8,
                    color: '#c3e88d', lineHeight: 1.5,
                    whiteSpace: 'pre-wrap',
                  }}>{JSON.stringify({
  username: "newuser",
  email: "new@example.com",
  password: "secret123",
  role: "user"
}, null, 2)}</pre>
                )}
              </div>
            )}
            {activeTab === 'auth' && (
              <div style={{ fontSize: 12, color: 'rgba(230,232,239,0.6)' }}>
                <div style={{ marginBottom: 8 }}>认证类型：Bearer Token</div>
                <div>Token: **********</div>
              </div>
            )}
          </div>

          {/* 响应区 */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
            <div style={{
              padding: '8px 16px', fontSize: 11, fontWeight: 600,
              color: 'rgba(230,232,239,0.5)',
              display: 'flex', alignItems: 'center', gap: 12,
              background: 'rgba(0,0,0,0.15)',
            }}>
              <span>响应</span>
              {response && (
                <>
                  <span style={{ color: '#30D158' }}>{response.status} {response.statusText}</span>
                  <span style={{ color: 'rgba(230,232,239,0.4)' }}>{response.time}ms</span>
                  <span style={{ color: 'rgba(230,232,239,0.4)' }}>{response.size}</span>
                </>
              )}
            </div>
            <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>
              {loading ? (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 12 }}>
                    <div style={{
                      width: 32, height: 32, border: '2px solid rgba(255,255,255,0.1)',
                      borderTopColor: '#0A84FF', borderRadius: '50%',
                      animation: 'spin 1s linear infinite',
                    }} />
                    <span style={{ fontSize: 12, color: 'rgba(230,232,239,0.5)' }}>正在发送请求...</span>
                  </div>
                </div>
              ) : response ? (
                <pre className="mono" style={{
                  fontSize: 11.5, lineHeight: 1.6,
                  color: '#c3e88d', margin: 0,
                  whiteSpace: 'pre-wrap',
                }}>{response.body}</pre>
              ) : (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'rgba(230,232,239,0.3)', fontSize: 12 }}>
                  点击「发送」按钮查看响应结果
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ParamRow({ keyLabel, value, placeholder }) {
  return (
    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
      <input value={keyLabel} style={paramInputStyle} readOnly />
      <span style={{ color: 'rgba(230,232,239,0.2)' }}>=</span>
      <input value={value} placeholder={placeholder} style={{ ...paramInputStyle, flex: 1 }} />
    </div>
  );
}

const paramInputStyle = {
  padding: '5px 10px', fontSize: 11.5,
  background: 'rgba(255,255,255,0.04)',
  border: '1px solid rgba(255,255,255,0.08)',
  borderRadius: 6, color: '#e6e8ef', outline: 'none',
  fontFamily: 'JetBrains Mono, monospace',
  width: 160,
};


// ─── 知识库/笔记模块 ───────────────────────────────────────
function KnowledgeBase() {
  const [search, setSearch] = React.useState('');
  const [selected, setSelected] = React.useState(0);

  const notes = [
    { title: 'React Hooks 最佳实践', summary: 'useState, useEffect, useCallback 的使用场景和性能优化建议...', tags: ['React', '前端', '性能'], date: '今天' },
    { title: 'MySQL 索引优化指南', summary: 'B+树索引原理、联合索引最左前缀原则、explain 分析...', tags: ['MySQL', '数据库', '性能'], date: '昨天' },
    { title: 'Docker Compose 常用配置', summary: '多容器编排、网络配置、数据卷挂载、环境变量传递...', tags: ['Docker', 'DevOps'], date: '3 天前' },
    { title: 'JWT 认证安全最佳实践', summary: 'Token 刷新机制、黑名单设计、CSRF 防护...', tags: ['安全', '后端', 'JWT'], date: '1 周前' },
    { title: 'Git 高级技巧', summary: 'rebase vs merge、cherry-pick、bisect 二分查找 bug...', tags: ['Git', '工具'], date: '2 周前' },
    { title: 'Kubernetes 入门笔记', summary: 'Pod、Service、Deployment、Ingress 核心概念...', tags: ['K8s', 'DevOps'], date: '1 个月前' },
  ];

  const snippets = [
    { title: '防抖节流函数', lang: 'JavaScript', code: 'function debounce(fn, delay) {\n  let timer = null;\n  return function(...args) {\n    clearTimeout(timer);\n    timer = setTimeout(() => fn.apply(this, args), delay);\n  };\n}' },
    { title: '深拷贝实现', lang: 'JavaScript', code: 'function deepClone(obj, map = new WeakMap()) {\n  if (obj === null || typeof obj !== "object") return obj;\n  if (map.has(obj)) return map.get(obj);\n  const clone = Array.isArray(obj) ? [] : {};\n  map.set(obj, clone);\n  for (const key in obj) {\n    if (obj.hasOwnProperty(key)) {\n      clone[key] = deepClone(obj[key], map);\n    }\n  }\n  return clone;\n}' },
    { title: '快速排序', lang: 'Python', code: 'def quick_sort(arr):\n    if len(arr) <= 1:\n        return arr\n    pivot = arr[len(arr) // 2]\n    left = [x for x in arr if x < pivot]\n    middle = [x for x in arr if x == pivot]\n    right = [x for x in arr if x > pivot]\n    return quick_sort(left) + middle + quick_sort(right)' },
  ];

  const sampleContent = `# React Hooks 最佳实践

## useState 使用建议

### 1. 合理拆分状态

不要把所有状态都放到一个 useState 中，应该按关联性拆分：

\`\`\`javascript
// 👎 不推荐
const [state, setState] = useState({
  list: [],
  loading: false,
  page: 1,
  filter: ''
});

// 👍 推荐
const [list, setList] = useState([]);
const [loading, setLoading] = useState(false);
const [pagination, setPagination] = useState({ page: 1, pageSize: 10 });
\`\`\`

### 2. 状态派生

能通过计算得到的值，不要单独存为状态：

\`\`\`javascript
// 👎 不推荐
const [fullName, setFullName] = useState('');

// 👍 推荐
const fullName = firstName + ' ' + lastName;
\`\`\`

## useEffect 注意事项

### 依赖数组要完整

- 所有在 effect 中使用的响应式值都应该出现在依赖数组中
- 使用 ESLint 的 exhaustive-deps 规则来检查

### 避免不必要的 effect

如果某个逻辑可以在事件回调中完成，就不要放到 useEffect 中。

## useCallback 与 useMemo

### useCallback 适用场景

1. 作为 props 传递给纯组件时
2. 作为其他 hook 的依赖时
3. 高频事件的处理函数

### useMemo 适用场景

1. 复杂计算的缓存
2. 保持引用稳定（对象/数组）
`;

  // 简单的 Markdown 渲染
  const renderMarkdown = (text) => {
    const lines = text.split('\n');
    const elements = [];
    let inCode = false;
    let codeBuffer = [];

    lines.forEach((line, i) => {
      if (line.startsWith('```')) {
        if (inCode) {
          elements.push(
            <pre key={i} className="mono" style={{
              background: 'rgba(0,0,0,0.3)', padding: 12, borderRadius: 8,
              fontSize: 12, color: '#c3e88d', overflow: 'auto', margin: '8px 0',
            }}>{codeBuffer.join('\n')}</pre>
          );
          codeBuffer = [];
          inCode = false;
        } else {
          inCode = true;
        }
        return;
      }
      if (inCode) {
        codeBuffer.push(line);
        return;
      }
      if (line.startsWith('# ')) {
        elements.push(<h1 key={i} style={{ fontSize: 20, fontWeight: 700, marginBottom: 12, marginTop: 8 }}>{line.slice(2)}</h1>);
      } else if (line.startsWith('## ')) {
        elements.push(<h2 key={i} style={{ fontSize: 16, fontWeight: 600, marginBottom: 8, marginTop: 16, color: '#0A84FF' }}>{line.slice(3)}</h2>);
      } else if (line.startsWith('### ')) {
        elements.push(<h3 key={i} style={{ fontSize: 14, fontWeight: 600, marginBottom: 6, marginTop: 12 }}>{line.slice(4)}</h3>);
      } else if (line.startsWith('- ')) {
        elements.push(<div key={i} style={{ display: 'flex', gap: 8, padding: '3px 0', paddingLeft: 16, fontSize: 13 }}>
          <span style={{ color: '#0A84FF' }}>•</span>
          <span dangerouslySetInnerHTML={{ __html: line.slice(2).replace(/`([^`]+)`/g, '<code style="background: rgba(255,255,255,0.1); padding: 1px 5px; border-radius: 4px; font-family: JetBrains Mono, monospace; font-size: 0.9em; color: #c3e88d;">$1</code>') }} />
        </div>);
      } else if (line.trim() === '') {
        elements.push(<div key={i} style={{ height: 8 }} />);
      } else {
        elements.push(<p key={i} style={{ fontSize: 13, lineHeight: 1.7, color: 'rgba(230,232,239,0.8)' }} dangerouslySetInnerHTML={{ __html: line.replace(/`([^`]+)`/g, '<code style="background: rgba(255,255,255,0.1); padding: 1px 5px; border-radius: 4px; font-family: JetBrains Mono, monospace; font-size: 0.9em; color: #c3e88d;">$1</code>') }} />);
      }
    });
    return elements;
  };

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* 左侧笔记列表 */}
      <div style={{
        width: 280, flexShrink: 0,
        background: 'rgba(20,22,36,0.4)',
        borderRight: '1px solid rgba(255,255,255,0.06)',
        display: 'flex', flexDirection: 'column',
      }}>
        <div style={{ padding: 12 }}>
          <div style={{ position: 'relative' }}>
            <Icon name="search" size={14} color="rgba(230,232,239,0.4)" style={{ position: 'absolute', left: 10, top: '50%', transform: 'translateY(-50%)' }} />
            <input
              value={search}
              onChange={e => setSearch(e.target.value)}
              placeholder="搜索笔记..."
              style={{
                width: '100%', padding: '8px 12px 8px 32px', fontSize: 12,
                background: 'rgba(255,255,255,0.06)',
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: 8, color: '#e6e8ef', outline: 'none',
              }}
            />
          </div>
        </div>
        <div style={{ padding: '0 12px 8px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 11, fontWeight: 600, color: 'rgba(230,232,239,0.5)' }}>笔记 ({notes.length})</span>
          <button style={{
            background: 'transparent', border: 'none', cursor: 'pointer',
            color: 'rgba(230,232,239,0.4)', padding: 2,
          }}>
            <Icon name="plus" size={14} />
          </button>
        </div>
        <div style={{ flex: 1, overflow: 'auto', padding: '0 8px' }}>
          {notes.map((note, i) => (
            <div key={i}
              onClick={() => setSelected(i)}
              style={{
                padding: '10px 12px', borderRadius: 8, cursor: 'pointer',
                marginBottom: 2,
                background: selected === i ? 'rgba(10,132,255,0.15)' : 'transparent',
                border: selected === i ? '1px solid rgba(10,132,255,0.25)' : '1px solid transparent',
              }}
            >
              <div style={{ fontSize: 12.5, fontWeight: 500, marginBottom: 4 }}>{note.title}</div>
              <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.45)', marginBottom: 6, lineHeight: 1.4, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>{note.summary}</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                {note.tags.slice(0, 2).map((t, ti) => (
                  <span key={ti} style={{
                    fontSize: 9, padding: '1px 6px', borderRadius: 4,
                    background: 'rgba(255,255,255,0.06)',
                    color: 'rgba(230,232,239,0.5)',
                  }}>{t}</span>
                ))}
                <div style={{ flex: 1 }} />
                <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)' }}>{note.date}</span>
              </div>
            </div>
          ))}
        </div>

        {/* 代码片段收藏 */}
        <div style={{ borderTop: '1px solid rgba(255,255,255,0.06)', padding: 12 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <span style={{ fontSize: 11, fontWeight: 600, color: 'rgba(230,232,239,0.5)' }}>代码片段</span>
            <span style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)' }}>{snippets.length}</span>
          </div>
          {snippets.slice(0, 2).map((s, i) => (
            <div key={i} style={{
              padding: '8px 10px', borderRadius: 6, marginBottom: 4,
              background: 'rgba(255,255,255,0.03)',
              cursor: 'pointer',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                <span style={{ fontSize: 11.5, fontWeight: 500 }}>{s.title}</span>
                <span style={{ fontSize: 9, padding: '1px 6px', borderRadius: 4, background: 'rgba(10,132,255,0.15)', color: '#0A84FF' }}>{s.lang}</span>
              </div>
              <div className="mono" style={{
                fontSize: 10, color: 'rgba(230,232,239,0.4)',
                whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
              }}>{s.code.split('\n')[0]}</div>
            </div>
          ))}
        </div>
      </div>

      {/* 右侧编辑+预览 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <div style={{
          padding: '10px 16px',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', alignItems: 'center', gap: 12,
        }}>
          <input
            defaultValue={notes[selected]?.title}
            style={{
              flex: 1, fontSize: 15, fontWeight: 600,
              background: 'transparent', border: 'none', outline: 'none',
              color: '#e6e8ef',
            }}
          />
          <div style={{ display: 'flex', gap: 6 }}>
            <button style={{
              padding: '5px 12px', fontSize: 11,
              background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
              borderRadius: 6, color: 'rgba(230,232,239,0.7)', cursor: 'pointer',
            }}>分享</button>
            <button style={{
              padding: '5px 12px', fontSize: 11,
              background: 'linear-gradient(135deg, #0A84FF, #0060DF)', border: 'none',
              borderRadius: 6, color: '#fff', cursor: 'pointer', fontWeight: 500,
            }}>保存</button>
          </div>
        </div>

        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          {/* 编辑区 */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', borderRight: '1px solid rgba(255,255,255,0.06)' }}>
            <div style={{
              padding: '6px 12px', fontSize: 10, fontWeight: 600,
              color: 'rgba(230,232,239,0.4)', textTransform: 'uppercase', letterSpacing: 0.5,
              background: 'rgba(0,0,0,0.1)',
            }}>
              编辑 Markdown
            </div>
            <textarea
              defaultValue={sampleContent}
              style={{
                flex: 1, padding: 16,
                background: 'transparent', border: 'none', outline: 'none',
                color: '#e6e8ef', fontSize: 13, lineHeight: 1.7,
                resize: 'none', fontFamily: 'JetBrains Mono, monospace',
              }}
            />
          </div>

          {/* 预览区 */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div style={{
              padding: '6px 16px', fontSize: 10, fontWeight: 600,
              color: 'rgba(230,232,239,0.4)', textTransform: 'uppercase', letterSpacing: 0.5,
              background: 'rgba(0,0,0,0.1)',
            }}>
              预览
            </div>
            <div style={{ flex: 1, padding: 16, overflow: 'auto' }}>
              {renderMarkdown(sampleContent)}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { TerminalModule, ProjectGit, APIDebug, KnowledgeBase });
