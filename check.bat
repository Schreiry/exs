@echo off
setlocal
REM ============================================================
REM  EXSUL - fast verification (NO release bundle)
REM  Same strict checks as build.bat, but without building the
REM  installer - for frequent runs during development.
REM  Stages: npm check -> fmt -> cargo check -> clippy -> test.
REM  No pause: set EXSUL_NOPAUSE=1
REM  ASCII only on purpose (no chcp / no Cyrillic).
REM ============================================================
cd /d "%~dp0"
set "STAGE=init"

echo.
echo ============================================================
echo   EXSUL  ::  verify (no release)
echo ============================================================

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
if errorlevel 1 rustup component add rustfmt
cargo clippy --version >nul 2>&1
if errorlevel 1 rustup component add clippy

set "STAGE=npm install"
if not exist node_modules (
  echo [1/5] npm install ...
  call npm install
  if errorlevel 1 goto :fail
) else (
  echo [1/5] npm dependencies present - skip
)

set "STAGE=npm run check (svelte-check + tsc)"
echo [2/5] frontend type-check ...
call npm run check
if errorlevel 1 goto :fail

set "STAGE=cargo fmt --check"
echo [3/5] rust format check ...
pushd src-tauri
cargo fmt --all -- --check
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

set "STAGE=cargo check + clippy -D warnings"
echo [4/5] cargo check + clippy (strict) ...
pushd src-tauri
cargo clippy --all-targets -- -D warnings
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

set "STAGE=cargo test"
echo [5/5] cargo test ...
pushd src-tauri
cargo test
set "ERR=%errorlevel%"
popd
if not "%ERR%"=="0" goto :fail

echo.
echo ============================================================
echo   OK  ::  ALL CHECKS PASSED
echo ============================================================
if not "%EXSUL_NOPAUSE%"=="1" pause
exit /b 0

:fail
echo.
echo ============================================================
echo   FAIL  ::  stage "%STAGE%" failed - see the errors above.
echo ============================================================
if not "%EXSUL_NOPAUSE%"=="1" pause
exit /b 1
