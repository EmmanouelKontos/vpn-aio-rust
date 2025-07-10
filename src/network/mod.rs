use crate::config::{RdpConfig, VpnConfig, VpnType, WolDevice};
use anyhow::Result;
use std::time::Duration;

pub mod monitor;
pub mod vpn;
pub mod wireguard;
pub mod rdp;
pub mod wol;

pub struct NetworkManager {
    pub vpn_status: VpnStatus,
    pub rdp_connections: Vec<RdpConnection>,
    pub wol_devices: Vec<WolDeviceStatus>,
}

#[derive(Debug, Clone)]
pub enum VpnStatus {
    Disconnected,
    Connecting,
    Connected(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RdpConnection {
    pub config: RdpConfig,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct WolDeviceStatus {
    pub device: WolDevice,
    pub is_online: bool,
    pub last_checked: std::time::Instant,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            vpn_status: VpnStatus::Disconnected,
            rdp_connections: Vec::new(),
            wol_devices: Vec::new(),
        }
    }

    pub async fn connect_vpn(&mut self, config: &VpnConfig) -> Result<()> {
        self.vpn_status = VpnStatus::Connecting;
        
        let result = match config.vpn_type {
            VpnType::OpenVpn => vpn::connect(config).await,
            VpnType::WireGuard => wireguard::connect(config).await,
        };
        
        match result {
            Ok(_) => {
                self.vpn_status = VpnStatus::Connected(config.name.clone());
                Ok(())
            }
            Err(e) => {
                self.vpn_status = VpnStatus::Error(e.to_string());
                Err(e)
            }
        }
    }

    pub async fn disconnect_vpn(&mut self, config: &VpnConfig) -> Result<()> {
        let result = match config.vpn_type {
            VpnType::OpenVpn => vpn::disconnect().await,
            VpnType::WireGuard => wireguard::disconnect(config).await,
        };
        
        match result {
            Ok(_) => {
                self.vpn_status = VpnStatus::Disconnected;
                Ok(())
            }
            Err(e) => {
                self.vpn_status = VpnStatus::Error(e.to_string());
                Err(e)
            }
        }
    }

    pub async fn check_vpn_status(&mut self, config: &VpnConfig) -> Result<bool> {
        match config.vpn_type {
            VpnType::OpenVpn => vpn::check_connection_status().await,
            VpnType::WireGuard => wireguard::check_connection_status(config).await,
        }
    }

    pub async fn connect_rdp(&mut self, config: &RdpConfig) -> Result<()> {
        rdp::connect(config).await
    }

    pub async fn wake_device(&self, device: &WolDevice) -> Result<()> {
        wol::wake_device(device).await
    }

    pub async fn check_device_status(&mut self, device: &WolDevice) -> bool {
        monitor::ping_device(&device.ip_address).await.unwrap_or(false)
    }

    pub async fn update_device_statuses(&mut self) -> Result<()> {
        let mut updates = Vec::new();
        
        for (index, device_status) in self.wol_devices.iter().enumerate() {
            if device_status.last_checked.elapsed() > Duration::from_secs(30) {
                match monitor::ping_device(&device_status.device.ip_address).await {
                    Ok(is_online) => updates.push((index, is_online)),
                    Err(e) => {
                        log::warn!("Failed to ping device {}: {}", device_status.device.name, e);
                        // Still update last_checked to avoid constant retries
                        updates.push((index, false));
                    }
                }
            }
        }
        
        for (index, is_online) in updates {
            if let Some(device_status) = self.wol_devices.get_mut(index) {
                device_status.is_online = is_online;
                device_status.last_checked = std::time::Instant::now();
            }
        }
        
        Ok(())
    }
}