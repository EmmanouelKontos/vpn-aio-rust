@echo off
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

echo Building VPN Manager...

REM Check if Rust is installed - try multiple common locations
set "CARGO_CMD="
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "CARGO_CMD=%USERPROFILE%\.cargo\bin\cargo.exe"
) else (
    where cargo >nul 2>&1
    if %errorlevel% equ 0 (
        set "CARGO_CMD=cargo"
    )
)

if "%CARGO_CMD%"=="" (
    echo Error: Rust is not installed or not found in PATH.
    echo Please install Rust from https://rustup.rs/
    echo Or make sure cargo.exe is in your PATH
    pause
    exit /b 1
)

echo Found Rust at: %CARGO_CMD%

REM Build in release mode
echo Building in release mode...
"%CARGO_CMD%" build --release
if %errorlevel% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

REM Create release directory
if not exist "release" mkdir release

REM Copy binary
copy "target\release\vpn-manager.exe" "release\" >nul
if %errorlevel% neq 0 (
    echo Failed to copy binary!
    pause
    exit /b 1
)

echo Build completed successfully!
echo Binary location: release\vpn-manager.exe
echo.
echo To run: .\release\vpn-manager.exe
echo To install: Copy vpn-manager.exe to a directory in your PATH

pause