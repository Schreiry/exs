@echo off
REM ============================================================
REM  Exsul — режим разработки (Tauri + Vite, hot reload)
REM ============================================================
cd /d "%~dp0.."

if not exist node_modules (
  echo [exsul] installing npm dependencies...
  call npm install || exit /b 1
)

echo [exsul] starting dev (Tauri + Vite)...
call npm run tauri dev
