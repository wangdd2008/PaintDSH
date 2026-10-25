# 08: 数字员工系统

**What to build:**

用户可以选择内置数字员工（图形设计师、滤镜专家、画布助手）或自定义员工作为 AI 角色。

**Blocked by:** 06

**Status:** resolved

- [x] 内置三类数字员工可用（图形设计师、滤镜专家、画布助手）
- [x] 自定义数字员工可创建和管理（名称、角色设定、技能、知识库）
- [x] 数字员工同步为 Harness agent preset

## Comments

- 提交：`cc78fdf`（`src-tauri/src/employees.rs` + 命令）、`4479512`（员工面板）。
- **第 3 项与规格的差异（实测结论）**：DSH 0.2.1 的 headless profile **没有 agent preset registry**，`.agent-presets/` 目录不存在。等价实现是把员工人设同步为 persona patch：
  `<PAINT_HOME>/harness/patches/employee-<id>.patch.yml`，内容覆盖 `system-prompt.personaPrefix`（可选 `agent-default-model`）；`HarnessEngine` 把它与沙箱策略合成同一个 `--patch` 文件传给 DSH，同时人设也拼进 prompt 首段。语义上等价于"选员工 = 选预设"，但落盘形态不同，已在契约 10.2 记录。
- 数据：`employees` 表（幂等建表），内置三条首次启动写入且不覆盖用户改动；`builtin` 行禁止删除；自定义员工支持增改删。
- 验证：`employees.rs` 6 个单测（seed 幂等、自定义 CRUD、内置保护、YAML 结构、同步与清理、persona 读取）。**未做**：真实 DSH 调用验证 personaPrefix 的实际提示词效果（无 API key）。
