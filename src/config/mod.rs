use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub name: String,
    pub config_path: String,
    pub username: String,
    pub password: String,
    pub auto_connect: bool,
    #[serde(default)]
    pub vpn_type: VpnType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VpnType {
    OpenVpn,
    WireGuard,
}

impl Default for VpnType {
    fn default() -> Self {
        VpnType::OpenVpn
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolDevice {
    pub name: String,
    pub mac_address: String,
    pub ip_address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vpn_configs: Vec<VpnConfig>,
    pub rdp_configs: Vec<RdpConfig>,
    pub wol_devices: Vec<WolDevice>,
    pub dark_mode: bool,
    #[serde(default)]
    pub auto_connect_vpn: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vpn_configs: Vec::new(),
            rdp_configs: Vec::new(),
            wol_devices: Vec::new(),
            dark_mode: true,
            auto_connect_vpn: false,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let mut config: Self = serde_json::from_str(&content)?;
            
            // Ensure all VPN configs have a type (for backwards compatibility)
            for vpn_config in &mut config.vpn_configs {
                if vpn_config.config_path.ends_with(".ovpn") {
                    vpn_config.vpn_type = VpnType::OpenVpn;
                } else if vpn_config.config_path.ends_with(".conf") {
                    vpn_config.vpn_type = VpnType::WireGuard;
                }
            }
            
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vpn-manager")
            .join("config.json")
    }
}