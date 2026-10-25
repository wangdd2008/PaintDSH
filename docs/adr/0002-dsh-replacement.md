# 从 Codex 替换为 DeepSeek Harness（DSH）

PAINT 的 AI 引擎从 OpenAI Codex CLI 全面替换为 DeepSeek Harness（DSH）。DSH 提供 headless 子进程 + Web UI 子进程双模式，自带 bash/fs/editor 工具链，以及 AI 网关、Token 计量、安全沙箱等完整能力。Codex 仅支持代码生成，而 DSH 提供的是面向研发的完整 AI 运行时。替换后，PAINT 可以利用 DSH 的 skill 生态、数字员工、知识库等高级特性。

**考虑过的选项**：保留 Codex，仅将其作为图形生成器。放弃原因是 Codex 的工具生态狭窄，无法复用 DSH Platform 已验证的 AI 网关、安全沙箱、计量系统等成熟模式。
