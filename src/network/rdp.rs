use crate::config::RdpConfig;
use anyhow::Result;
use tokio::process::Command;

pub async fn connect(config: &RdpConfig) -> Result<()> {
    log::info!("Attempting RDP connection to {}:{} with user '{}' and domain '{}'", 
               config.host, config.port, config.username, 
               config.domain.as_deref().unwrap_or("none"));
    
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
pub async fn test_mstsc_basic() -> Result<()> {
    log::info!("Testing basic mstsc functionality");
    
    // Test 1: Just run mstsc without arguments to see if it's available
    let mut cmd = std::process::Command::new("mstsc");
    cmd.arg("/?"); // Show help
    
    log::info!("Testing: mstsc /?");
    
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::info!("mstsc /? output:\nSTDOUT:\n{}\nSTDERR:\n{}", stdout, stderr);
            
            if output.status.success() || stdout.contains("MSTSC") || stderr.contains("MSTSC") {
                log::info!("mstsc is available and working");
                
                // Test 2: Try launching mstsc with no parameters (should open the connection dialog)
                let mut cmd2 = std::process::Command::new("mstsc");
                log::info!("Testing: mstsc (no params)");
                match cmd2.spawn() {
                    Ok(_) => {
                        log::info!("mstsc launched successfully with no parameters");
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("Failed to launch mstsc with no parameters: {}", e);
                        Ok(()) // This is not a critical failure
                    }
                }
            } else {
                Err(anyhow::anyhow!("mstsc doesn't seem to be working properly"))
            }
        }
        Err(e) => {
            log::error!("Failed to run mstsc: {}", e);
            Err(anyhow::anyhow!("mstsc is not available: {}", e))
        }
    }
}

#[cfg(windows)]
pub async fn connect_with_mstsc(config: &RdpConfig) -> Result<()> {
    let port = if config.port == 0 { 3389 } else { config.port };
    
    log::info!("Attempting RDP connection to {}:{}", config.host, port);
    
    // Try the most straightforward approach that should work
    let connection_string = if port == 3389 {
        config.host.clone()
    } else {
        format!("{}:{}", config.host, port)
    };
    
    // Method 1: Direct mstsc command with /v parameter (most reliable)
    let mut cmd = std::process::Command::new("mstsc");
    cmd.arg("/v");
    cmd.arg(&connection_string);
    
    log::info!("Executing: mstsc /v {}", connection_string);
    
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully launched mstsc");
            return Ok(());
        }
        Err(e) => {
            log::warn!("Direct mstsc failed: {}", e);
        }
    }
    
    // Method 2: Try with colon format
    let mut cmd = std::process::Command::new("mstsc");
    cmd.arg(format!("/v:{}", connection_string));
    
    log::info!("Executing: mstsc /v:{}", connection_string);
    
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully launched mstsc with colon format");
            return Ok(());
        }
        Err(e) => {
            log::warn!("Colon format failed: {}", e);
        }
    }
    
    // Method 3: Create minimal RDP file
    connect_with_rdp_file_simple(config).await
}

#[cfg(windows)]
pub async fn connect_with_rdp_file_simple(config: &RdpConfig) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let rdp_file = temp_dir.join(format!("{}.rdp", config.name));
    
    let port = if config.port == 0 { 3389 } else { config.port };
    
    // Create the absolute minimal RDP file that Windows will accept
    let rdp_content = format!(
        "full address:s:{}:{}\r\n\
         username:s:{}\r\n\
         prompt for credentials:i:1\r\n\
         administrative session:i:1\r\n",
        config.host, 
        port, 
        config.username
    );
    
    log::info!("Creating RDP file with content:\n{}", rdp_content);
    std::fs::write(&rdp_file, rdp_content)?;
    
    // Try different ways to launch the RDP file
    
    // Method 1: Direct mstsc with file
    let mut cmd = std::process::Command::new("mstsc");
    cmd.arg(&rdp_file);
    
    log::info!("Trying: mstsc {}", rdp_file.display());
    
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully launched RDP using file method");
            
            // Clean up the RDP file after a delay
            let rdp_file_clone = rdp_file.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let _ = std::fs::remove_file(&rdp_file_clone);
            });
            
            return Ok(());
        }
        Err(e) => {
            log::warn!("Method 1 (mstsc file) failed: {}", e);
        }
    }
    
    // Method 2: Use Windows to open the file
    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/c", "start", "", rdp_file.to_str().unwrap()]);
    
    log::info!("Trying: cmd /c start {}", rdp_file.display());
    
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully launched RDP using Windows file association");
            
            // Clean up the RDP file after a delay
            let rdp_file_clone = rdp_file.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let _ = std::fs::remove_file(&rdp_file_clone);
            });
            
            return Ok(());
        }
        Err(e) => {
            log::warn!("Windows file association failed: {}, trying fallback method", e);
        }
    }
    
    // Method 3: Last resort - just open mstsc without parameters
    let mut cmd = std::process::Command::new("mstsc");
    
    log::info!("Trying fallback: mstsc (no parameters)");
    
    match cmd.spawn() {
        Ok(_) => {
            log::info!("Successfully launched mstsc manually. User will need to enter connection details.");
            log::info!("Please connect to: {}:{}", config.host, port);
            
            // Clean up the RDP file after a delay
            let rdp_file_clone = rdp_file.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                let _ = std::fs::remove_file(&rdp_file_clone);
            });
            
            return Ok(());
        }
        Err(e) => {
            log::error!("All RDP connection methods failed. Last error: {}", e);
            let _ = std::fs::remove_file(&rdp_file); // Clean up immediately on failure
            return Err(anyhow::anyhow!(
                "Failed to start RDP connection to {}:{}. \
                 Please ensure:\n\
                 1. The host is reachable\n\
                 2. RDP is enabled on the target machine\n\
                 3. Windows Remote Desktop Client (mstsc) is available\n\
                 \nAs a last resort, manually connect to: {}:{}\n\
                 \nLast error: {}", 
                config.host, port, config.host, port, e
            ));
        }
    }
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
        // mstsc is built into Windows, check if it's accessible
        let mstsc_available = std::process::Command::new("where")
            .arg("mstsc.exe")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        // Also check system32 directory directly as backup
        let mstsc_path = std::path::Path::new(r"C:\Windows\System32\mstsc.exe");
        let mstsc_exists = mstsc_path.exists();
        
        (mstsc_available || mstsc_exists, false)
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