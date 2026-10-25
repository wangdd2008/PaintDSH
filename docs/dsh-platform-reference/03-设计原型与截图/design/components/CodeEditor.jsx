// 代码编辑器模块
function CodeEditor() {
  const [activeTab, setActiveTab] = React.useState(0);
  const [showAIPanel, setShowAIPanel] = React.useState(true);

  const files = [
    { name: 'UserService.java', path: 'src/main/java/com/example/service/UserService.java', lang: 'java' },
    { name: 'UserController.java', path: 'src/main/java/com/example/controller/UserController.java', lang: 'java' },
    { name: 'AuthInterceptor.java', path: 'src/main/java/com/example/interceptor/AuthInterceptor.java', lang: 'java' },
    { name: 'application.yml', path: 'src/main/resources/application.yml', lang: 'yaml' },
  ];

  const fileTree = [
    { name: 'src', type: 'folder', expanded: true, children: [
      { name: 'main', type: 'folder', expanded: true, children: [
        { name: 'java', type: 'folder', expanded: true, children: [
          { name: 'com', type: 'folder', expanded: true, children: [
            { name: 'example', type: 'folder', expanded: true, children: [
              { name: 'controller', type: 'folder', expanded: false, children: [
                { name: 'UserController.java', type: 'file', lang: 'java' },
              ]},
              { name: 'service', type: 'folder', expanded: true, children: [
                { name: 'UserService.java', type: 'file', lang: 'java' },
              ]},
              { name: 'interceptor', type: 'folder', expanded: false, children: [
                { name: 'AuthInterceptor.java', type: 'file', lang: 'java' },
              ]},
              { name: 'repository', type: 'folder', expanded: false, children: [
                { name: 'UserRepository.java', type: 'file', lang: 'java' },
              ]},
              { name: 'entity', type: 'folder', expanded: false, children: [
                { name: 'User.java', type: 'file', lang: 'java' },
              ]},
            ]},
          ]},
        ]},
        { name: 'resources', type: 'folder', expanded: false, children: [
          { name: 'application.yml', type: 'file', lang: 'yaml' },
          { name: 'mapper', type: 'folder', expanded: false },
        ]},
      ]},
      { name: 'test', type: 'folder', expanded: false, children: [
        { name: 'java', type: 'folder', expanded: false },
      ]},
    ]},
    { name: 'pom.xml', type: 'file', lang: 'xml' },
    { name: 'Dockerfile', type: 'file', lang: 'docker' },
    { name: 'README.md', type: 'file', lang: 'md' },
  ];

  // 文件内容（带语法高亮的伪代码）
  const codeContents = [
    // UserService.java
    [
      { indent: 0, text: 'package', cls: 'code-keyword' }, { indent: 1, text: 'com.example.service;', cls: 'code-variable' },
      { indent: -1, text: '' },
      { indent: 0, text: 'import', cls: 'code-keyword' }, { indent: 1, text: 'org.springframework.stereotype.Service;', cls: 'code-variable' },
      { indent: 0, text: 'import', cls: 'code-keyword' }, { indent: 1, text: 'org.springframework.beans.factory.annotation.Autowired;', cls: 'code-variable' },
      { indent: 0, text: 'import', cls: 'code-keyword' }, { indent: 1, text: 'java.util.List;', cls: 'code-variable' },
      { indent: 0, text: 'import', cls: 'code-keyword' }, { indent: 1, text: 'java.util.Optional;', cls: 'code-variable' },
      { indent: -1, text: '' },
      { indent: 0, text: '@Service', cls: 'code-type' },
      { indent: 0, text: 'public', cls: 'code-keyword' },
      { indent: 1, text: 'class', cls: 'code-keyword' },
      { indent: 1, text: 'UserService', cls: 'code-type' },
      { indent: 1, text: '{' },
      { indent: -1, text: '' },
      { indent: 1, text: '@Autowired', cls: 'code-type' },
      { indent: 1, text: 'private', cls: 'code-keyword' },
      { indent: 2, text: 'UserRepository', cls: 'code-type' },
      { indent: 2, text: 'userRepository;', cls: 'code-variable' },
      { indent: -1, text: '' },
      { indent: 1, text: '// 根据ID查找用户', cls: 'code-comment' },
      { indent: 1, text: 'public', cls: 'code-keyword' },
      { indent: 2, text: 'Optional<User>', cls: 'code-type' },
      { indent: 2, text: 'findById', cls: 'code-function' },
      { indent: 2, text: '(Long id) {', cls: 'code-variable' },
      { indent: 2, text: 'return', cls: 'code-keyword' },
      { indent: 3, text: 'userRepository.findById(id);', cls: 'code-variable' },
      { indent: 1, text: '}' },
      { indent: -1, text: '' },
      { indent: 1, text: '// 获取所有用户', cls: 'code-comment' },
      { indent: 1, text: 'public', cls: 'code-keyword' },
      { indent: 2, text: 'List<User>', cls: 'code-type' },
      { indent: 2, text: 'findAll', cls: 'code-function' },
      { indent: 2, text: '() {', cls: '' },
      { indent: 2, text: 'return', cls: 'code-keyword' },
      { indent: 3, text: 'userRepository.findAll();', cls: 'code-variable' },
      { indent: 1, text: '}' },
      { indent: -1, text: '' },
      { indent: 1, text: '// 保存用户', cls: 'code-comment' },
      { indent: 1, text: 'public', cls: 'code-keyword' },
      { indent: 2, text: 'User', cls: 'code-type' },
      { indent: 2, text: 'save', cls: 'code-function' },
      { indent: 2, text: '(User user) {', cls: 'code-variable' },
      { indent: 2, text: '// 验证邮箱格式', cls: 'code-comment' },
      { indent: 2, text: 'if', cls: 'code-keyword' },
      { indent: 3, text: '(!isValidEmail(user.getEmail())) {', cls: 'code-variable' },
      { indent: 3, text: 'throw', cls: 'code-keyword' },
      { indent: 4, text: 'new', cls: 'code-keyword' },
      { indent: 4, text: 'IllegalArgumentException', cls: 'code-type' },
      { indent: 4, text: '("Invalid email format");', cls: 'code-string' },
      { indent: 2, text: '}' },
      { indent: 2, text: 'return', cls: 'code-keyword' },
      { indent: 3, text: 'userRepository.save(user);', cls: 'code-variable' },
      { indent: 1, text: '}' },
      { indent: -1, text: '' },
      { indent: 1, text: '// 删除用户', cls: 'code-comment' },
      { indent: 1, text: 'public', cls: 'code-keyword' },
      { indent: 2, text: 'void', cls: 'code-keyword' },
      { indent: 2, text: 'deleteById', cls: 'code-function' },
      { indent: 2, text: '(Long id) {', cls: 'code-variable' },
      { indent: 2, text: 'userRepository.deleteById(id);', cls: 'code-variable' },
      { indent: 1, text: '}' },
      { indent: -1, text: '' },
      { indent: 1, text: '// 邮箱验证', cls: 'code-comment' },
      { indent: 1, text: 'private', cls: 'code-keyword' },
      { indent: 2, text: 'boolean', cls: 'code-keyword' },
      { indent: 2, text: 'isValidEmail', cls: 'code-function' },
      { indent: 2, text: '(String email) {', cls: 'code-variable' },
      { indent: 2, text: 'String regex =', cls: 'code-variable' },
      { indent: 3, text: '"^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+$"', cls: 'code-string' },
      { indent: 2, text: ';', cls: '' },
      { indent: 2, text: 'return', cls: 'code-keyword' },
      { indent: 3, text: 'email.matches(regex);', cls: 'code-variable' },
      { indent: 1, text: '}' },
      { indent: -1, text: '' },
      { indent: 0, text: '}' },
    ],
  ];

  const aiSuggestions = [
    {
      title: '缓存优化建议',
      desc: '为 findById / findAll 方法添加 @Cacheable 注解，减少数据库查询',
      code: `@Cacheable(value = "users", key = "#id")\npublic Optional<User> findById(Long id) {\n    return userRepository.findById(id);\n}`,
      confidence: 95,
    },
    {
      title: '异常处理增强',
      desc: '使用自定义业务异常替代 IllegalArgumentException',
      code: `throw new UserNotFoundException(\n    "User not found with id: " + id\n);`,
      confidence: 88,
    },
  ];

  const [treeExpanded, setTreeExpanded] = React.useState({});

  // 终端输出
  const terminalLines = [
    { text: '➜  auth-service git:(feature/oauth2) ', type: 'prompt' },
    { text: 'mvn spring-boot:run', type: 'cmd' },
    { text: '', type: '' },
    { text: '[INFO] Scanning for projects...', type: 'info' },
    { text: '[INFO] ', type: 'info' },
    { text: '[INFO] ------------------< com.example:auth-service >------------------', type: 'info' },
    { text: '[INFO] Building auth-service 1.0.0-SNAPSHOT', type: 'info' },
    { text: '[INFO] --------------------------------[ jar ]---------------------------------', type: 'info' },
    { text: '[INFO] ', type: 'info' },
    { text: '[INFO] >>> spring-boot-maven-plugin:3.2.0:run (default-cli) > test-compile @ auth-service >>>', type: 'info' },
    { text: '[INFO] ', type: 'info' },
    { text: '[INFO] --- maven-resources-plugin:3.3.1:resources (default-resources) @ auth-service ---', type: 'info' },
    { text: '[INFO] Copying 2 resources from src/main/resources', type: 'info' },
    { text: '', type: '' },
    { text: "  .   ____          _            __ _ _", type: 'spring' },
    { text: " /\\\\ / ___  _ _(_)_ __  __ _  \\ \\ \\ \\  ", type: 'spring' },
    { text: "( ( )\\___  _  _| | _ \\/ _  |  \\ \\ \\ \\  ", type: 'spring' },
    { text: " \\\\/  ___)| |_)| | | | | || (_| |  ) ) ) )", type: 'spring' },
    { text: "     |____| .__|_| |_|_| |_\\__, | / / / /", type: 'spring' },
    { text: " =========|_|==============|___/=/_/_/_/", type: 'spring' },
    { text: " :: Spring Boot ::                (v3.2.0)", type: 'spring' },
    { text: '', type: '' },
    { text: '2024-01-15T10:23:45.234+08:00  INFO 3421 --- [           main] com.example.AuthApplication              : Starting AuthApplication using Java 17.0.9 on MacBook-Pro with PID 3421', type: 'log' },
    { text: '2024-01-15T10:23:45.237+08:00  INFO 3421 --- [           main] com.example.AuthApplication              : No active profile set, falling back to 1 default profile: "default"', type: 'log' },
    { text: '2024-01-15T10:23:46.102+08:00  INFO 3421 --- [           main] o.s.b.w.embedded.tomcat.TomcatWebServer  : Tomcat initialized with port 8080 (http)', type: 'log' },
    { text: '2024-01-15T10:23:46.876+08:00  INFO 3421 --- [           main] com.example.AuthApplication              : Started AuthApplication in 2.104 seconds (process running for 2.567)', type: 'success' },
  ];

  // 渲染代码行
  const renderCodeLine = (line, lineNum) => {
    if (line.indent === -1) {
      return (
        <div key={lineNum} style={{ display: 'flex' }}>
          <span className="mono" style={lineNumStyle}>{''}</span>
          <span style={{ height: 20 }}>&nbsp;</span>
        </div>
      );
    }
    return (
      <div key={lineNum} style={{ display: 'flex', lineHeight: '20px' }}>
        <span className="mono" style={lineNumStyle}>{lineNum + 1}</span>
        <span style={{ paddingLeft: 4 + line.indent * 16 }} className={'mono ' + line.cls}>
          {line.text}
        </span>
      </div>
    );
  };

  const [termInput, setTermInput] = React.useState('');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* 标签页栏 */}
      <div style={{
        display: 'flex', alignItems: 'stretch',
        background: 'rgba(20,22,36,0.6)',
        borderBottom: '1px solid rgba(255,255,255,0.06)',
        padding: '0 8px',
        minHeight: 36,
      }}>
        {files.map((f, i) => (
          <div
            key={i}
            onClick={() => setActiveTab(i)}
            style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '0 14px', minWidth: 140,
              cursor: 'pointer',
              borderBottom: activeTab === i ? '2px solid #0A84FF' : '2px solid transparent',
              background: activeTab === i ? 'rgba(30,32,48,0.9)' : 'transparent',
              marginRight: 2,
              borderRadius: '6px 6px 0 0',
            }}
          >
            <span style={{ fontSize: 11, color: activeTab === i ? '#e6e8ef' : 'rgba(230,232,239,0.5)' }}>{f.name}</span>
            {activeTab === i && (
              <div style={{
                width: 14, height: 14, borderRadius: 3,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <Icon name="close" size={10} color="rgba(230,232,239,0.4)" />
              </div>
            )}
          </div>
        ))}
      </div>

      {/* 主编辑区 */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* 文件树 */}
        <div style={{
          width: 220, flexShrink: 0,
          background: 'rgba(20,22,36,0.4)',
          borderRight: '1px solid rgba(255,255,255,0.06)',
          padding: '8px 0',
          overflow: 'auto',
          fontSize: 12,
        }}>
          <div style={{ padding: '4px 12px 8px', fontSize: 10, color: 'rgba(230,232,239,0.4)', fontWeight: 600, textTransform: 'uppercase', letterSpacing: 0.5 }}>
            资源管理器
          </div>
          <FileTree items={fileTree} level={0} />
        </div>

        {/* 代码编辑区 + AI面板 */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          {/* 代码区 */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            <div style={{ flex: 1, overflow: 'auto', padding: '12px 0' }}>
              {codeContents[0].map((line, i) => renderCodeLine(line, i))}
            </div>

            {/* AI 代码建议面板 */}
            {showAIPanel && (
              <div style={{
                width: 280, flexShrink: 0,
                background: 'rgba(25,28,45,0.6)',
                borderLeft: '1px solid rgba(255,255,255,0.06)',
                padding: 12,
                overflow: 'auto',
                display: 'flex', flexDirection: 'column', gap: 10,
              }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <div style={{
                      width: 20, height: 20, borderRadius: 6,
                      background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
                      display: 'flex', alignItems: 'center', justifyContent: 'center',
                    }}>
                      <Icon name="sparkles" size={12} color="#fff" />
                    </div>
                    <span style={{ fontSize: 12, fontWeight: 600 }}>AI 建议</span>
                  </div>
                  <button onClick={() => setShowAIPanel(false)} style={{
                    background: 'transparent', border: 'none', cursor: 'pointer',
                    padding: 2, color: 'rgba(230,232,239,0.4)',
                  }}>
                    <Icon name="close" size={14} />
                  </button>
                </div>

                {aiSuggestions.map((s, i) => (
                  <div key={i} style={{
                    background: 'rgba(255,255,255,0.04)',
                    borderRadius: 10, padding: 10,
                    border: '1px solid rgba(255,255,255,0.06)',
                  }}>
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                      <span style={{ fontSize: 11.5, fontWeight: 600 }}>{s.title}</span>
                      <span style={{
                        fontSize: 9, padding: '1px 6px', borderRadius: 8,
                        background: 'rgba(48,209,88,0.15)', color: '#30D158',
                      }}>{s.confidence}%</span>
                    </div>
                    <p style={{ fontSize: 10.5, color: 'rgba(230,232,239,0.5)', marginBottom: 8 }}>{s.desc}</p>
                    <pre className="mono" style={{
                      fontSize: 10, padding: 8, borderRadius: 6,
                      background: 'rgba(0,0,0,0.3)',
                      color: '#c3e88d', lineHeight: 1.5,
                      whiteSpace: 'pre-wrap',
                    }}>{s.code}</pre>
                    <div style={{ display: 'flex', gap: 6, marginTop: 8 }}>
                      <button style={{
                        flex: 1, padding: '5px 0', fontSize: 10.5,
                        background: 'rgba(10,132,255,0.2)', color: '#0A84FF',
                        border: 'none', borderRadius: 6, cursor: 'pointer', fontWeight: 500,
                      }}>应用</button>
                      <button style={{
                        flex: 1, padding: '5px 0', fontSize: 10.5,
                        background: 'rgba(255,255,255,0.06)', color: 'rgba(230,232,239,0.6)',
                        border: 'none', borderRadius: 6, cursor: 'pointer',
                      }}>解释</button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* 迷你终端 */}
          <div style={{
            height: 180, flexShrink: 0,
            background: 'rgba(10,12,22,0.8)',
            borderTop: '1px solid rgba(255,255,255,0.06)',
            display: 'flex', flexDirection: 'column',
          }}>
            <div style={{
              display: 'flex', alignItems: 'center', gap: 12,
              padding: '6px 12px', fontSize: 11,
              borderBottom: '1px solid rgba(255,255,255,0.04)',
            }}>
              <span style={{ color: '#e6e8ef', fontWeight: 500 }}>终端</span>
              <span style={{ color: 'rgba(230,232,239,0.4)' }}>问题</span>
              <span style={{ color: 'rgba(230,232,239,0.4)' }}>输出</span>
              <span style={{ color: 'rgba(230,232,239,0.4)' }}>调试控制台</span>
              <div style={{ flex: 1 }} />
              <Icon name="plus" size={14} color="rgba(230,232,239,0.4)" />
              <Icon name="trash" size={14} color="rgba(230,232,239,0.4)" />
              <Icon name="split" size={14} color="rgba(230,232,239,0.4)" />
            </div>
            <div style={{ flex: 1, overflow: 'auto', padding: '6px 12px', fontSize: 11, lineHeight: 1.6 }} className="mono">
              {terminalLines.map((line, i) => (
                <div key={i} style={{
                  color: line.type === 'prompt' ? '#30D158'
                       : line.type === 'success' ? '#30D158'
                       : line.type === 'error' ? '#FF6B6B'
                       : line.type === 'spring' ? '#6DB33F'
                       : line.type === 'log' ? 'rgba(230,232,239,0.7)'
                       : 'rgba(230,232,239,0.6)',
                }}>{line.text}</div>
              ))}
              <div style={{ display: 'flex', alignItems: 'center', marginTop: 2 }}>
                <span style={{ color: '#30D158' }}>➜</span>
                <span style={{ color: '#569CD6', marginLeft: 6 }}>auth-service</span>
                <span style={{ color: '#9CDCFE', marginLeft: 4 }}>git:</span>
                <span style={{ color: '#E76F51', marginLeft: 2 }}>(feature/oauth2)</span>
                <input
                  value={termInput}
                  onChange={e => setTermInput(e.target.value)}
                  style={{
                    flex: 1, marginLeft: 8,
                    background: 'transparent', border: 'none', outline: 'none',
                    color: '#e6e8ef', fontFamily: 'JetBrains Mono, monospace',
                    fontSize: 11,
                  }}
                  placeholder="输入命令..."
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

const lineNumStyle = {
  display: 'inline-block',
  width: 44,
  textAlign: 'right',
  paddingRight: 12,
  color: 'rgba(230,232,239,0.25)',
  fontSize: 11.5,
  userSelect: 'none',
  flexShrink: 0,
};

// 递归文件树
function FileTree({ items, level }) {
  const [expanded, setExpanded] = React.useState({});
  const toggle = (name) => setExpanded(prev => ({ ...prev, [name]: !prev[name] }));
  const isExpanded = (name) => expanded[name] !== undefined ? expanded[name] : (level < 2);

  return (
    <div style={{ fontSize: 12 }}>
      {items.map((item, i) => (
        <div key={i}>
          <div
            onClick={() => item.type === 'folder' && toggle(item.name)}
            style={{
              display: 'flex', alignItems: 'center', gap: 4,
              padding: '3px 8px 3px ' + (level * 12 + 8) + 'px',
              cursor: item.type === 'file' ? 'pointer' : 'default',
              color: item.type === 'file' ? 'rgba(230,232,239,0.75)' : 'rgba(230,232,239,0.85)',
              borderRadius: 4,
            }}
          >
            {item.type === 'folder' ? (
              <>
                <Icon name={isExpanded(item.name) ? 'chevronDown' : 'chevronRight'} size={10} color="rgba(230,232,239,0.4)" />
                <Icon name="folder" size={14} color="#FFD60A" />
              </>
            ) : (
              <>
                <span style={{ width: 10 }} />
                <Icon name="file" size={14} color="rgba(230,232,239,0.5)" />
              </>
            )}
            <span style={{ fontSize: 11.5, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {item.name}
            </span>
          </div>
          {item.type === 'folder' && item.children && isExpanded(item.name) && (
            <FileTree items={item.children} level={level + 1} />
          )}
        </div>
      ))}
    </div>
  );
}

Object.assign(window, { CodeEditor });
