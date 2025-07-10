use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: String,
    pub release_notes: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Clone)]
pub struct AppUpdater {
    repo_owner: String,
    repo_name: String,
    current_version: String,
}

impl AppUpdater {
    pub fn new(repo_owner: &str, repo_name: &str, current_version: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            current_version: current_version.to_string(),
        }
    }
    
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );
        
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "vpn-manager")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch release info: {}", response.status()));
        }
        
        let release: GitHubRelease = response.json().await?;
        
        let latest_version = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);
        let current_version = self.current_version.strip_prefix('v').unwrap_or(&self.current_version);
        
        let update_available = self.is_newer_version(latest_version, current_version)?;
        
        // Find the appropriate asset for the current platform
        let download_url = self.get_download_url(&release.assets)?;
        
        Ok(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            update_available,
            download_url,
            release_notes: release.body,
        })
    }
    
    fn is_newer_version(&self, latest: &str, current: &str) -> Result<bool> {
        use semver::Version;
        
        let latest_version = Version::parse(latest)?;
        let current_version = Version::parse(current)?;
        
        Ok(latest_version > current_version)
    }
    
    fn get_download_url(&self, assets: &[GitHubAsset]) -> Result<String> {
        // Determine platform
        #[cfg(target_os = "windows")]
        let platform_keywords = ["windows", "win32", "win64", "x86_64", "amd64"];
        
        #[cfg(target_os = "linux")]
        let platform_keywords = ["linux", "x86_64", "amd64"];
        
        #[cfg(target_os = "macos")]
        let platform_keywords = ["macos", "darwin", "osx", "x86_64", "amd64"];
        
        // Look for platform-specific binary
        for asset in assets {
            let name_lower = asset.name.to_lowercase();
            if platform_keywords.iter().any(|&keyword| name_lower.contains(keyword)) {
                return Ok(asset.browser_download_url.clone());
            }
        }
        
        // Fallback to first asset
        if let Some(asset) = assets.first() {
            return Ok(asset.browser_download_url.clone());
        }
        
        Err(anyhow::anyhow!("No suitable download asset found"))
    }
    
    pub async fn download_and_install_update(&self, update_info: &UpdateInfo) -> Result<()> {
        let temp_dir = std::env::temp_dir();
        
        // Determine file extension based on platform
        #[cfg(windows)]
        let extension = ".exe";
        #[cfg(not(windows))]
        let extension = "";
        
        let filename = format!("vpn-manager-{}{}", update_info.latest_version, extension);
        let temp_file = temp_dir.join(&filename);
        
        // Download the update
        let client = reqwest::Client::new();
        let response = client.get(&update_info.download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download update: {}", response.status()));
        }
        
        let content = response.bytes().await?;
        std::fs::write(&temp_file, content)?;
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_file)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&temp_file, perms)?;
        }
        
        // Get current executable path
        let current_exe = std::env::current_exe()?;
        let backup_path = format!("{}.backup", current_exe.display());
        
        // Create backup of current executable
        std::fs::copy(&current_exe, &backup_path)?;
        
        // On Windows, we need to handle the file replacement differently
        #[cfg(windows)]
        {
            // Use a batch script to replace the executable after the current process exits
            let batch_script = format!(
                r#"@echo off
timeout /t 2 /nobreak >nul
move "{}" "{}"
start "" "{}"
del "%~f0""#,
                temp_file.display(),
                current_exe.display(),
                current_exe.display()
            );
            
            let script_path = temp_dir.join("update.bat");
            std::fs::write(&script_path, batch_script)?;
            
            // Start the batch script
            Command::new("cmd")
                .args(["/C", "start", "", script_path.to_str().unwrap()])
                .spawn()?;
        }
        
        #[cfg(not(windows))]
        {
            // On Unix systems, we can replace the file directly
            std::fs::copy(&temp_file, &current_exe)?;
            std::fs::remove_file(&temp_file)?;
        }
        
        Ok(())
    }
    
    pub fn restart_application(&self) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        
        Command::new(current_exe)
            .spawn()?;
        
        std::process::exit(0);
    }
    
    pub fn get_changelog_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/releases",
            self.repo_owner, self.repo_name
        )
    }
}