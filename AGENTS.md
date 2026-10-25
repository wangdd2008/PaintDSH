# AGENTS.md

所有材料语言必须为中文。

## 先读这些

| 要做什么 | 读哪里 |
| --- | --- |
| 改代码（IPC / 类型 / 存储 / 依赖） | `docs/engineering-contract.md`：§10.1 依赖钉版（**禁止 `cargo update`**）、§10.2 实测差异、§11 本机环境 |
| 跑命令、查环境变量 | `README.md` §4 / §5 / §9 |
| 启动窗口、链接失败、端口被占 | `docs/runbook-dev.md` |
| 理解模块边界或某个决策 | `docs/architecture-design.md`、`docs/adr/` |
| 确认术语 | `GLOSSARY.md` |

## Agent skills

### Issue tracker

Issues 以本地 markdown 文件形式存储在 `.scratch/<feature>/` 目录下。详见 `docs/agents/issue-tracker.md`。

### Triage labels

使用默认的五个标签：needs-triage、needs-info、ready-for-agent、ready-for-human、wontfix。详见 `docs/agents/triage-labels.md`。

### Domain docs

单仓库（single-context）布局。详见 `docs/agents/domain.md`。
