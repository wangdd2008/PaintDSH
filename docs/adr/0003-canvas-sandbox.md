# 自定义"画图友好"安全沙箱策略

DSH 默认的 headless 安全策略是 read-only + approval=never，这对画图场景过于严格——AI 需要创建图层、修改画布。我们采用自定义沙箱策略 `canvas-write`：允许画布相关的写入操作（图层增删、像素修改），但内核级拒绝 rm/mkfs/dd 等危险系统命令。策略通过 `--patch` 参数注入 DSH，实现方式与 DSH Platform 的 `HEADLESS_SANDBOX_PATCH` 相同，但配置项不同。

**考虑过的选项**：完全禁用沙箱（风险过高）；保持 read-only 但通过前端代理写入（增加复杂度且不安全）。自定义策略在安全性与功能性之间取得平衡。
