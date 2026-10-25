# 06: AI 网关

**What to build:**

所有 AI 调用经过统一网关，可自动选择引擎（harness/direct），Token 消耗实时计量，审计日志完整记录。

**Blocked by:** 05

**Status:** ready-for-agent

- [ ] AI 网关可正确选择引擎（harness/direct）
- [ ] Token 计量准确记录（ai_calls 表）
- [ ] 并发控制（最多 2 个在途请求）和审计日志可用
