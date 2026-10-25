# 05: DSH Headless 集成 + 自定义安全沙箱

**What to build:**

用户可以通过 AI 面板输入自然语言指令，DSH headless 子进程在后端执行，自定义 canvas-write 安全沙箱策略生效。

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] DSH headless 子进程可正常启动和执行任务
- [ ] 自定义 canvas-write 安全沙箱策略生效（允许画布写入，禁止危险命令）
- [ ] 超时看门狗和全局锁机制可用
