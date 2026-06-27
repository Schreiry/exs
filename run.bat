@echo off
setlocal
REM ============================================================
REM  EXSUL - just run the app.
REM  - If a release binary exists, launch it directly (no compile).
REM  - Otherwise start dev mode (first run compiles, then launches).
REM  Run: double-click, or  run.bat
REM  ASCII only on purpose (no chcp / no Cyrillic).
REM ============================================================
cd /d "%~dp0"

set "EXE=src-tauri\target\release\exsul.exe"
if exist "%EXE%" (
  echo Launching built Exsul - no compilation needed ...
  start "" "%EXE%"
  exit /b 0
)

echo No release binary found.
echo Starting in dev mode ^(first run compiles Rust, then opens the app^) ...
echo.

where npm >nul 2>&1
if errorlevel 1 (
  echo [FATAL] npm not found in PATH - install Node.js from https://nodejs.org
  pause
  exit /b 1
)
where cargo >nul 2>&1
if errorlevel 1 (
  echo [FATAL] cargo not found in PATH - install Rust from https://rustup.rs
  pause
  exit /b 1
)

if not exist node_modules (
  echo Installing npm dependencies ...
  call npm install
  if errorlevel 1 (
    echo [FATAL] npm install failed.
    pause
    exit /b 1
  )
)

call npm run tauri dev
if errorlevel 1 (
  echo.
  echo Dev session ended with an error - see above.
  pause
  exit /b 1
)
exit /b 0
