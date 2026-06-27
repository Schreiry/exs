@echo off
setlocal
REM ============================================================
REM  EXSUL - strict verify + release build (fail-fast)
REM  Stages: npm check -> fmt -> cargo check -> clippy(-D warnings)
REM          -> cargo test -> tauri build (release).
REM  Any failing stage stops with exit code 1 and names the stage.
REM  Run:        double-click, or  build.bat
REM  No pause:   set EXSUL_NOPAUSE=1  (for CI / non-interactive)
REM  Output:     src-tauri\target\release\bundle\
REM  NOTE: ASCII only on purpose - no chcp / no Cyrillic, so the
REM        file always parses regardless of console code page.
REM ============================================================
cd /d "%~dp0"
set "STAGE=init"

echo.
echo ============================================================
echo   EXSUL  ::  strict build pipeline
echo ============================================================

REM ---- 0. prerequisites ----
where cargo >nul 2>&1
if errorlevel 1 (
  set "STAGE=cargo not found in PATH - install Rust from https://rustup.rs"
  goto :fail
)
where npm >nul 2>&1
if errorlevel 1 (
  set "STAGE=npm not found in PATH - install Node.js from https://nodejs.org"
  goto :fail
)
cargo fmt --version >nul 2>&1
if errorlevel 1 (
  echo [setup] installing rustfmt component ...
  rustup component add rustfmt
)
cargo clippy --version >nul 2>&1
if errorlevel 1 (
  echo [setup] installing clippy component ...
  rustup component add clippy
)

REM ---- 1. npm dependencies ----
set "STAGE=npm install"
if not exist node_modules (
  echo [1/7] npm install ...
  call npm install
  if errorlevel 1 goto :fail
) else (
  echo [1/7] npm dependencies present - skip
)

REM ---- 2. frontend type-check (svelte-check + tsc strict) ----
set "STAGE=npm run check (svelte-check + tsc)"
echo [2/7] frontend type-check ...
call npm run check
if errorlevel 1 goto :fail

REM ---- 3. rust formatting ----
set "STAGE=cargo fmt --check"
echo [3/7] rust format check ...
pushd src-tauri
cargo fmt --all -- --check
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

REM ---- 4. rust type-check ----
set "STAGE=cargo check"
echo [4/7] cargo check ...
pushd src-tauri
cargo check --all-targets
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

REM ---- 5. clippy (warnings = errors) ----
set "STAGE=cargo clippy -D warnings"
echo [5/7] clippy (strict) ...
pushd src-tauri
cargo clippy --all-targets -- -D warnings
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

REM ---- 6. unit tests ----
set "STAGE=cargo test"
echo [6/7] cargo test ...
pushd src-tauri
cargo test
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

REM ---- 7. release build (frontend build runs via beforeBuildCommand) ----
set "STAGE=npm run tauri build"
echo [7/7] tauri build (release) ...
call npm run tauri build
if errorlevel 1 goto :fail

echo.
echo ============================================================
echo   OK  ::  ALL GREEN - release built
echo   output: src-tauri\target\release\bundle\
echo ============================================================
if not "%EXSUL_NOPAUSE%"=="1" pause
exit /b 0

:fail
echo.
echo ============================================================
echo   FAIL  ::  stage "%STAGE%" failed - see the errors above.
echo   The release was NOT built. Nothing got through.
echo ============================================================
if not "%EXSUL_NOPAUSE%"=="1" pause
exit /b 1
