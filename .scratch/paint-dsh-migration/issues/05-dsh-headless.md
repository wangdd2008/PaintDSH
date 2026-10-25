# 05: DSH Headless 集成 + 自定义安全沙箱

**What to build:**

用户可以通过 AI 面板输入自然语言指令，DSH headless 子进程在后端执行，自定义 canvas-write 安全沙箱策略生效。

**Blocked by:** 01

**Status:** resolved

- [x] DSH headless 子进程可正常启动和执行任务
- [x] 自定义 canvas-write 安全沙箱策略生效（允许画布写入，禁止危险命令）
- [x] 超时看门狗和全局锁机制可用

## Comments

- 提交：`cc78fdf`。文件：`src-tauri/src/harness.rs`。
- 与规格假设的差异（依据本机真实 DSH 0.2.1-rc.3 实测）：
  - 入口是 `node <pkg>/lib/bin.js`（不 spawn `.cmd` 垫片），`--profile headless --patch <yml> --json <task>`，`--patch` 必须在 task 之前；
  - 事件流是 NDJSON，token 计量取 `status/step_end.usage`（`inputTokens`/`outputTokens` 可缺省），成败看退出码 + `turn_end.reason.kind`；
  - **沙箱是文件写边界，不是命令黑名单**：`canvas-write`（`mode: workspace-write` + `defaultPreset` + `approval: never`）能限制写入范围到工作区，但无法阻止工作区内的 `rm`。真正的"危险命令禁止"由第 2 项之外的机制承担：AI 没有执行 PAINT 操作的能力，只能输出 ```paint-ops 由 `ai::ops` 按 action 白名单执行（ticket 07）。
- 稳健性：stdout 用独立线程读 + 主线程 100ms 轮询，保证子进程静默时看门狗仍能触发；超时/取消用 `taskkill /T /F` 杀整棵树（避免 pwsh 孤儿）；`RUN_LOCK` 全局串行，并发请求直接 `busy`；stderr 单独消费避免管道写满死锁。
- 验证：`harness.rs` 12 个单测（参数顺序、两套 patch 内容、patch 幂等、NDJSON 解析、token 汇总、成功判定、路径解析、真实子进程回放、看门狗 300ms 杀树、并发 busy）。**未做**：真实 `dsh --profile headless` 端到端调用（需要 DEEPSEEK_API_KEY，本机未配置）。
