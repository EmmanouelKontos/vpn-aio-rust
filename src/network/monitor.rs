use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct DeviceDetectionResult {
    pub is_online: bool,
    pub method_used: String,
    pub response_time: Option<Duration>,
    pub details: String,
}

pub async fn detect_device(ip: &str) -> Result<DeviceDetectionResult> {
    let start_time = std::time::Instant::now();
    
    // Try multiple detection methods in order of reliability
    // ARP table check (fastest)
    match timeout(Duration::from_secs(1), check_arp_table(ip)).await {
        Ok(Ok(true)) => {
            let response_time = start_time.elapsed();
            return Ok(DeviceDetectionResult {
                is_online: true,
                method_used: "ARP".to_string(),
                response_time: Some(response_time),
                details: format!("Device detected via ARP in {:?}", response_time),
            });
        }
        Ok(Ok(false)) => log::debug!("Device {} not detected via ARP", ip),
        Ok(Err(e)) => log::warn!("Error detecting device {} via ARP: {}", ip, e),
        Err(_) => log::warn!("Timeout detecting device {} via ARP", ip),
    }
    
    // TCP port scan (medium speed)
    match timeout(Duration::from_secs(2), tcp_port_scan(ip)).await {
        Ok(Ok(true)) => {
            let response_time = start_time.elapsed();
            return Ok(DeviceDetectionResult {
                is_online: true,
                method_used: "TCP_SCAN".to_string(),
                response_time: Some(response_time),
                details: format!("Device detected via TCP scan in {:?}", response_time),
            });
        }
        Ok(Ok(false)) => log::debug!("Device {} not detected via TCP scan", ip),
        Ok(Err(e)) => log::warn!("Error detecting device {} via TCP scan: {}", ip, e),
        Err(_) => log::warn!("Timeout detecting device {} via TCP scan", ip),
    }
    
    // Ping (slowest but most reliable)
    match timeout(Duration::from_secs(3), ping_device_internal(ip)).await {
        Ok(Ok(true)) => {
            let response_time = start_time.elapsed();
            return Ok(DeviceDetectionResult {
                is_online: true,
                method_used: "PING".to_string(),
                response_time: Some(response_time),
                details: format!("Device detected via PING in {:?}", response_time),
            });
        }
        Ok(Ok(false)) => log::debug!("Device {} not detected via PING", ip),
        Ok(Err(e)) => log::warn!("Error detecting device {} via PING: {}", ip, e),
        Err(_) => log::warn!("Timeout detecting device {} via PING", ip),
    }
    
    // If all methods fail, device is considered offline
    Ok(DeviceDetectionResult {
        is_online: false,
        method_used: "ALL_METHODS".to_string(),
        response_time: None,
        details: "Device not detected by any method".to_string(),
    })
}

pub async fn ping_device(ip: &str) -> Result<bool> {
    match detect_device(ip).await {
        Ok(result) => Ok(result.is_online),
        Err(e) => Err(e),
    }
}

async fn ping_device_internal(ip: &str) -> Result<bool> {
    #[cfg(windows)]
    {
        use std::process::Stdio;
        let mut cmd = tokio::process::Command::new("ping");
        cmd.arg("-n")
           .arg("1")
           .arg("-w")
           .arg("1000")  // Reduced timeout to 1 second
           .arg(ip)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .stdin(Stdio::null());
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let output = cmd.output().await?;
        
        // Check both exit status and output content for better accuracy
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Windows ping success indicators
            Ok(stdout.contains("TTL=") || stdout.contains("bytes="))
        } else {
            Ok(false)
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = tokio::process::Command::new("ping")
            .arg("-c")
            .arg("1")
            .arg("-W")
            .arg("1")  // Reduced timeout to 1 second
            .arg(ip)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Unix ping success indicators
            Ok(stdout.contains("ttl=") || stdout.contains("time="))
        } else {
            Ok(false)
        }
    }
}

async fn check_arp_table(ip: &str) -> Result<bool> {
    #[cfg(windows)]
    {
        let mut cmd = tokio::process::Command::new("arp");
        cmd.arg("-a")
           .arg(ip)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped())
           .stdin(std::process::Stdio::null());
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if IP is in ARP table and not marked as incomplete
        Ok(stdout.contains(ip) && !stdout.contains("incomplete"))
    }
    
    #[cfg(not(windows))]
    {
        let output = tokio::process::Command::new("arp")
            .arg("-n")
            .arg(ip)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check if IP is in ARP table and has a MAC address
        Ok(stdout.contains(ip) && stdout.contains(":") && !stdout.contains("incomplete"))
    }
}

