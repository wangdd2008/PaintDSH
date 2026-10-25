@echo off
rem One-command dev environment: Vite (1420) + Tauri window. Log: %TEMP%\paint-dev.log
rem   scripts\dev.cmd        runs npm run tauri dev (compiles Rust)
rem   scripts\dev.cmd --exe  reuses the prebuilt debug exe, skips compiling
rem                          (workaround when the link fails, see docs/runbook-dev.md)
rem ASCII-only on purpose: cmd.exe parses .cmd in the OEM codepage, and UTF-8 Chinese
rem bytes split the line (reproduced on this machine 2026-10-25). Chinese notes live
rem in docs/runbook-dev.md.
setlocal

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=E:\data\.cache\paint-dsh\cargo-target"
set "LOG=%TEMP%\paint-dev.log"
set "ROOT=%~dp0.."
set "MODE=%~1"

chcp 65001 >nul
echo [dev] %DATE% %TIME% mode="%MODE%" > "%LOG%"
pushd "%ROOT%"

rem --- Vite: reuse if 1420 is already listening, else start one ---
netstat -ano | findstr /c:"127.0.0.1:1420" >nul
if not errorlevel 1 (
  echo [dev] vite already on 1420, reuse
  goto vite_ready
)

echo [dev] starting vite
start "paint-vite" /min cmd /c "npm run dev >> ""%LOG%"" 2>&1"

set /a WAITED=0
:wait_vite
netstat -ano | findstr /c:"127.0.0.1:1420" >nul
if not errorlevel 1 goto vite_ready
if %WAITED% geq 30 goto vite_timeout
timeout /t 1 /nobreak >nul
set /a WAITED+=1
goto wait_vite

:vite_timeout
echo [dev] FAILED: vite not listening on 1420 after 30s, see "%LOG%"
popd
exit /b 1

:vite_ready
rem --- App: compile and launch, or run the existing exe ---
if /i "%MODE%"=="--exe" (
  if not exist "%CARGO_TARGET_DIR%\debug\paint-dsh.exe" (
    echo [dev] FAILED: %CARGO_TARGET_DIR%\debug\paint-dsh.exe not found
    popd
    exit /b 1
  )
  echo [dev] launching existing exe, no recompile
  start "paint-app" /d "%ROOT%\src-tauri" "%CARGO_TARGET_DIR%\debug\paint-dsh.exe"
) else (
  echo [dev] npm run tauri dev (log: %LOG%)
  start "paint-tauri" /min cmd /c "npm run tauri dev >> ""%LOG%"" 2>&1"
)

rem --- Wait for the window (compile mode needs the Rust build, up to 240s) ---
set /a WAITED=0
:wait_window
tasklist /fi "imagename eq paint-dsh.exe" | findstr /i "paint-dsh.exe" >nul
if not errorlevel 1 goto window_up
if %WAITED% geq 240 goto window_timeout
timeout /t 2 /nobreak >nul
set /a WAITED+=2
goto wait_window

:window_up
echo [dev] WINDOW UP, log: %LOG%
popd
exit /b 0

:window_timeout
echo [dev] FAILED: no paint-dsh.exe after 240s, see "%LOG%"
echo [dev] hint: if the log shows LNK1318, retry with: scripts\dev.cmd --exe
popd
exit /b 1
