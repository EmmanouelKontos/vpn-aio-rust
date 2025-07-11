use crate::config::VpnConfig;
use anyhow::Result;
use tokio::process::Command as TokioCommand;

pub async fn connect(config: &VpnConfig) -> Result<()> {
    #[cfg(windows)]
    {
        connect_windows(config).await
    }
    
    #[cfg(unix)]
    {
        connect_unix(config).await
    }
}

#[cfg(windows)]
pub async fn connect_windows(config: &VpnConfig) -> Result<()> {
    let mut cmd = TokioCommand::new("openvpn");
    cmd.arg("--config")
        .arg(&config.config_path)
        .arg("--daemon")
        .arg("--auth-user-pass")
        .arg("NUL")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd.output().await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start OpenVPN: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(unix)]
pub async fn connect_unix(config: &VpnConfig) -> Result<()> {
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
    #[cfg(windows)]
    {
        disconnect_windows().await
    }
    
    #[cfg(unix)]
    {
        disconnect_unix().await
    }
}

#[cfg(windows)]
pub async fn disconnect_windows() -> Result<()> {
    let mut cmd = TokioCommand::new("taskkill");
    cmd.arg("/F")
        .arg("/IM")
        .arg("openvpn.exe")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd.output().await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to stop OpenVPN: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(unix)]
pub async fn disconnect_unix() -> Result<()> {
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
    #[cfg(windows)]
    {
        get_status_windows().await
    }
    
    #[cfg(unix)]
    {
        get_status_unix().await
    }
}

pub async fn check_connection_status() -> Result<bool> {
    get_status().await
}

#[cfg(windows)]
pub async fn get_status_windows() -> Result<bool> {
    let mut cmd = TokioCommand::new("tasklist");
    cmd.arg("/FI")
        .arg("IMAGENAME eq openvpn.exe")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null());
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd.output().await?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("openvpn.exe"))
}

#[cfg(unix)]
pub async fn get_status_unix() -> Result<bool> {
    let output = TokioCommand::new("pgrep")
        .arg("openvpn")
        .output()
        .await?;

    Ok(output.status.success())
}

pub fn get_available_configs() -> Result<Vec<String>> {
    let mut configs = Vec::new();
    
    #[cfg(windows)]
    {
        let appdata_config = format!("{}\\OpenVPN\\config", std::env::var("APPDATA").unwrap_or_default());
        let config_dirs = vec![
            "C:\\Program Files\\OpenVPN\\config",
            "C:\\Program Files (x86)\\OpenVPN\\config",
            &appdata_config,
        ];
        
        for config_dir in config_dirs {
            if let Ok(entries) = std::fs::read_dir(config_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("ovpn") {
                        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                            configs.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(unix)]
    {
        let config_dir = std::path::Path::new("/etc/openvpn");
        
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
    }

    Ok(configs)
}