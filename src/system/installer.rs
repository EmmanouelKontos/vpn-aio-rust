use super::{PackageManager, SystemInfo};
use anyhow::Result;
use std::process::Command;

pub struct PackageInstaller {
    package_manager: PackageManager,
}

impl PackageInstaller {
    pub fn new(system_info: &SystemInfo) -> Self {
        Self {
            package_manager: system_info.package_manager.clone(),
        }
    }
    
    pub async fn install_packages(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        match self.package_manager {
            PackageManager::Apt => self.install_apt(packages).await,
            PackageManager::Pacman => self.install_pacman(packages).await,
            PackageManager::Dnf => self.install_dnf(packages).await,
            PackageManager::Yum => self.install_yum(packages).await,
            PackageManager::Zypper => self.install_zypper(packages).await,
            PackageManager::Unknown => Err(anyhow::anyhow!("Unknown package manager")),
            PackageManager::Chocolatey => self.install_chocolatey(packages).await,
            PackageManager::Scoop => self.install_scoop(packages).await,
            PackageManager::Winget => self.install_winget(packages).await
        }
    }
    
    pub async fn update_package_cache(&self) -> Result<()> {
        match self.package_manager {
            PackageManager::Apt => {
                let output = Command::new("sudo")
                    .args(&["apt", "update"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update package cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Pacman => {
                let output = Command::new("sudo")
                    .args(&["pacman", "-Sy"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update package cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Dnf => {
                let output = Command::new("sudo")
                    .args(&["dnf", "check-update"])
                    .output()?;
                
                // dnf check-update returns 100 when updates are available, which is normal
                if !output.status.success() && output.status.code() != Some(100) {
                    return Err(anyhow::anyhow!("Failed to update package cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Yum => {
                let output = Command::new("sudo")
                    .args(&["yum", "check-update"])
                    .output()?;
                
                // yum check-update returns 100 when updates are available, which is normal
                if !output.status.success() && output.status.code() != Some(100) {
                    return Err(anyhow::anyhow!("Failed to update package cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Zypper => {
                let output = Command::new("sudo")
                    .args(&["zypper", "refresh"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update package cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Unknown => {
                return Err(anyhow::anyhow!("Unknown package manager"));
            }
            PackageManager::Chocolatey => {
                let output = Command::new("choco")
                    .args(&["upgrade", "all", "-y"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update chocolatey: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Scoop => {
                let output = Command::new("scoop")
                    .arg("update")
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update scoop: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            PackageManager::Winget => {
                let output = Command::new("winget")
                    .args(&["upgrade", "--all"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!("Failed to update winget: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
        }
        
        Ok(())
    }
    
    async fn install_apt(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["apt", "install", "-y"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_pacman(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["pacman", "-S", "--noconfirm"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_dnf(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["dnf", "install", "-y"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_yum(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["yum", "install", "-y"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_zypper(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["zypper", "install", "-y"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("sudo")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_chocolatey(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["install", "-y"];
        for package in packages {
            args.push(package);
        }
        
        let output = Command::new("choco")
            .args(&args)
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to install packages: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }
    
    async fn install_scoop(&self, packages: &[String]) -> Result<()> {
        for package in packages {
            let output = Command::new("scoop")
                .args(&["install", package])
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to install package {}: {}", 
                    package, String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        Ok(())
    }
    
    async fn install_winget(&self, packages: &[String]) -> Result<()> {
        for package in packages {
            let output = Command::new("winget")
                .args(&["install", "--id", package, "--silent", "--accept-source-agreements", "--accept-package-agreements"])
                .output()?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to install package {}: {}", 
                    package, String::from_utf8_lossy(&output.stderr)));
            }
        }
        
        Ok(())
    }
    
    pub fn get_install_command(&self, packages: &[String]) -> String {
        match self.package_manager {
            PackageManager::Apt => format!("sudo apt install -y {}", packages.join(" ")),
            PackageManager::Pacman => format!("sudo pacman -S --noconfirm {}", packages.join(" ")),
            PackageManager::Dnf => format!("sudo dnf install -y {}", packages.join(" ")),
            PackageManager::Yum => format!("sudo yum install -y {}", packages.join(" ")),
            PackageManager::Zypper => format!("sudo zypper install -y {}", packages.join(" ")),
            PackageManager::Chocolatey => format!("choco install -y {}", packages.join(" ")),
            PackageManager::Scoop => format!("scoop install {}", packages.join(" ")),
            PackageManager::Winget => format!("winget install {}", packages.join(" ")),
            PackageManager::Unknown => {
                #[cfg(windows)]
                {
                    // Provide alternative Windows installation methods
                    let mut commands = Vec::new();
                    
                    // Check if any package managers are available
                    if which::which("winget").is_ok() {
                        let winget_packages: Vec<String> = packages.iter().map(|p| {
                            match p.as_str() {
                                "openvpn" => "OpenVPN.OpenVPN".to_string(),
                                "wireguard" => "WireGuard.WireGuard".to_string(),
                                _ => p.clone()
                            }
                        }).collect();
                        commands.push(format!("winget install {}", winget_packages.join(" ")));
                    } else if which::which("choco").is_ok() {
                        commands.push(format!("choco install -y {}", packages.join(" ")));
                    } else if which::which("scoop").is_ok() {
                        commands.push(format!("scoop install {}", packages.join(" ")));
                    } else {
                        // No package manager found, provide manual download instructions
                        commands.push("# No package manager found. Manual installation required:".to_string());
                        for package in packages {
                            match package.as_str() {
                                "openvpn" => commands.push("# OpenVPN: Download from https://openvpn.net/community-downloads/".to_string()),
                                "wireguard" => commands.push("# WireGuard: Download from https://www.wireguard.com/install/".to_string()),
                                _ => commands.push(format!("# {}: Search for official installer", package)),
                            }
                        }
                    }
                    
                    commands.join("\n")
                }
                
                #[cfg(unix)]
                {
                    "No package manager detected. Please install packages manually.".to_string()
                }
            }
        }
    }
}