@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-runtime.ps1"
if errorlevel 1 (
  echo Dependency installation did not complete.
  pause
  exit /b 1
)
start "" "%~dp0PCL.Proto.exe"