async fn tcp_port_scan(ip: &str) -> Result<bool> {
    // Common ports to check for device presence
    let common_ports = vec![
        22,   // SSH
        23,   // Telnet
        80,   // HTTP
        135,  // RPC
        139,  // NetBIOS
        443,  // HTTPS
        445,  // SMB
        3389, // RDP
        5900, // VNC
        8080, // HTTP Alt
    ];
    
    let ip_addr: IpAddr = ip.parse()?;
    
    // Try to connect to any of the common ports
    for port in common_ports {
        let socket_addr = SocketAddr::new(ip_addr, port);
        
        match timeout(Duration::from_millis(500), TcpStream::connect(socket_addr)).await {
            Ok(Ok(_)) => {
                log::debug!("Device {} detected via TCP port {}", ip, port);
                return Ok(true);
            }
            Ok(Err(_)) => continue, // Port closed or filtered
            Err(_) => continue, // Timeout
        }
    }
    
    Ok(false)
}

// Enhanced device detection with MAC address lookup
pub async fn detect_device_detailed(ip: &str) -> Result<DeviceDetectionResult> {
    let mut result = detect_device(ip).await?;
    
    // If device is detected, try to get additional information
    if result.is_online {
        if let Ok(mac) = get_mac_address(ip).await {
            result.details = format!("{} (MAC: {})", result.details, mac);
        }
    }
    
    Ok(result)
}

async fn get_mac_address(ip: &str) -> Result<String> {
    #[cfg(windows)]
    {
        let mut cmd = tokio::process::Command::new("arp");
        cmd.arg("-a")
           .arg(ip)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped())
           .stdin(std::process::Stdio::null());
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        
        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse MAC address from ARP output
        for line in stdout.lines() {
            if line.contains(ip) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        let output = tokio::process::Command::new("arp")
            .arg("-n")
            .arg(ip)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null())
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse MAC address from ARP output
        for line in stdout.lines() {
            if line.contains(ip) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    return Ok(parts[2].to_string());
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("MAC address not found for {}", ip))
}

pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    
    use network_interface::NetworkInterfaceConfig;
    for iface in network_interface::NetworkInterface::show()? {
        if let Some(addr) = iface.addr.first() {
            interfaces.push(NetworkInterface {
                name: iface.name,
                ip_address: addr.ip().to_string(),
                is_up: !iface.addr.is_empty(),
            });
        }
    }
    
    Ok(interfaces)
}

// Quick device check for UI responsiveness
pub async fn quick_device_check(ip: &str) -> bool {
    // Use only fast methods for quick checks
    // First try ARP table (fastest)
    match timeout(Duration::from_millis(500), check_arp_table(ip)).await {
        Ok(Ok(true)) => return true,
        Ok(Ok(false)) => {},
        Ok(Err(_)) => {},
        Err(_) => {},
    }
    
    // If ARP fails, try a very quick TCP scan on most common ports
    match timeout(Duration::from_millis(1000), tcp_port_scan_quick(ip)).await {
        Ok(Ok(true)) => return true,
        Ok(Ok(false)) => {},
        Ok(Err(_)) => {},
        Err(_) => {},
    }
    
    false
}

// Quick TCP scan for only the most common ports
async fn tcp_port_scan_quick(ip: &str) -> Result<bool> {
    // Only check the most common ports for quick detection
    let quick_ports = vec![
        80,   // HTTP
        443,  // HTTPS
        22,   // SSH
        3389, // RDP
        445,  // SMB
    ];
    
    let ip_addr: IpAddr = ip.parse()?;
    
    for port in quick_ports {
        let socket_addr = SocketAddr::new(ip_addr, port);
        
        match timeout(Duration::from_millis(200), TcpStream::connect(socket_addr)).await {
            Ok(Ok(_)) => {
                log::debug!("Device {} detected via quick TCP port {}", ip, port);
                return Ok(true);
            }
            Ok(Err(_)) => continue,
            Err(_) => continue,
        }
    }
    
    Ok(false)
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub is_up: bool,
}

pub async fn scan_network(base_ip: &str) -> Vec<DeviceInfo> {
    let mut active_devices = Vec::new();
    let parts: Vec<&str> = base_ip.split('.').collect();
    
    if parts.len() != 4 {
        return active_devices;
    }
    
    let network_base = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
    let mut tasks = Vec::new();
    
    log::info!("Scanning network {}.0/24...", network_base);
    
    for i in 1..=254 {
        let ip = format!("{}.{}", network_base, i);
        tasks.push(tokio::spawn(async move {
            match detect_device_detailed(&ip).await {
                Ok(result) if result.is_online => {
                    Some(DeviceInfo {
                        ip_address: ip,
                        method_detected: result.method_used,
                        response_time: result.response_time,
                        details: result.details,
                    })
                }
                _ => None,
            }
        }));
    }
    
    for task in tasks {
        if let Ok(Some(device_info)) = task.await {
            active_devices.push(device_info);
        }
    }
    
    log::info!("Network scan complete. Found {} active devices", active_devices.len());
    active_devices
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub ip_address: String,
    pub method_detected: String,
    pub response_time: Option<Duration>,
    pub details: String,
}