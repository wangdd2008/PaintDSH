# Domain Docs

工程技能在探索代码库时应如何消费本仓库的领域文档。

## 探索前阅读

- 仓库根目录下的 **`GLOSSARY.md`**，或
- 如存在，仓库根目录下的 **`GLOSSARY-MAP.md`**：它指向每个上下文的 `GLOSSARY.md`。阅读与主题相关的每个文件。
- **`docs/adr/`**：阅读与您即将工作的区域相关的 ADR。

如果这些文件不存在，**静默继续**。不要标记其缺失；不要建议提前创建它们。`/domain-modeling` 技能（通过 `/grill-with-docs` 和 `/improve-codebase-architecture` 调用）会在术语或决策实际确定时惰性创建它们。

## 文件结构

单仓库（大多数仓库）：

```
/
├── GLOSSARY.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

多仓库（根目录存在 `GLOSSARY-MAP.md`）：

```
/
├── GLOSSARY-MAP.md
├── docs/adr/                          ← 系统级决策
└── src/
    ├── ordering/
    │   ├── GLOSSARY.md
    │   └── docs/adr/                  ← 上下文特定决策
    └── billing/
        ├── GLOSSARY.md
        └── docs/adr/
```

## 使用术语表词汇

当您的输出命名领域概念时（issue 标题、重构提案、假设、测试名称），使用 `GLOSSARY.md` 中定义的术语。不要偏离术语表明确避免的同义词。

如果您需要的概念尚未在术语表中，这是一个信号：要么您在发明项目不使用的语言（请重新考虑），要么存在真正的空白（记录给 `/domain-modeling`）。

## 标记 ADR 冲突

如果您的输出与现有 ADR 矛盾，请显式指出，而不是静默覆盖：

> _与 ADR-0007（event-sourced orders）矛盾，但值得重新讨论，因为…_
