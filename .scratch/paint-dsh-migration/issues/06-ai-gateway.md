# 06: AI 网关

**What to build:**

所有 AI 调用经过统一网关，可自动选择引擎（harness/direct），Token 消耗实时计量，审计日志完整记录。

**Blocked by:** 05

**Status:** resolved

- [x] AI 网关可正确选择引擎（harness/direct）
- [x] Token 计量准确记录（ai_calls 表）
- [x] 并发控制（最多 2 个在途请求）和审计日志可用

## Comments

- 提交：`cc78fdf`。文件：`src-tauri/src/ai/{gateway,engines,context,repository}.rs`。
- 引擎选择：显式指定 > `module=employee` > 画布类关键词（画/图层/滤镜/导出…）→ harness；其余 → direct；首选引擎不可用时自动降级，两者都不可用报 `ai`。
- 并发：Direct 用自建计数闸门 `CountingGate(2)`（15s 等待上限，超时 `busy`）；Harness 由 `harness::RUN_LOCK` 串行（DSH 单任务特性）。
- 计量与审计：`ai_sessions` / `ai_messages` / `ai_calls` 三表；每次调用记录 module、employee、engine、model、tokens、latency、success、error；`ai_usage_summary(days)` 用 SQL 按日聚合 token 与失败数，不额外建表。
- 事件：`ai://delta`（内容/推理分流）、`ai://done`（含 token 与 latency）、`ai://error`（独立事件，绝不塞进 done）。
- `ai_chat_start` 立即返回 `{sessionId, engine, model}`，真实执行在后台线程（用独立 SQLite 连接，WAL + busy_timeout 兜底），避免阻塞 UI。
- 验证：`ai::gateway` 11 个单测（启发式、显式优先级、降级、流式事件、审计、失败路径、计数闸门跨线程释放、上下文注入），`ai::repository` 4 个，`ai::engines` 6 个（SSE 解析、usage、空 chunk、失败映射）。**未做**：真实 DeepSeek API 调用（无 key）。
