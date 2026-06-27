@echo off
REM ============================================================
REM  Exsul — запуск собранного бинарника (соберёт, если нет)
REM ============================================================
cd /d "%~dp0.."

set "EXE=src-tauri\target\release\exsul.exe"
if not exist "%EXE%" (
  echo [exsul] release binary not found, building...
  call npm run tauri build || exit /b 1
)

echo [exsul] launching %EXE%
start "" "%EXE%"
