@echo off
REM ============================================================
REM  Exsul — production build (NSIS installer + exe)
REM  Output: src-tauri\target\release\bundle\
REM ============================================================
cd /d "%~dp0.."

if not exist node_modules (
  echo [exsul] installing npm dependencies...
  call npm install || exit /b 1
)

echo [exsul] building release...
call npm run tauri build || exit /b 1

echo [exsul] done. See src-tauri\target\release\bundle\
