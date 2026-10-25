@echo off
rem Unified check: typecheck + frontend tests + cargo test + clippy.
rem Env facts: docs/engineering-contract.md section 11. Commands mirror README section 9.
rem ASCII-only on purpose: cmd.exe parses .cmd in the OEM codepage, and UTF-8 Chinese
rem bytes split the line (reproduced on this machine 2026-10-25). Chinese notes live
rem in docs/runbook-dev.md.
setlocal

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=E:\data\.cache\paint-dsh\cargo-target"

pushd "%~dp0.."

call npm run typecheck
if errorlevel 1 goto fail
call npm run test
if errorlevel 1 goto fail

pushd src-tauri
cargo test
if errorlevel 1 goto fail_deps
cargo clippy --all-targets -- -D warnings
if errorlevel 1 goto fail_deps
popd

popd
echo.
echo VERIFY OK
exit /b 0

:fail_deps
popd
:fail
popd
echo.
echo VERIFY FAILED
exit /b 1
