@echo off
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

REM Check if the binary exists
if not exist "release\vpn-manager.exe" (
    echo VPN Manager not found. Building...
    call build.bat
    if %errorlevel% neq 0 (
        echo Build failed!
        pause
        exit /b 1
    )
)

REM Run the application
echo Starting VPN Manager...
start "" "release\vpn-manager.exe"