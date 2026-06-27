@echo off
REM Delegates to the canonical strict pipeline in the project root (build.bat):
REM checks (npm check + fmt + cargo check + clippy + test) -> release build.
call "%~dp0..\build.bat"
exit /b %errorlevel%
