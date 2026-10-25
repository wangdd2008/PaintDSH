# 开发启动与排障（runbook）

本机环境事实（`cargo` 不在 PATH、共享 `CARGO_TARGET_DIR`、crates 镜像）见 `docs/engineering-contract.md` §11；
命令与环境变量速查见 `README.md` §9、§5。本文只写「怎么把窗口跑起来」和「跑不起来怎么办」。

一条命令的入口是 `scripts\dev.cmd`（见下）；失败时按场景查对应小节。

## 1. 正常启动

```cmd
cd /d E:\data\src\PaintDSH
npm run tauri dev
```

判断窗口真的起来了，看进程而不是看日志（日志会停在 `Building [===> ] 428/430` 很久）：

```cmd
tasklist /fi "imagename eq paint-dsh.exe"
powershell -NoProfile -Command "Get-Process paint-dsh | Select-Object Id,MainWindowTitle,Responding"
```

`MainWindowTitle` 为 `PAINT`、`Responding` 为 `True` 即正常。`paint-dsh.exe` 约 20 MB、常驻内存 30 MB 上下。

**已知噪声**（不用管）：启动时会打印

```
Error Found version mismatched Tauri packages ... tauri (v2.11.6) : @tauri-apps/api (v2.12.4)
```

这是钉版（契约 §10.1）带来的必然结果，只影响提示，不影响运行。

## 2. 链接失败（LNK1318）：用已有 exe 启动

症状：Rust 编译到 428/430 后失败，`link.exe` 退出码 1318：

```
libtauri-<hash>.rlib(tauri-<hash>.tauri.<hash>-cgu.04.rcgu.o) : fatal error LNK1318: 非预期的 PDB 错误: OK (0)
error: could not compile `paint-dsh` (lib) due to 1 previous error
```

失败点是 `paint_dsh_lib.dll`（crate-type 里的 cdylib）的 PDB 写入，**稳定复现**：删掉
`debug\deps\paint_dsh_lib.{pdb,dll,dll.lib,dll.exp}` 再跑，仍停在同一个 CGU。磁盘空间、并发链接、残留
`mspdbsrv` 都已排除（2026-10-25 实测）。

失败**只影响 cdylib**：`cargo test` 与 `cargo clippy --all-targets` 照常通过（`npm run verify` 全绿），
所以校验/提交不受这个链接问题影响。

绕行：debug 构建会按 `tauri.conf.json` 的 `build.devUrl` 加载 `http://localhost:1420`，
所以不需要 `tauri dev` 的编排——只要 Vite 在 1420，直接跑共享 target 里的 exe 即可。

```cmd
:: 终端 1：起 Vite
cd /d E:\data\src\PaintDSH && npm run dev

:: 终端 2：起窗口
cd /d E:\data\src\PaintDSH\src-tauri
"E:\data\.cache\paint-dsh\cargo-target\debug\paint-dsh.exe"
```

或用脚本一步到位：

```cmd
scripts\dev.cmd --exe
```

代价：**Rust 改动不会生效**（exe 是上一次成功链接的产物），前端改动仍然走 Vite 热更新。
先确认 exe 不比源码旧：

```cmd
dir "E:\data\.cache\paint-dsh\cargo-target\debug\paint-dsh.exe"
```

exe 不存在或过旧时没有绕行，只能先修链接（换 MSVC 工具链版本，或临时降低该 crate 的 debug 信息量）。

## 3. 端口 1420 被占用

`npm run tauri dev` 的 Rust 编译失败后，它启动的 Vite 子进程**会存活并继续占用 1420**
（父进程退出不会带走它）。下一次 `npm run dev` 直接：

```
Error: Port 1420 is already in use
```

处理方式（按 PID，不要按进程名杀 `node`，本机有多个不相关的 node 进程）：

```cmd
netstat -ano | findstr ":1420"     :: 最后一列是 PID
taskkill /pid <PID> /f
```

如果只是要复用已有的 Vite，就不用杀——`scripts\dev.cmd` 会检测到 1420 已监听并直接复用。

## 4. 排查 AI / 后端问题时的信息盲区

应用**不写运行日志**：`stdout` / `stderr` 为空，启动、DSH 拉起、AI 调用失败都没有痕迹。
目前唯一可查的落点是数据库：

```cmd
python -c "import sqlite3;print(sqlite3.connect(r'C:\Users\hzb\.paint\paint.db').execute('select count(*) from ai_calls').fetchone())"
```

`ai_calls` 只记录**成功落库**的调用；401、超时、harness 无输出都不会留痕。
需要现场时要按 ticket 给应用补 `%PAINT_HOME%\logs\app.log`（滚动、复用 `harness_web::redact` 脱敏）。
在那之前，AI 链路失败只能靠隔离变量（`PAINT_HARNESS_SANDBOX`、`DEEPSEEK_API_KEY`）+ 手工重跑定位。

## 5. 脚本约定：`scripts\*.cmd` 保持 ASCII

`cmd.exe` 按 OEM 代码页（本机 936）解析批处理文件，文件里写 UTF-8 中文注释会被拆成命令执行，
实测报 `'ev（会编译' is not recognized`（2026-10-25）。所以 `scripts\verify.cmd`、`scripts\dev.cmd`
的注释与输出都是英文，中文说明只写在这里和 README。改脚本时不要"顺手"把注释翻译回去。
