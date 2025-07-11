use crate::config::{RdpConfig, VpnConfig, VpnType, WolDevice};
use anyhow::Result;
use std::time::Duration;

pub mod monitor;
pub mod vpn;
pub mod wireguard;
pub mod rdp;
pub mod wol;

#[derive(Clone)]
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
    
    pub async fn initialize(&mut self, vpn_configs: &[VpnConfig], wol_devices: &[WolDevice]) -> Result<()> {
        // Check if any VPN is already connected
        for config in vpn_configs {
            if let Ok(is_connected) = self.check_vpn_status(config).await {
                if is_connected {
                    self.vpn_status = VpnStatus::Connected(config.name.clone());
                    break;
                }
            }
        }
        
        // Initialize WoL device statuses
        self.wol_devices = wol_devices.iter().map(|device| {
            WolDeviceStatus {
                device: device.clone(),
                is_online: false,
                last_checked: std::time::Instant::now() - Duration::from_secs(60), // Force initial check
            }
        }).collect();
        
        Ok(())
    }
    
    pub async fn refresh_vpn_status(&mut self, vpn_configs: &[VpnConfig]) -> Result<()> {
        // First check if currently connected VPN is still active
        if let VpnStatus::Connected(name) = &self.vpn_status {
            if let Some(config) = vpn_configs.iter().find(|c| &c.name == name) {
                if let Ok(is_connected) = self.check_vpn_status(config).await {
                    if !is_connected {
                        self.vpn_status = VpnStatus::Disconnected;
                    }
                    return Ok(());
                }
            }
        }
        
        // If no specific VPN is marked as connected, check all configs
        for config in vpn_configs {
            if let Ok(is_connected) = self.check_vpn_status(config).await {
                if is_connected {
                    self.vpn_status = VpnStatus::Connected(config.name.clone());
                    return Ok(());
                }
            }
        }
        
        // If no VPN is connected, mark as disconnected
        if !matches!(self.vpn_status, VpnStatus::Connecting) {
            self.vpn_status = VpnStatus::Disconnected;
        }
        
        Ok(())
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
    
    pub async fn check_any_vpn_connected(&mut self, vpn_configs: &[VpnConfig]) -> Result<Option<String>> {
        for config in vpn_configs {
            if let Ok(is_connected) = self.check_vpn_status(config).await {
                if is_connected {
                    return Ok(Some(config.name.clone()));
                }
            }
        }
        Ok(None)
    }

    pub async fn connect_rdp(&mut self, config: &RdpConfig) -> Result<()> {
        rdp::connect(config).await
    }

    pub async fn wake_device(&mut self, device: &WolDevice) -> Result<()> {
        let result = wol::wake_device(device).await;
        
        // After sending wake packet, wait a bit then check status multiple times
        if result.is_ok() {
            log::info!("WoL packet sent to {}, waiting for device to wake up...", device.name);
            
            // Check status multiple times with increasing delays
            for i in 0..5 {
                let delay = Duration::from_millis(2000 + (i * 1000)); // 2s, 3s, 4s, 5s, 6s
                tokio::time::sleep(delay).await;
                
                let is_online = self.check_device_status(device).await;
                if is_online {
                    log::info!("Device {} is now online after WoL", device.name);
                    break;
                }
                
                log::debug!("Device {} still offline, attempt {} of 5", device.name, i + 1);
            }
        }
        
        result
    }

    pub async fn check_device_status(&mut self, device: &WolDevice) -> bool {
        let detection_result = monitor::detect_device(&device.ip_address).await;
        
        let is_online = match detection_result {
            Ok(result) => {
                log::info!("Device {} detection: {}", device.name, result.details);
                result.is_online
            }
            Err(e) => {
                log::warn!("Failed to detect device {}: {}", device.name, e);
                false
            }
        };
        
        // Update the device status in our list
        if let Some(device_status) = self.wol_devices.iter_mut().find(|d| d.device.name == device.name) {
            device_status.is_online = is_online;
            device_status.last_checked = std::time::Instant::now();
        }
        
        is_online
    }

    pub async fn update_device_statuses(&mut self) -> Result<()> {
        let mut updates = Vec::new();
        
        for (index, device_status) in self.wol_devices.iter().enumerate() {
            if device_status.last_checked.elapsed() > Duration::from_secs(30) {
                match monitor::detect_device(&device_status.device.ip_address).await {
                    Ok(detection_result) => {
                        log::debug!("Device {} status update: {}", device_status.device.name, detection_result.details);
                        updates.push((index, detection_result.is_online));
                    }
                    Err(e) => {
                        log::warn!("Failed to detect device {}: {}", device_status.device.name, e);
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
    
    pub fn sync_wol_devices(&mut self, config_devices: &[WolDevice]) {
        // Remove devices that are no longer in config
        self.wol_devices.retain(|status| {
            config_devices.iter().any(|config_device| config_device.name == status.device.name)
        });
        
        // Add new devices from config
        for config_device in config_devices {
            if !self.wol_devices.iter().any(|status| status.device.name == config_device.name) {
                self.wol_devices.push(WolDeviceStatus {
                    device: config_device.clone(),
                    is_online: false,
                    last_checked: std::time::Instant::now() - Duration::from_secs(60), // Force initial check
                });
            }
        }
    }
    
    pub async fn quick_update_device_statuses(&mut self) -> Result<()> {
        // Use quick checks for more frequent updates
        for device_status in &mut self.wol_devices {
            if device_status.last_checked.elapsed() > Duration::from_secs(10) {
                let is_online = monitor::quick_device_check(&device_status.device.ip_address).await;
                if device_status.is_online != is_online {
                    log::info!("Device {} status changed: {} -> {}", 
                        device_status.device.name, 
                        device_status.is_online, 
                        is_online
                    );
                    device_status.is_online = is_online;
                }
                device_status.last_checked = std::time::Instant::now();
            }
        }
        Ok(())
    }
}