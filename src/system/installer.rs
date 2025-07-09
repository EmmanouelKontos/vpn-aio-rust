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
    
    pub fn get_install_command(&self, packages: &[String]) -> String {
        match self.package_manager {
            PackageManager::Apt => format!("sudo apt install -y {}", packages.join(" ")),
            PackageManager::Pacman => format!("sudo pacman -S --noconfirm {}", packages.join(" ")),
            PackageManager::Dnf => format!("sudo dnf install -y {}", packages.join(" ")),
            PackageManager::Yum => format!("sudo yum install -y {}", packages.join(" ")),
            PackageManager::Zypper => format!("sudo zypper install -y {}", packages.join(" ")),
            PackageManager::Unknown => "Unknown package manager".to_string(),
        }
    }
}