use crate::config::VpnConfig;
use anyhow::Result;
use std::process::Command;
use tokio::process::Command as TokioCommand;

pub async fn connect(config: &VpnConfig) -> Result<()> {
    let output = TokioCommand::new("openvpn")
        .arg("--config")
        .arg(&config.config_path)
        .arg("--daemon")
        .arg("--auth-user-pass")
        .arg("/dev/stdin")
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start OpenVPN: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub async fn disconnect() -> Result<()> {
    let output = TokioCommand::new("pkill")
        .arg("openvpn")
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to stop OpenVPN: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub async fn get_status() -> Result<bool> {
    let output = TokioCommand::new("pgrep")
        .arg("openvpn")
        .output()
        .await?;

    Ok(output.status.success())
}

pub fn get_available_configs() -> Result<Vec<String>> {
    let config_dir = std::path::Path::new("/etc/openvpn");
    let mut configs = Vec::new();

    if config_dir.exists() {
        for entry in std::fs::read_dir(config_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ovpn") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    configs.push(file_name.to_string());
                }
            }
        }
    }

    Ok(configs)
}