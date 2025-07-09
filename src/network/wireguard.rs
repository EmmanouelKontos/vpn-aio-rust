use crate::config::VpnConfig;
use anyhow::Result;
use tokio::process::Command;
use std::path::Path;

pub async fn connect(config: &VpnConfig) -> Result<()> {
    // Check if config file exists
    if !Path::new(&config.config_path).exists() {
        return Err(anyhow::anyhow!("WireGuard config file not found: {}", config.config_path));
    }
    
    // Use wg-quick to bring up the interface
    let output = Command::new("sudo")
        .args(&["wg-quick", "up", &config.config_path])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start WireGuard: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

pub async fn disconnect(config: &VpnConfig) -> Result<()> {
    // Use wg-quick to bring down the interface
    let output = Command::new("sudo")
        .args(&["wg-quick", "down", &config.config_path])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to stop WireGuard: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

pub async fn get_status(interface_name: &str) -> Result<bool> {
    let output = Command::new("wg")
        .args(&["show", interface_name])
        .output()
        .await?;
    
    Ok(output.status.success())
}

pub async fn get_interface_from_config(config_path: &str) -> Result<String> {
    let content = std::fs::read_to_string(config_path)?;
    
    // Parse the interface name from the config file
    for line in content.lines() {
        if line.trim().starts_with("# Interface:") {
            if let Some(interface) = line.split(':').nth(1) {
                return Ok(interface.trim().to_string());
            }
        }
    }
    
    // Fallback: try to extract from filename
    let path = Path::new(config_path);
    if let Some(stem) = path.file_stem() {
        if let Some(name) = stem.to_str() {
            return Ok(name.to_string());
        }
    }
    
    // Default interface name
    Ok("wg0".to_string())
}

pub async fn list_interfaces() -> Result<Vec<String>> {
    let output = Command::new("wg")
        .arg("show")
        .output()
        .await?;
    
    if !output.status.success() {
        return Ok(Vec::new());
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();
    
    for line in output_str.lines() {
        if line.starts_with("interface: ") {
            if let Some(interface) = line.strip_prefix("interface: ") {
                interfaces.push(interface.to_string());
            }
        }
    }
    
    Ok(interfaces)
}

pub fn get_available_configs() -> Result<Vec<String>> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let user_config_dir = format!("{}/.config/wireguard", home_dir);
    
    let config_dirs = vec![
        "/etc/wireguard",
        &user_config_dir,
    ];
    
    let mut configs = Vec::new();
    
    for config_dir in config_dirs {
        if let Ok(entries) = std::fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("conf") {
                    configs.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(configs)
}

pub fn validate_config(config_path: &str) -> Result<bool> {
    let content = std::fs::read_to_string(config_path)?;
    
    // Basic validation - check for required sections
    let has_interface = content.contains("[Interface]");
    let has_peer = content.contains("[Peer]");
    let has_private_key = content.contains("PrivateKey");
    let has_public_key = content.contains("PublicKey");
    
    Ok(has_interface && has_peer && has_private_key && has_public_key)
}

pub fn get_config_info(config_path: &str) -> Result<WireGuardConfigInfo> {
    let content = std::fs::read_to_string(config_path)?;
    let mut info = WireGuardConfigInfo::default();
    
    let mut current_section = "";
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line;
            continue;
        }
        
        if current_section == "[Interface]" {
            if line.starts_with("Address") {
                if let Some(address) = line.split('=').nth(1) {
                    info.address = address.trim().to_string();
                }
            } else if line.starts_with("DNS") {
                if let Some(dns) = line.split('=').nth(1) {
                    info.dns = dns.trim().to_string();
                }
            }
        } else if current_section == "[Peer]" {
            if line.starts_with("Endpoint") {
                if let Some(endpoint) = line.split('=').nth(1) {
                    info.endpoint = endpoint.trim().to_string();
                }
            } else if line.starts_with("AllowedIPs") {
                if let Some(allowed_ips) = line.split('=').nth(1) {
                    info.allowed_ips = allowed_ips.trim().to_string();
                }
            }
        }
    }
    
    Ok(info)
}

#[derive(Debug, Default)]
pub struct WireGuardConfigInfo {
    pub address: String,
    pub dns: String,
    pub endpoint: String,
    pub allowed_ips: String,
}