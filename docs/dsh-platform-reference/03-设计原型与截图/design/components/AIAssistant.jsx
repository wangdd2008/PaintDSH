// AI 智能助手模块
function AIAssistant({ compact = false }) {
  const [messages, setMessages] = React.useState([
    {
      id: 1, role: 'ai',
      content: '你好！我是 DevStation AI 助手 🤖\n\n我可以帮你：\n- 解释和分析代码\n- 发现并修复 Bug\n- 性能优化建议\n- 生成单元测试\n- 编写文档注释\n- 代码重构建议\n\n有什么我可以帮你的吗？',
      time: '10:24',
    },
    {
      id: 2, role: 'user',
      content: '帮我看看这段 Java 代码有什么可以优化的地方：\n\n```java\npublic List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    for (User user : users) {\n        user.setPassword(null);\n    }\n    return users;\n}\n```',
      time: '10:25',
      code: 'public List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    for (User user : users) {\n        user.setPassword(null);\n    }\n    return users;\n}',
    },
    {
      id: 3, role: 'ai',
      content: `这段代码有几个可以优化的点：\n\n### 1. 使用 Stream API 提升可读性\n可以用 Java 8 Stream API 替代传统 for 循环，代码更简洁：\n\n~~~java\npublic List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    users.forEach(user -> user.setPassword(null));\n    return users;\n}\n~~~\n\n### 2. 添加缓存（推荐）\nfindAll 通常是高频查询，建议加缓存减少数据库压力：\n\n~~~java\n@Cacheable(value = "users", key = "'all'")\npublic List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    users.forEach(user -> user.setPassword(null));\n    return users;\n}\n~~~\n\n### 3. 使用 DTO 替代直接修改实体\n修改实体对象可能导致脏数据问题，更好的做法是使用 DTO：\n\n~~~java\npublic List<UserDTO> findAll() {\n    return userRepository.findAll().stream()\n        .map(user -> modelMapper.map(user, UserDTO.class))\n        .collect(Collectors.toList());\n}\n~~~\n\n需要我详细解释任何一个方案吗？`,
      time: '10:25',
      codeBlocks: [
        'public List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    users.forEach(user -> user.setPassword(null));\n    return users;\n}',
        '@Cacheable(value = "users", key = "\'all\'")\npublic List<User> findAll() {\n    List<User> users = userRepository.findAll();\n    users.forEach(user -> user.setPassword(null));\n    return users;\n}',
        'public List<UserDTO> findAll() {\n    return userRepository.findAll().stream()\n        .map(user -> modelMapper.map(user, UserDTO.class))\n        .collect(Collectors.toList());\n}',
      ],
    },
  ]);

  const [input, setInput] = React.useState('');
  const messagesEndRef = React.useRef(null);

  React.useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  const quickActions = [
    { icon: 'doc', name: '代码解释', color: '#0A84FF' },
    { icon: 'bug', name: 'Bug 修复', color: '#FF6B6B' },
    { icon: 'zap', name: '性能优化', color: '#FFD60A' },
    { icon: 'test', name: '单元测试', color: '#30D158' },
    { icon: 'edit', name: '生成文档', color: '#BF5AF2' },
    { icon: 'refresh', name: '代码重构', color: '#FF9F0A' },
  ];

  const contextFiles = [
    { name: 'UserService.java', path: 'service/', lang: 'Java' },
    { name: 'UserController.java', path: 'controller/', lang: 'Java' },
    { name: 'auth-service', path: 'project/', lang: '项目' },
  ];

  const sendMessage = () => {
    if (!input.trim()) return;
    const now = new Date();
    const timeStr = now.getHours().toString().padStart(2, '0') + ':' + now.getMinutes().toString().padStart(2, '0');
    const userMsg = { id: Date.now(), role: 'user', content: input, time: timeStr };
    setMessages(prev => [...prev, userMsg]);
    setInput('');

    // 模拟 AI 回复
    setTimeout(() => {
      const aiMsg = {
        id: Date.now() + 1,
        role: 'ai',
        content: '好的，我来帮你分析这个问题。根据你描述的情况，可能有以下几个原因：\n\n1. **数据库连接池配置不当** - 检查 HikariCP 的最大连接数设置\n2. **N+1 查询问题** - 确认是否有循环内查询的情况\n3. **缺少索引** - 检查慢查询日志，添加适当的索引\n\n需要我帮你生成具体的优化代码吗？',
        time: timeStr,
      };
      setMessages(prev => [...prev, aiMsg]);
    }, 1000);
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  if (compact) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: 4 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '0 4px 8px' }}>
          <div style={{
            width: 20, height: 20, borderRadius: 6,
            background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <Icon name="sparkles" size={12} color="#fff" />
          </div>
          <span style={{ fontSize: 13, fontWeight: 600 }}>AI 助手</span>
        </div>
        <div style={{ flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 8, paddingRight: 4 }}>
          {messages.slice(-2).map(msg => (
            <div key={msg.id} style={{
              display: 'flex', flexDirection: 'column',
              alignItems: msg.role === 'user' ? 'flex-end' : 'flex-start',
            }}>
              <div style={{
                maxWidth: '90%', padding: '8px 12px', borderRadius: 12,
                background: msg.role === 'user'
                  ? 'rgba(10,132,255,0.25)'
                  : 'rgba(255,255,255,0.06)',
                fontSize: 11, lineHeight: 1.5,
                borderTopRightRadius: msg.role === 'user' ? 4 : 12,
                borderTopLeftRadius: msg.role === 'user' ? 12 : 4,
              }}>
                <span style={{
                  display: '-webkit-box', WebkitLineClamp: 3, WebkitBoxOrient: 'vertical',
                  overflow: 'hidden',
                }}>{msg.content}</span>
              </div>
            </div>
          ))}
        </div>
        <div style={{ marginTop: 8, display: 'flex', gap: 6 }}>
          <input
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="问 AI 任何问题..."
            style={{
              flex: 1, padding: '7px 12px', fontSize: 11,
              background: 'rgba(255,255,255,0.06)',
              border: '1px solid rgba(255,255,255,0.08)',
              borderRadius: 8, color: '#e6e8ef', outline: 'none',
            }}
          />
          <button onClick={sendMessage} style={{
            width: 30, height: 30, borderRadius: 8,
            background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
            border: 'none', cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <Icon name="send" size={14} color="#fff" />
          </button>
        </div>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* 主聊天区 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        {/* 快捷指令栏 */}
        <div style={{
          padding: '10px 16px',
          borderBottom: '1px solid rgba(255,255,255,0.06)',
          display: 'flex', gap: 8, overflow: 'auto',
          background: 'rgba(20,22,36,0.4)',
        }}>
          {quickActions.map((a, i) => (
            <button key={i}
              onClick={() => setInput(`请帮我${a.name}`)}
              style={{
                display: 'flex', alignItems: 'center', gap: 6,
                padding: '6px 12px', borderRadius: 8,
                background: 'rgba(255,255,255,0.05)',
                border: '1px solid rgba(255,255,255,0.08)',
                color: 'rgba(230,232,239,0.75)', fontSize: 12,
                cursor: 'pointer', flexShrink: 0,
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = a.color + '20';
                e.currentTarget.style.borderColor = a.color + '50';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(255,255,255,0.05)';
                e.currentTarget.style.borderColor = 'rgba(255,255,255,0.08)';
              }}
            >
              <Icon name={a.icon} size={14} color={a.color} />
              {a.name}
            </button>
          ))}
        </div>

        {/* 消息列表 */}
        <div style={{ flex: 1, overflow: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>
          {messages.map(msg => (
            <div key={msg.id} className="fade-in" style={{
              display: 'flex', flexDirection: 'column',
              alignItems: msg.role === 'user' ? 'flex-end' : 'flex-start',
              maxWidth: '80%', alignSelf: msg.role === 'user' ? 'flex-end' : 'flex-start',
            }}>
              <div style={{ display: 'flex', gap: 8, alignItems: 'flex-end', flexDirection: msg.role === 'user' ? 'row-reverse' : 'row' }}>
                {msg.role === 'ai' && (
                  <div style={{
                    width: 28, height: 28, borderRadius: 8, flexShrink: 0,
                    background: 'linear-gradient(135deg, #0A84FF, #BF5AF2)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                  }}>
                    <Icon name="sparkles" size={14} color="#fff" />
                  </div>
                )}
                <div style={{
                  padding: '10px 14px', borderRadius: 14,
                  background: msg.role === 'user'
                    ? 'rgba(10,132,255,0.25)'
                    : 'rgba(255,255,255,0.06)',
                  border: msg.role === 'user' ? '1px solid rgba(10,132,255,0.3)' : '1px solid rgba(255,255,255,0.06)',
                  fontSize: 13, lineHeight: 1.65,
                  borderTopRightRadius: msg.role === 'user' ? 4 : 14,
                  borderTopLeftRadius: msg.role === 'user' ? 14 : 4,
                }}>
                  {msg.content.split(/```|~~~\w*/).map((part, i) => {
                    if (i % 2 === 1) {
                      // 代码块
                      const [lang, ...codeLines] = part.trim().split('\n');
                      const code = codeLines.join('\n');
                      return (
                        <div key={i} style={{ marginTop: 8, marginBottom: 8 }}>
                          <div style={{
                            display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                            padding: '4px 10px', fontSize: 10,
                            background: 'rgba(0,0,0,0.3)',
                            borderRadius: '6px 6px 0 0',
                            color: 'rgba(230,232,239,0.5)',
                          }}>
                            <span>{lang}</span>
                            <button style={{
                              background: 'transparent', border: 'none',
                              color: 'rgba(230,232,239,0.5)', cursor: 'pointer',
                              fontSize: 10, display: 'flex', alignItems: 'center', gap: 4,
                            }}>
                              <Icon name="copy" size={10} /> 复制
                            </button>
                          </div>
                          <pre className="mono" style={{
                            padding: 12, margin: 0,
                            background: 'rgba(0,0,0,0.35)',
                            borderRadius: '0 0 6px 6px',
                            fontSize: 11.5, lineHeight: 1.6,
                            color: '#c3e88d',
                            overflow: 'auto',
                            whiteSpace: 'pre-wrap',
                            wordBreak: 'break-all',
                          }}>{code}</pre>
                        </div>
                      );
                    }
                    // 普通文本
                    const lines = part.trim().split('\n').filter(l => l);
                    return (
                      <div key={i}>
                        {lines.map((line, li) => {
                          if (line.startsWith('### ')) {
                            return <div key={li} style={{ fontWeight: 600, marginTop: 8, marginBottom: 4 }}>{line.replace('### ', '')}</div>;
                          }
                          if (line.startsWith('- ')) {
                            return <div key={li} style={{ display: 'flex', gap: 6, paddingLeft: 4 }}>
                              <span style={{ color: '#0A84FF' }}>•</span>
                              <span dangerouslySetInnerHTML={{ __html: formatInlineCode(line.replace('- ', '')) }} />
                            </div>;
                          }
                          if (/^\d+\./.test(line)) {
                            return <div key={li} style={{ display: 'flex', gap: 6, paddingLeft: 4 }}>
                              <span style={{ color: '#BF5AF2', fontWeight: 500 }}>{line.match(/^\d+/)[0]}.</span>
                              <span dangerouslySetInnerHTML={{ __html: formatInlineCode(line.replace(/^\d+\.\s*/, '')) }} />
                            </div>;
                          }
                          return <div key={li} dangerouslySetInnerHTML={{ __html: formatInlineCode(line) }} />;
                        })}
                      </div>
                    );
                  })}
                </div>
                {msg.role === 'user' && (
                  <div style={{
                    width: 28, height: 28, borderRadius: 8, flexShrink: 0,
                    background: 'rgba(48,209,88,0.2)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                  }}>
                    <Icon name="user" size={14} color="#30D158" />
                  </div>
                )}
              </div>
              <span style={{
                fontSize: 10, color: 'rgba(230,232,239,0.35)',
                marginTop: 4, paddingLeft: msg.role === 'ai' ? 36 : 0,
                paddingRight: msg.role === 'user' ? 36 : 0,
              }}>{msg.time}</span>
            </div>
          ))}
          <div ref={messagesEndRef} />
        </div>

        {/* 输入框 */}
        <div style={{ padding: '12px 16px', borderTop: '1px solid rgba(255,255,255,0.06)' }}>
          <div style={{
            display: 'flex', alignItems: 'flex-end', gap: 8,
            background: 'rgba(255,255,255,0.05)',
            border: '1px solid rgba(255,255,255,0.08)',
            borderRadius: 12, padding: 10,
          }}>
            <textarea
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="输入消息... (Enter 发送，Shift+Enter 换行)"
              rows={1}
              style={{
                flex: 1, resize: 'none', border: 'none', outline: 'none',
                background: 'transparent', color: '#e6e8ef',
                fontSize: 13, lineHeight: 1.5, fontFamily: 'inherit',
                maxHeight: 120,
              }}
            />
            <button onClick={sendMessage} style={{
              width: 36, height: 36, borderRadius: 10,
              background: input.trim()
                ? 'linear-gradient(135deg, #0A84FF, #BF5AF2)'
                : 'rgba(255,255,255,0.1)',
              border: 'none', cursor: input.trim() ? 'pointer' : 'not-allowed',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              flexShrink: 0,
              transition: 'all 0.2s',
            }}>
              <Icon name="send" size={16} color={input.trim() ? '#fff' : 'rgba(230,232,239,0.3)'} />
            </button>
          </div>
        </div>
      </div>

      {/* 右侧上下文面板 */}
      <div style={{
        width: 240, flexShrink: 0,
        borderLeft: '1px solid rgba(255,255,255,0.06)',
        background: 'rgba(20,22,36,0.4)',
        display: 'flex', flexDirection: 'column',
      }}>
        <div style={{
          padding: '12px 14px', fontSize: 11, fontWeight: 600,
          color: 'rgba(230,232,239,0.5)',
          textTransform: 'uppercase', letterSpacing: 0.5,
          borderBottom: '1px solid rgba(255,255,255,0.06)',
        }}>
          当前上下文
        </div>
        <div style={{ padding: 8, flex: 1, overflow: 'auto' }}>
          <div style={{ fontSize: 11, color: 'rgba(230,232,239,0.4)', padding: '4px 8px 8px' }}>
            已读取 {contextFiles.length} 个文件
          </div>
          {contextFiles.map((f, i) => (
            <div key={i} style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '8px 10px', borderRadius: 8,
              marginBottom: 2,
            }}>
              <Icon name="file" size={14} color="rgba(230,232,239,0.4)" />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 11.5, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</div>
                <div style={{ fontSize: 10, color: 'rgba(230,232,239,0.35)' }}>{f.path}</div>
              </div>
              <span style={{
                fontSize: 9, padding: '1px 6px', borderRadius: 4,
                background: 'rgba(255,255,255,0.06)',
                color: 'rgba(230,232,239,0.5)',
              }}>{f.lang}</span>
            </div>
          ))}
        </div>
        <div style={{ padding: 10, borderTop: '1px solid rgba(255,255,255,0.06)' }}>
          <button style={{
            width: '100%', padding: '8px 0', fontSize: 11,
            background: 'rgba(10,132,255,0.15)', color: '#0A84FF',
            border: '1px solid rgba(10,132,255,0.3)', borderRadius: 8,
            cursor: 'pointer',
          }}>
            + 添加上下文
          </button>
        </div>
      </div>
    </div>
  );
}

// 行内代码格式化
function formatInlineCode(text) {
  return text.replace(/`([^`]+)`/g, '<code style="background: rgba(255,255,255,0.1); padding: 1px 5px; border-radius: 4px; font-family: JetBrains Mono, monospace; font-size: 0.9em; color: #c3e88d;">$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong style="font-weight: 600;">$1</strong>');
}

Object.assign(window, { AIAssistant });
