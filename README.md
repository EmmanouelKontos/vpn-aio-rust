# VPN Manager AIO

A modern, user-friendly VPN manager with integrated remote access capabilities for Linux systems.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-red.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

## Features

### 🔒 VPN Management
- **OpenVPN Support**: Full OpenVPN client integration
- **WireGuard Support**: Modern WireGuard VPN protocol
- **File Browser**: Easy VPN configuration file selection
- **Connection Status**: Real-time connection monitoring with animations
- **Auto-detection**: Automatic VPN client detection and installation

### 🖥️ Remote Access
- **RDP Connections**: Remote Desktop Protocol support
- **Wake-on-LAN**: Network device wake-up capability
- **Device Monitoring**: Real-time device status tracking
- **Unified Interface**: Combined remote access management

### 🎨 Modern UI
- **Dark Mode**: Beautiful dark theme optimized for extended use
- **Glassy Design**: Modern, professional interface with transparency effects
- **Smooth Animations**: Responsive UI with loading indicators and transitions
- **Intuitive Navigation**: Clean, beginner-friendly interface
- **Loading States**: Visual feedback for all operations

### 🔧 System Integration
- **Cross-Distribution**: Support for major Linux distributions (Ubuntu, Debian, Arch, Fedora, openSUSE)
- **Dependency Management**: Automatic detection and installation of required packages
- **Auto-Updates**: GitHub-based automatic update system
- **Logging**: Comprehensive logging and crash recovery

## Installation

### Prerequisites
- Linux operating system
- Rust 1.70+ (for building from source)
- Display server (X11 or Wayland)

### From Binary (Recommended)
```bash
# Download the latest release
wget https://github.com/emmanouil/vpn-aio/releases/latest/download/vpn-manager
chmod +x vpn-manager
./vpn-manager
```

### From Source
```bash
# Clone the repository
git clone https://github.com/emmanouil/vpn-aio.git
cd vpn-aio

# Build and run
cargo run --release
```

## Usage

### Setting Up VPN Connections
1. Navigate to the **VPN** tab
2. Click **Add VPN Connection**
3. Select VPN type (OpenVPN or WireGuard)
4. Use the **Browse** button to select your configuration file
5. Enter credentials (for OpenVPN)
6. Click **Add Connection**

### Managing Remote Connections
1. Go to the **Remote** tab
2. Add RDP connections with host details
3. Add WOL devices with MAC addresses
4. Use the **Home** tab for quick access to all devices

### System Dependencies
The application automatically detects and offers to install required dependencies:
- **OpenVPN**: `openvpn` package
- **WireGuard**: `wireguard-tools` package
- **RDP Client**: `freerdp` or `remmina` package
- **Network Tools**: `iputils-ping` package

## Configuration

Configuration files are stored in:
- **Linux**: `~/.config/vpn-manager/config.json`

### Configuration Structure
```json
{
  "dark_mode": true,
  "vpn_configs": [...],
  "rdp_configs": [...],
  "wol_devices": [...]
}
```

## Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── config/              # Configuration management
├── network/             # Network operations
│   ├── vpn.rs          # OpenVPN integration
│   ├── wireguard.rs    # WireGuard integration
│   ├── rdp.rs          # RDP client integration
│   ├── wol.rs          # Wake-on-LAN implementation
│   └── monitor.rs      # Network monitoring
├── system/              # System integration
│   ├── installer.rs    # Package installation
│   └── updater.rs      # Automatic updates
└── ui/                  # User interface
    ├── components.rs    # UI components
    ├── theme.rs        # Theme and styling
    └── panels/         # Application panels
```

### Building for Release
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Contributing
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Security

This application handles sensitive network configurations. Security considerations:

- Configuration files are stored with restricted permissions
- No credentials are logged or transmitted
- All network operations use secure protocols
- VPN configurations are handled by system VPN clients

## Platform Support

### Supported Distributions
- **Ubuntu/Debian**: APT package manager
- **Arch Linux**: Pacman package manager
- **Fedora**: DNF package manager
- **CentOS/RHEL**: YUM package manager
- **openSUSE**: Zypper package manager

### Required Packages
| Feature | Package | Auto-Install |
|---------|---------|-------------|
| OpenVPN | `openvpn` | ✅ |
| WireGuard | `wireguard-tools` | ✅ |
| RDP Client | `freerdp` or `remmina` | ✅ |
| Network Tools | `iputils-ping` | ✅ |

## Troubleshooting

### Common Issues

**Application won't start**
- Ensure you have a display server running (X11 or Wayland)
- Check that all dependencies are installed
- Run with `RUST_LOG=info` for detailed logging

**VPN connection fails**
- Verify your configuration file is valid
- Check that the VPN client is installed
- Ensure proper network permissions

**RDP connection issues**
- Verify the target host is reachable
- Check that RDP is enabled on the target system
- Confirm credentials are correct

**Wake-on-LAN not working**
- Ensure WOL is enabled in BIOS/UEFI
- Check network interface supports WOL
- Verify MAC address is correct

### Logging
Enable detailed logging:
```bash
RUST_LOG=debug ./vpn-manager
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [egui](https://github.com/emilk/egui)
- VPN integration via OpenVPN and WireGuard
- Remote access via FreeRDP and Remmina
- Icons and design inspiration from modern UI frameworks

## Support

- **Issues**: [GitHub Issues](https://github.com/emmanouil/vpn-aio/issues)
- **Documentation**: [Wiki](https://github.com/emmanouil/vpn-aio/wiki)
- **Discussions**: [GitHub Discussions](https://github.com/emmanouil/vpn-aio/discussions)

---

**Made with ❤️ for the Linux community**