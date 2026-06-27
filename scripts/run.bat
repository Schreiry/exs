@echo off
REM Delegates to the canonical launcher in the project root (run.bat).
call "%~dp0..\run.bat"
exit /b %errorlevel%
