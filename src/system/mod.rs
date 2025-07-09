use anyhow::Result;
use std::process::Command;
use which::which;

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
    Apt,      // Ubuntu/Debian
    Pacman,   // Arch Linux
    Dnf,      // Fedora
    Yum,      // CentOS/RHEL
    Zypper,   // openSUSE
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
    
    pub fn get_missing_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|dep| !dep.is_installed).collect()
    }
    
    pub fn get_required_missing_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|dep| dep.required && !dep.is_installed).collect()
    }
}

fn detect_distribution() -> Result<String> {
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

fn detect_package_manager() -> PackageManager {
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

fn check_dependencies(package_manager: &PackageManager) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    
    // VPN dependencies
    dependencies.push(check_dependency("OpenVPN", "openvpn", get_package_name("openvpn", package_manager), true)?);
    dependencies.push(check_dependency("WireGuard", "wg", get_package_name("wireguard-tools", package_manager), true)?);
    
    // RDP dependencies
    dependencies.push(check_dependency("FreeRDP", "xfreerdp", get_package_name("freerdp", package_manager), false)?);
    dependencies.push(check_dependency("Remmina", "remmina", get_package_name("remmina", package_manager), false)?);
    
    // Network tools
    dependencies.push(check_dependency("Ping", "ping", get_package_name("iputils-ping", package_manager), true)?);
    dependencies.push(check_dependency("Sudo", "sudo", get_package_name("sudo", package_manager), true)?);
    
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

fn get_package_name(default: &str, package_manager: &PackageManager) -> String {
    match (default, package_manager) {
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
        "wg" => {
            if let Ok(output) = Command::new("wg").arg("--version").output() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                return Some(version_str.trim().to_string());
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
        _ => {}
    }
    None
}