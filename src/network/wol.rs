use crate::config::WolDevice;
use anyhow::Result;
use wake_on_lan::MagicPacket;

pub async fn wake_device(device: &WolDevice) -> Result<()> {
    let mac_bytes = parse_mac_address(&device.mac_address)?;
    let magic_packet = MagicPacket::new(&mac_bytes);
    
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    
    let broadcast_addr = format!("255.255.255.255:{}", device.port);
    socket.send_to(magic_packet.magic_bytes(), broadcast_addr)?;
    
    if !device.ip_address.is_empty() && device.ip_address != "255.255.255.255" {
        let device_addr = format!("{}:{}", device.ip_address, device.port);
        socket.send_to(magic_packet.magic_bytes(), device_addr)?;
    }
    
    Ok(())
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