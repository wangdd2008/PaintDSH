# DSH Platform（DevStation）

M1 智能研发桌面工作台 —— "桌面即应用"。完整设计见 [DESIGN.md](./DESIGN.md)。

![Dashboard](docs/manual/assets/01-dashboard.png)

## 目录结构

```
├── src/                  # 前端（React 19 + Vite + TypeScript）
│   ├── shell/            # 应用壳：标题栏 / 导航栏 / 右侧面板 / 状态栏
│   ├── modules/          # 功能模块：Dashboard / 建模 / 数字员工 / 服务 / 终端 / 编辑器 / 工具箱 / API 调试 …
│   ├── components/       # 通用组件：⌘K 命令面板 / 监控 / Git / Docker / 进程 / 技能管理 …
│   ├── ai/               # AI 对话面板与 Harness 视图
│   ├── stores/           # zustand 全局状态
│   └── lib/              # 桥接层 / DDL 生成 / 图布局 / Markdown 渲染
├── src-tauri/            # Rust 后端（Tauri 2）
│   └── src/              # AI 网关 / 配置 / SQLite 存储 / Git / 终端 PTY / 系统采样 / hprof 分析 …
├── docs/                 # 文档与截图
├── design/               # 设计稿原型（HTML/JSX）
└── DESIGN.md             # 完整设计文档
```

## 当前状态

- ✅ 设计稿 AppShell 1:1（标题栏 ⌘K / 60px 导航 / 右侧 AI·监控双面板 / 底部状态栏）
- ✅ ⌘K 命令面板（导航 / AI 指令 / 操作 / Git 分组，模糊搜索）
- ✅ 低功耗调度（活跃 1s → 空闲 5s → 后台 30s，省电模式再放慢一倍）
- ✅ 系统采样（CPU P/E 核、内存、磁盘、网络、电池）
- ✅ AI 双引擎：**DeepSeek Harness**（默认，headless 子进程，自带工具与会话管理）+ 直连 API（SSE 逐字流式）
- ✅ token 用量统计与月度预算
- ✅ 右栏 AI 对话 + Dashboard 任务 + 设置页
- ✅ **建模中心**：数据建模（概念/逻辑/物理三层图 + 生成 DDL）· 业务建模（完整业务流程 + 微服务/产品接口拓扑）· 全部支持 AI 生成与手动编辑
- ✅ **监控明细下钻**：指标卡下钻 · 进程树 · 端口查进程 · 启动链路追踪
- ✅ **数字员工**：内置五岗 + 自定义（角色/技能/知识库），产出可转建模图
- ✅ **工具箱**：JSON 格式化与字段统计 · Token 用量统计（天/周/月趋势 · 场景/员工分类 · 任务 Top）· Beyond Compare 式文件比对（行级+词级高亮）· hprof 堆转储分析（实测 811MB/12s）
- ✅ **微服务运行**：Maven/JDK 配置、多项目并行启动、实时日志、整组进程管理
- ✅ **知识库**：Markdown 笔记编辑/预览/全文搜索
- ✅ **终端**：本地多标签终端（portable-pty + xterm，随项目目录启动）
- ✅ **代码编辑器**：项目文件树 + CodeMirror 多标签编辑（多语言高亮 · ⌘S 保存 · 新建/删除）
- ✅ **Git 可视化**：状态/暂存/差异/提交历史 + 提交/拉取/推送（系统 git 凭证）
- ✅ **API 调试**：多方法 HTTP 客户端 · 请求头/请求体 · JSON 美化响应 · 历史记录
- ✅ **可展开侧栏**：图标/图文两种形态，收展状态记忆
- ✅ **自定义数字员工**：角色设定 + 技能 + 挂接知识库笔记
- ⏳ 代码编辑器 / Git / API 调试 → 后续版本

## 开发

```bash
export DEEPSEEK_API_KEY=sk-…   # AI 功能必需的 API Key（环境变量读取，不落盘）
npm install
npm run tauri dev    # 开发
npm run tauri build  # 打包
cargo test                     # Rust 单测
DSH_TEST_API_KEY=sk-… cargo test -- --ignored harness_e2e  # 真实链路验证
```

数据目录：`~/.dsh-platform/`（config.toml + platform.db + harness/）。

harness 目录默认自动探测同工作区的 `deepseek-desktop`，也可在设置页手动指定。
