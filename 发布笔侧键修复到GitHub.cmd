@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\publish-pen-reference.ps1"
if errorlevel 1 (
  echo Publication failed. Keep the error above for diagnosis.
) else (
  echo PhotoCraft reference pushed and verified successfully.
)
pause