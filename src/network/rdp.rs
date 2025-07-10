use crate::config::RdpConfig;
use anyhow::Result;
use tokio::process::Command;

pub async fn connect(config: &RdpConfig) -> Result<()> {
    #[cfg(windows)]
    {
        connect_with_mstsc(config).await
    }
    
    #[cfg(unix)]
    {
        connect_with_xfreerdp(config).await
    }
}

#[cfg(windows)]
pub async fn connect_with_mstsc(config: &RdpConfig) -> Result<()> {
    let mut cmd = Command::new("mstsc");
    
    // Build connection string
    let connection_string = format!("{}:{}", config.host, config.port);
    cmd.arg(format!("/v:{}", connection_string));
    
    // Add username if provided
    if !config.username.is_empty() {
        cmd.arg(format!("/u:{}", config.username));
    }
    
    // Add domain if provided
    if let Some(domain) = &config.domain {
        cmd.arg(format!("/d:{}", domain));
    }
    
    // Add common flags
    cmd.arg("/f"); // Full screen
    cmd.arg("/admin"); // Admin mode
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start RDP connection: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(unix)]
pub async fn connect_with_xfreerdp(config: &RdpConfig) -> Result<()> {
    let mut cmd = Command::new("xfreerdp");
    
    cmd.arg(format!("/v:{}", config.host));
    cmd.arg(format!("/port:{}", config.port));
    cmd.arg(format!("/u:{}", config.username));
    
    if !config.password.is_empty() {
        cmd.arg(format!("/p:{}", config.password));
    }
    
    if let Some(domain) = &config.domain {
        cmd.arg(format!("/d:{}", domain));
    }
    
    cmd.arg("/cert-ignore");
    cmd.arg("/compression");
    cmd.arg("/clipboard");
    cmd.arg("/auto-reconnect");
    cmd.arg("/f");

    let output = cmd.output().await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start RDP connection: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub async fn connect_with_remmina(config: &RdpConfig) -> Result<()> {
    let mut cmd = Command::new("remmina");
    
    let connection_string = format!(
        "rdp://{}:{}@{}:{}",
        config.username,
        config.password,
        config.host,
        config.port
    );
    
    cmd.arg("-c");
    cmd.arg(connection_string);

    let output = cmd.output().await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to start Remmina RDP connection: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

pub fn is_rdp_client_available() -> (bool, bool) {
    #[cfg(windows)]
    {
        // mstsc is built into Windows
        let mstsc_available = std::process::Command::new("where")
            .arg("mstsc")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        (mstsc_available, false)
    }
    
    #[cfg(unix)]
    {
        let xfreerdp_available = std::process::Command::new("which")
            .arg("xfreerdp")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        let remmina_available = std::process::Command::new("which")
            .arg("remmina")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        (xfreerdp_available, remmina_available)
    }
}