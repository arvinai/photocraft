@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-install-pen-fix.ps1"
if errorlevel 1 (
  echo Build or installation failed. Keep the error above for diagnosis.
) else (
  echo PhotoCraft pen button fix installed successfully.
)
pause
