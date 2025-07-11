use crate::config::WolDevice;
use crate::network::monitor::{get_network_interfaces, NetworkInterface};
use anyhow::Result;
use wake_on_lan::MagicPacket;

pub async fn wake_device(device: &WolDevice) -> Result<()> {
    let mac_bytes = parse_mac_address(&device.mac_address)?;
    let magic_packet = MagicPacket::new(&mac_bytes);
    
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    
    let mut sent_count = 0;
    let mut errors = Vec::new();
    
    // Send to global broadcast
    let global_broadcast = format!("255.255.255.255:{}", device.port);
    match socket.send_to(magic_packet.magic_bytes(), &global_broadcast) {
        Ok(_) => {
            sent_count += 1;
            log::info!("WoL packet sent to global broadcast {}", global_broadcast);
        }
        Err(e) => {
            errors.push(format!("Global broadcast failed: {}", e));
            log::warn!("Failed to send WoL packet to global broadcast: {}", e);
        }
    }
    
    // Send to device IP if specified
    if !device.ip_address.is_empty() && device.ip_address != "255.255.255.255" {
        let device_addr = format!("{}:{}", device.ip_address, device.port);
        match socket.send_to(magic_packet.magic_bytes(), &device_addr) {
            Ok(_) => {
                sent_count += 1;
                log::info!("WoL packet sent to device IP {}", device_addr);
            }
            Err(e) => {
                errors.push(format!("Device IP failed: {}", e));
                log::warn!("Failed to send WoL packet to device IP: {}", e);
            }
        }
    }
    
    // Send to all possible network broadcast addresses
    if let Ok(interfaces) = get_network_interfaces().await {
        for interface in interfaces {
            if let Ok(broadcast_addr) = calculate_broadcast_address(&interface.ip_address) {
                let broadcast_target = format!("{}:{}", broadcast_addr, device.port);
                match socket.send_to(magic_packet.magic_bytes(), &broadcast_target) {
                    Ok(_) => {
                        sent_count += 1;
                        log::info!("WoL packet sent to network broadcast {}", broadcast_target);
                    }
                    Err(e) => {
                        log::warn!("Failed to send WoL packet to {}: {}", broadcast_target, e);
                    }
                }
            }
        }
    }
    
    if sent_count > 0 {
        log::info!("Successfully sent {} WoL packets for device {}", sent_count, device.name);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to send any WoL packets. Errors: {}", errors.join(", ")))
    }
}

fn parse_mac_address(mac_str: &str) -> Result<[u8; 6]> {
    let cleaned = mac_str.replace([':', '-'], "");
    
    if cleaned.len() != 12 {
        return Err(anyhow::anyhow!("Invalid MAC address format"));
    }
    
    let mut mac_bytes = [0u8; 6];
    for (i, chunk) in cleaned.chars().collect::<Vec<_>>().chunks(2).enumerate() {
        let hex_str: String = chunk.iter().collect();
        mac_bytes[i] = u8::from_str_radix(&hex_str, 16)
            .map_err(|_| anyhow::anyhow!("Invalid MAC address format"))?;
    }
    
    Ok(mac_bytes)
}

pub fn validate_mac_address(mac_str: &str) -> bool {
    parse_mac_address(mac_str).is_ok()
}

pub fn format_mac_address(mac_str: &str) -> String {
    let cleaned = mac_str.replace([':', '-'], "");
    if cleaned.len() == 12 {
        format!(
            "{}:{}:{}:{}:{}:{}",
            &cleaned[0..2],
            &cleaned[2..4],
            &cleaned[4..6],
            &cleaned[6..8],
            &cleaned[8..10],
            &cleaned[10..12]
        ).to_uppercase()
    } else {
        mac_str.to_string()
    }
}

fn calculate_broadcast_address(ip: &str) -> Result<String> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(anyhow::anyhow!("Invalid IP address format"));
    }
    
    // Default to /24 subnet for broadcast calculation
    let broadcast = format!("{}.{}.{}.255", parts[0], parts[1], parts[2]);
    Ok(broadcast)
}