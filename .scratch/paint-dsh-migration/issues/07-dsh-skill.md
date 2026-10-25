# 07: DSH Skill + AI 生成图层

**What to build:**

AI 通过 DSH Skill（canvas-create-layer 等）操作画布，自动生成新图层，用户可查看和编辑。

**Blocked by:** 02, 05, 06

**Status:** resolved

- [x] 画布操作封装为 DSH Skill（SKILL.md）
- [x] AI 可通过 skill 创建新图层
- [x] 生成的图层可正常渲染和编辑

## Comments

- 提交：`cc78fdf`（skills + ops）、`4479512`（AI 图层在图层面板可见可编辑）。
- 机制：AI 不能直接调用 PAINT。约定它在最终答复里输出一段 ` ```paint-ops ` 代码块（JSON 数组），后端 `ai::ops` 解析 → action 白名单校验 → 逐条走 `command_dispatcher`（`actor = agent`）；无 ops 的答复是纯聊天。
- Skill：7 个 `canvas-*` SKILL.md（get-context / create-layer / set-layer-content / apply-filter / draw-shape / select-region / export-image）写在 `~/.paint/skills`，同步到 `$PAINT_HOME/harness/skills/canvas-*`；协议正文写进 `$DSH_HOME/AGENTS.md`（DSH 每次 run 自动注入）。同步是清单式幂等：内容不变不写盘、源删除则清理远端、用户禁用（改名 `*.disabled`）不会被"复活"。
- 安全：`MAX_OPS_PER_REPLY=20`；`delete_layer` 视为高风险，默认不执行，改为发 `ai://pending-op` 等用户确认；未知 action 整条拒绝并记入 `rejected`。
- 验证：`skills.rs` 10 个单测（frontmatter、id 唯一、同步/幂等/禁用/清理/列表分离/导入），`ai::ops` 9 个单测（三种 JSON 形状、忽略其它代码块、未知 action、超量、经 dispatcher 落地、单条失败不阻断、危险操作待确认）。AI 生成的图层带 `metadata.createdBy = agent`，前端图层面板显示 AI 角标并可正常编辑。
