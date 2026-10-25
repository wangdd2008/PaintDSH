# 学习记录 0003：AI 网关设计

## 学习时间

2026-10-25

## 学习内容

- AI 网关的六大职责（上下文装配、引擎调度、并发控制、Token 计量、审计追踪、SSE 解析）
- 双引擎调度逻辑（harness vs direct）
- 并发控制：Semaphore(2) 的实现和原因
- Token 计量与审计追踪（ai_calls 表 + ai_usage_daily 表）
- SSE 流式解析（parse_sse_line）
- 事件驱动架构（ai://delta/done/error）

## 关键洞察

1. **AI 网关是"交通指挥中心"**：所有 AI 调用的唯一入口，统一管理和调度
2. **双引擎互补**：Harness 用于复杂任务（带工具链），Direct 用于简单对话（流式体验好）
3. **并发控制分层**：Harness 路径用全局锁（Mutex），Direct 路径用 Semaphore(2)
4. **计量双表设计**：ai_calls 详细记录 + ai_usage_daily 按日汇总，兼顾灵活性和性能
5. **SSE 协议细节**：content/reasoning 逐条推送，usage 只在最后一条返回

## 待深化问题

- 安全沙箱的详细实现（read-only 如何在系统层面生效）
- Web UI 模式的品牌定制和主题同步
- 数字员工作为 preset 的同步机制

## 下一步

进入第四课：安全沙箱与数据互通（DSH_HOME 共享、工作区注册、preset/skill 同步）
