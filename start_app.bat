@echo off
:: Check for administrative privileges
net session >nul 2>&1
if %errorLevel% == 0 (
    goto :run
) else (
    echo Requesting administrative privileges...
    powershell -Command "Start-Process -FilePath '%0' -Verb RunAs"
    exit /b
)

:run
echo Starting Dx3 with Admin Privileges...
cd /d "%~dp0"
call cargo tauri dev
pause
