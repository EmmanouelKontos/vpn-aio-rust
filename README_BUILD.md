# VPN Manager - Build & Run Instructions

## Quick Start

### Option 1: Easy Install (Recommended)
1. Double-click `install.bat` to build and install the application
2. This will:
   - Build the application automatically
   - Install it to your system
   - Create a desktop shortcut
   - Add it to your programs menu

### Option 2: Just Run Once
1. Double-click `run.bat` to build and run the application directly
2. This will build if needed and start the application

### Option 3: Manual Build
1. Double-click `build.bat` to build the application
2. The executable will be created in the `release` folder
3. Double-click `release\vpn-manager.exe` to run

## Building on Windows

### Prerequisites
- Rust installed from https://rustup.rs/
- Windows 10 or later

### Build Commands
```cmd
# Build the application
build.bat

# Run the application
run.bat

# Install system-wide
install.bat
```

## Building on Linux/WSL

### Prerequisites
- Rust installed from https://rustup.rs/
- Linux/WSL environment

### Build Commands
```bash
# Build the application
./build.sh

# Run the application
./release/vpn-manager
```

## Features

✅ **WireGuard Support**: Connect, disconnect, and monitor WireGuard VPN connections
✅ **OpenVPN Support**: Connect to OpenVPN servers
✅ **RDP Management**: Manage remote desktop connections
✅ **Wake-on-LAN**: Wake up remote devices
✅ **Auto-connect**: Optional auto-connect to VPN on startup
✅ **No Console Window**: Runs as a proper Windows GUI application
✅ **Real-time Status**: Live connection monitoring
✅ **Modern UI**: Glass-style dark theme interface

## Running the Application

### After Installation
- Use the desktop shortcut "VPN Manager"
- Or search for "VPN Manager" in the Start menu
- Or run directly from the installation folder

### Application Features
- **VPN Panel**: Manage your VPN connections with real-time status
- **Remote Panel**: RDP connections and Wake-on-LAN devices
- **Settings Panel**: Configure auto-connect and appearance
- **Home Panel**: Quick overview of system status

### Settings
- **Auto-connect**: Enable/disable automatic VPN connection on startup
- **Theme**: Dark/Light mode (requires restart)
- **Dependencies**: Check required system tools and get installation commands
  - **Copy Install Command**: Copies the package installation command to clipboard
  - **Open Terminal/PowerShell**: Opens a new terminal window where you can run the command

## Troubleshooting

### "Rust not found" error
- Make sure Rust is installed from https://rustup.rs/
- Restart your command prompt/terminal after installation
- Try running `cargo --version` to verify installation

### Application won't start
- Check if Windows Defender is blocking the executable
- Try running as administrator
- Check the log file in `%APPDATA%\vpn-manager\vpn-manager.log`

### VPN connection issues
- Ensure WireGuard or OpenVPN is installed on your system
- Check that config files are valid
- Run the application as administrator for system-level VPN operations

### Installing dependencies
- The application will show you which dependencies are missing
- Click "Copy Install Command" to copy the installation command
- Click "Open Terminal/PowerShell" to open a terminal
- Paste and run the command as administrator
- Common dependencies:
  - **Windows**: WireGuard, OpenVPN (via Chocolatey, Scoop, or Winget)
  - **Linux**: wireguard-tools, openvpn (via apt, dnf, pacman, etc.)

## File Structure

```
vpn-aio-rust/
├── src/                    # Source code
├── release/               # Built executable
├── build.bat             # Windows build script
├── build.sh              # Linux build script
├── run.bat               # Quick run script
├── install.bat           # Installation script
└── README_BUILD.md       # This file
```

## Notes

- The application runs without a console window in release mode
- Logs are written to `%APPDATA%\vpn-manager\vpn-manager.log` on Windows
- Configuration is stored in `%APPDATA%\vpn-manager\config.json`
- Auto-connect is disabled by default for security