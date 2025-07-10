use anyhow::Result;
use std::process::Command;
use which::which;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub mod installer;
pub mod updater;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub distribution: String,
    pub package_manager: PackageManager,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub enum PackageManager {
    Apt,        // Ubuntu/Debian
    Pacman,     // Arch Linux
    Dnf,        // Fedora
    Yum,        // CentOS/RHEL
    Zypper,     // openSUSE
    Chocolatey, // Windows
    Scoop,      // Windows
    Winget,     // Windows
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub binary_name: String,
    pub package_name: String,
    pub is_installed: bool,
    pub version: Option<String>,
    pub required: bool,
}

impl SystemInfo {
    pub fn detect() -> Result<Self> {
        let distribution = detect_distribution()?;
        let package_manager = detect_package_manager();
        let dependencies = check_dependencies(&package_manager)?;
        
        Ok(Self {
            distribution,
            package_manager,
            dependencies,
        })
    }
    
    pub fn refresh_dependencies(&mut self) -> Result<()> {
        self.dependencies = check_dependencies(&self.package_manager)?;
        Ok(())
    }
    
    pub fn get_missing_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|dep| !dep.is_installed).collect()
    }
    
    pub fn get_required_missing_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|dep| dep.required && !dep.is_installed).collect()
    }
}

fn detect_distribution() -> Result<String> {
    #[cfg(windows)]
    {
        return detect_windows_version();
    }
    
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("lsb_release").arg("-d").output() {
            if output.status.success() {
                let description = String::from_utf8_lossy(&output.stdout);
                if let Some(dist) = description.strip_prefix("Description:") {
                    return Ok(dist.trim().to_string());
                }
            }
        }
        
        // Fallback to /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let name = line.strip_prefix("PRETTY_NAME=").unwrap();
                    return Ok(name.trim_matches('"').to_string());
                }
            }
        }
        
        Ok("Unknown Linux".to_string())
    }
}

#[cfg(windows)]
fn detect_windows_version() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let cur_ver = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;
    
    let product_name: String = cur_ver.get_value("ProductName").unwrap_or_else(|_| "Windows".to_string());
    let display_version: String = cur_ver.get_value("DisplayVersion").unwrap_or_else(|_| "".to_string());
    
    if display_version.is_empty() {
        Ok(product_name)
    } else {
        Ok(format!("{} {}", product_name, display_version))
    }
}

fn detect_package_manager() -> PackageManager {
    #[cfg(windows)]
    {
        if which("winget").is_ok() {
            PackageManager::Winget
        } else if which("choco").is_ok() {
            PackageManager::Chocolatey
        } else if which("scoop").is_ok() {
            PackageManager::Scoop
        } else {
            PackageManager::Unknown
        }
    }
    
    #[cfg(unix)]
    {
        if which("apt").is_ok() {
            PackageManager::Apt
        } else if which("pacman").is_ok() {
            PackageManager::Pacman
        } else if which("dnf").is_ok() {
            PackageManager::Dnf
        } else if which("yum").is_ok() {
            PackageManager::Yum
        } else if which("zypper").is_ok() {
            PackageManager::Zypper
        } else {
            PackageManager::Unknown
        }
    }
}

fn check_dependencies(package_manager: &PackageManager) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    
    #[cfg(windows)]
    {
        // VPN dependencies for Windows
        dependencies.push(check_dependency_windows("OpenVPN", "openvpn", get_package_name("openvpn", package_manager), true)?);
        dependencies.push(check_dependency_windows("WireGuard", "wireguard", get_package_name("wireguard", package_manager), true)?);
        
        // RDP dependencies (built into Windows)
        dependencies.push(check_dependency_windows("Remote Desktop", "mstsc", "builtin".to_string(), false)?);
        
        // Network tools (built into Windows)
        dependencies.push(check_dependency_windows("Ping", "ping", "builtin".to_string(), true)?);
    }
    
    #[cfg(unix)]
    {
        // VPN dependencies for Unix-like systems
        dependencies.push(check_dependency("OpenVPN", "openvpn", get_package_name("openvpn", package_manager), true)?);
        dependencies.push(check_dependency("WireGuard", "wg", get_package_name("wireguard-tools", package_manager), true)?);
        
        // RDP dependencies
        dependencies.push(check_dependency("FreeRDP", "xfreerdp", get_package_name("freerdp", package_manager), false)?);
        dependencies.push(check_dependency("Remmina", "remmina", get_package_name("remmina", package_manager), false)?);
        
        // Network tools
        dependencies.push(check_dependency("Ping", "ping", get_package_name("iputils-ping", package_manager), true)?);
        dependencies.push(check_dependency("Sudo", "sudo", get_package_name("sudo", package_manager), true)?);
    }
    
    Ok(dependencies)
}

fn check_dependency(name: &str, binary: &str, package: String, required: bool) -> Result<Dependency> {
    let is_installed = which(binary).is_ok();
    let version = if is_installed {
        get_version(binary)
    } else {
        None
    };
    
    Ok(Dependency {
        name: name.to_string(),
        binary_name: binary.to_string(),
        package_name: package,
        is_installed,
        version,
        required,
    })
}

