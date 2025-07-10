@echo off
set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%"

echo Installing VPN Manager...

REM Check if binary exists
if not exist "release\vpn-manager.exe" (
    echo VPN Manager not found. Building first...
    call build.bat
    if %errorlevel% neq 0 (
        echo Build failed!
        pause
        exit /b 1
    )
)

REM Create Program Files directory
set "INSTALL_DIR=%PROGRAMFILES%\VPN Manager"
if not exist "%INSTALL_DIR%" (
    mkdir "%INSTALL_DIR%"
)

REM Copy executable
copy "release\vpn-manager.exe" "%INSTALL_DIR%\" >nul
if %errorlevel% neq 0 (
    echo Failed to copy to Program Files. Trying user directory...
    set "INSTALL_DIR=%USERPROFILE%\VPN Manager"
    if not exist "%INSTALL_DIR%" (
        mkdir "%INSTALL_DIR%"
    )
    copy "release\vpn-manager.exe" "%INSTALL_DIR%\" >nul
    if %errorlevel% neq 0 (
        echo Installation failed!
        pause
        exit /b 1
    )
)

echo Installation completed successfully!
echo VPN Manager installed to: %INSTALL_DIR%
echo.
echo To run: Double-click "%INSTALL_DIR%\vpn-manager.exe"
echo Or run: "%INSTALL_DIR%\vpn-manager.exe"
echo.
echo Creating desktop shortcut...

REM Create desktop shortcut
set "DESKTOP=%USERPROFILE%\Desktop"
echo Set oWS = WScript.CreateObject("WScript.Shell") > "%TEMP%\shortcut.vbs"
echo sLinkFile = "%DESKTOP%\VPN Manager.lnk" >> "%TEMP%\shortcut.vbs"
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> "%TEMP%\shortcut.vbs"
echo oLink.TargetPath = "%INSTALL_DIR%\vpn-manager.exe" >> "%TEMP%\shortcut.vbs"
echo oLink.WorkingDirectory = "%INSTALL_DIR%" >> "%TEMP%\shortcut.vbs"
echo oLink.Description = "VPN Manager - Network Management Tool" >> "%TEMP%\shortcut.vbs"
echo oLink.Save >> "%TEMP%\shortcut.vbs"
cscript "%TEMP%\shortcut.vbs" >nul
del "%TEMP%\shortcut.vbs"

echo Desktop shortcut created: "VPN Manager.lnk"
echo.
echo Installation complete! You can now run VPN Manager from:
echo - Desktop shortcut
echo - Start menu (search for "VPN Manager")
echo - Direct path: "%INSTALL_DIR%\vpn-manager.exe"

pause