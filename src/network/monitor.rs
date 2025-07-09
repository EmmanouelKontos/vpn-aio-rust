use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

pub async fn ping_device(ip: &str) -> Result<bool> {
    match timeout(Duration::from_secs(5), ping_device_internal(ip)).await {
        Ok(result) => result,
        Err(_) => Err(anyhow::anyhow!("Ping timeout for {}", ip)),
    }
}

async fn ping_device_internal(ip: &str) -> Result<bool> {
    let output = tokio::process::Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("-W")
        .arg("3")
        .arg(ip)
        .output()
        .await?;

    Ok(output.status.success())
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

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub is_up: bool,
}

pub async fn scan_network(base_ip: &str) -> Vec<String> {
    let mut active_devices = Vec::new();
    let parts: Vec<&str> = base_ip.split('.').collect();
    
    if parts.len() != 4 {
        return active_devices;
    }
    
    let network_base = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
    let mut tasks = Vec::new();
    
    for i in 1..=254 {
        let ip = format!("{}.{}", network_base, i);
        tasks.push(tokio::spawn(async move {
            if ping_device(&ip).await.unwrap_or(false) {
                Some(ip)
            } else {
                None
            }
        }));
    }
    
    for task in tasks {
        if let Ok(Some(ip)) = task.await {
            active_devices.push(ip);
        }
    }
    
    active_devices
}