#[cfg(windows)]
fn check_dependency_windows(name: &str, binary: &str, package: String, required: bool) -> Result<Dependency> {
    let is_installed = match binary {
        "wireguard" => {
            // Check multiple locations for WireGuard on Windows
            let wireguard_paths = vec![
                "C:\\Program Files\\WireGuard\\wireguard.exe",
                "C:\\Program Files (x86)\\WireGuard\\wireguard.exe",
            ];
            
            wireguard_paths.iter().any(|path| std::path::Path::new(path).exists()) || which("wireguard").is_ok()
        },
        "openvpn" => {
            // Check multiple locations for OpenVPN on Windows
            let openvpn_paths = vec![
                "C:\\Program Files\\OpenVPN\\bin\\openvpn.exe",
                "C:\\Program Files (x86)\\OpenVPN\\bin\\openvpn.exe",
            ];
            
            openvpn_paths.iter().any(|path| std::path::Path::new(path).exists()) || which("openvpn").is_ok()
        },
        "mstsc" => {
            // Remote Desktop Connection is built into Windows
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let mstsc_path = format!("{}\\System32\\mstsc.exe", windir);
            std::path::Path::new(&mstsc_path).exists()
        },
        "ping" => {
            // Ping is built into Windows
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            let ping_path = format!("{}\\System32\\ping.exe", windir);
            std::path::Path::new(&ping_path).exists()
        },
        _ => which(binary).is_ok(),
    };
    
    let version = if is_installed {
        get_version_windows(binary)
    } else {
        None
    };
    
    Ok(Dependency {
        name: name.to_string(),
        binary_name: binary.to_string(),
        package_name: package,
        is_installed,
        version,
        required,
    })
}

fn get_package_name(default: &str, package_manager: &PackageManager) -> String {
    match (default, package_manager) {
        // Windows package names
        ("openvpn", PackageManager::Winget) => "OpenVPN.OpenVPN".to_string(),
        ("openvpn", PackageManager::Chocolatey) => "openvpn".to_string(),
        ("openvpn", PackageManager::Scoop) => "openvpn".to_string(),
        ("wireguard", PackageManager::Winget) => "WireGuard.WireGuard".to_string(),
        ("wireguard", PackageManager::Chocolatey) => "wireguard".to_string(),
        ("wireguard", PackageManager::Scoop) => "wireguard".to_string(),
        
        // Linux package names
        ("freerdp", PackageManager::Apt) => "freerdp2-x11".to_string(),
        ("freerdp", PackageManager::Pacman) => "freerdp".to_string(),
        ("freerdp", PackageManager::Dnf) => "freerdp".to_string(),
        ("iputils-ping", PackageManager::Pacman) => "iputils".to_string(),
        ("wireguard-tools", PackageManager::Apt) => "wireguard-tools".to_string(),
        ("wireguard-tools", PackageManager::Pacman) => "wireguard-tools".to_string(),
        ("wireguard-tools", PackageManager::Dnf) => "wireguard-tools".to_string(),
        
        _ => default.to_string(),
    }
}

fn get_version(binary: &str) -> Option<String> {
    match binary {
        "openvpn" => {
            if let Ok(output) = Command::new("openvpn").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = version_str.lines().next() {
                    return Some(line.to_string());
                }
            }
        }
        "wg" | "wireguard" => {
            #[cfg(windows)]
            {
                if let Ok(output) = Command::new("wireguard").arg("--version").output() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    return Some(version_str.trim().to_string());
                }
            }
            #[cfg(unix)]
            {
                if let Ok(output) = Command::new("wg").arg("--version").output() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    return Some(version_str.trim().to_string());
                }
            }
        }
        "xfreerdp" => {
            if let Ok(output) = Command::new("xfreerdp").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = version_str.lines().next() {
                    return Some(line.to_string());
                }
            }
        }
        "remmina" => {
            if let Ok(output) = Command::new("remmina").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = version_str.lines().next() {
                    return Some(line.to_string());
                }
            }
        }
        "mstsc" => {
            #[cfg(windows)]
            {
                return Some("Built-in Windows RDP Client".to_string());
            }
        }
        "ping" => {
            if let Ok(output) = Command::new("ping").arg("/?").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = version_str.lines().next() {
                    return Some(line.to_string());
                }
            }
        }
        _ => {}
    }
    None
}

#[cfg(windows)]
fn get_version_windows(binary: &str) -> Option<String> {
    match binary {
        "wireguard" => {
            let wireguard_paths = vec![
                "C:\\Program Files\\WireGuard\\wireguard.exe",
                "C:\\Program Files (x86)\\WireGuard\\wireguard.exe",
            ];
            
            for path in wireguard_paths {
                if std::path::Path::new(path).exists() {
                    if let Ok(output) = Command::new(path).arg("--version").output() {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        if !version_str.trim().is_empty() {
                            return Some(version_str.trim().to_string());
                        }
                    }
                }
            }
            
            // Fallback to PATH version
            if let Ok(output) = Command::new("wireguard").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                return Some(version_str.trim().to_string());
            }
        },
        "openvpn" => {
            let openvpn_paths = vec![
                "C:\\Program Files\\OpenVPN\\bin\\openvpn.exe",
                "C:\\Program Files (x86)\\OpenVPN\\bin\\openvpn.exe",
            ];
            
            for path in openvpn_paths {
                if std::path::Path::new(path).exists() {
                    if let Ok(output) = Command::new(path).arg("--version").output() {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        if let Some(line) = version_str.lines().next() {
                            return Some(line.to_string());
                        }
                    }
                }
            }
            
            // Fallback to PATH version
            if let Ok(output) = Command::new("openvpn").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = version_str.lines().next() {
                    return Some(line.to_string());
                }
            }
        },
        _ => return get_version(binary),
    }
    
    None
}