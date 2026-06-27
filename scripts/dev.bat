@echo off
REM ============================================================
REM  Exsul - development mode (Tauri + Vite, hot reload).
REM  ASCII only (no chcp / no Cyrillic) for robust parsing.
REM ============================================================
cd /d "%~dp0.."

if not exist node_modules (
  echo [exsul] installing npm dependencies ...
  call npm install
  if errorlevel 1 (
    echo [exsul] npm install failed.
    pause
    exit /b 1
  )
)

echo [exsul] starting dev (Tauri + Vite) ...
call npm run tauri dev
if errorlevel 1 pause
exit /b %errorlevel%
