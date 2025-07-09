use crate::config::RdpConfig;
use anyhow::Result;
use tokio::process::Command;

pub async fn connect(config: &RdpConfig) -> Result<()> {
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