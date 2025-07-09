use eframe::egui;
use crate::config::Config;
use crate::network::{NetworkManager, VpnStatus};
use crate::ui::components::{Card, GlassButton, StatusIndicator};
use crate::ui::theme::Theme;

pub struct HomePanel;

impl HomePanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, network_manager: &mut NetworkManager) {
        let theme = Theme::new();
        
        ui.heading("Dashboard");
        ui.add_space(20.0);
        
        // VPN Status Overview
        Self::draw_vpn_overview(ui, &theme, config, network_manager);
        ui.add_space(16.0);
        
        // Remote Devices Grid
        Self::draw_remote_devices(ui, &theme, config, network_manager);
    }
    
    fn draw_vpn_overview(ui: &mut egui::Ui, theme: &Theme, config: &Config, network_manager: &NetworkManager) {
        Card::show(ui, theme, "VPN Status", |ui| {
            ui.horizontal(|ui| {
                // VPN Status
                match &network_manager.vpn_status {
                    VpnStatus::Disconnected => {
                        StatusIndicator::show(ui, theme, false, "No VPN Connection");
                    }
                    VpnStatus::Connecting => {
                        StatusIndicator::show(ui, theme, false, "Connecting...");
                    }
                    VpnStatus::Connected(name) => {
                        StatusIndicator::show(ui, theme, true, &format!("Connected to {}", name));
                    }
                    VpnStatus::Error(err) => {
                        ui.label(egui::RichText::new(format!("VPN Error: {}", err)).color(theme.error));
                    }
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{} VPN configs", config.vpn_configs.len())).color(theme.text_secondary));
                });
            });
        });
    }
    
    fn draw_remote_devices(ui: &mut egui::Ui, theme: &Theme, config: &mut Config, network_manager: &mut NetworkManager) {
        Card::show(ui, theme, "Remote Devices", |ui| {
            if config.rdp_configs.is_empty() && config.wol_devices.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("No remote devices configured").color(theme.text_secondary));
                });
                return;
            }
            
            // Create a grid layout for devices
            let available_width = ui.available_width();
            let card_width = 200.0;
            let cards_per_row = (available_width / (card_width + 16.0)).floor() as usize;
            let cards_per_row = cards_per_row.max(1);
            
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(16.0, 16.0);
                
                // RDP Devices
                for rdp_config in &config.rdp_configs {
                    Self::draw_rdp_device_card(ui, theme, rdp_config, card_width);
                }
                
                // WOL Devices
                for wol_device in &config.wol_devices {
                    Self::draw_wol_device_card(ui, theme, wol_device, network_manager, card_width);
                }
            });
        });
    }
    
    fn draw_rdp_device_card(ui: &mut egui::Ui, theme: &Theme, rdp_config: &crate::config::RdpConfig, card_width: f32) {
        egui::Frame::none()
            .fill(theme.surface_variant)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(card_width, 120.0));
                
                ui.vertical_centered(|ui| {
                    // Device icon
                    ui.label(egui::RichText::new("🖥️").size(32.0));
                    ui.add_space(8.0);
                    
                    // Device name
                    ui.label(egui::RichText::new(&rdp_config.name).strong());
                    ui.label(egui::RichText::new(format!("{}:{}", rdp_config.host, rdp_config.port)).color(theme.text_secondary));
                    
                    ui.add_space(8.0);
                    
                    // Connect button
                    if ui.add(egui::Button::new("Connect")
                        .fill(theme.primary)
                        .min_size(egui::vec2(80.0, 28.0))).clicked() {
                        // RDP connection logic
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        runtime.block_on(async {
                            let _ = crate::network::rdp::connect(rdp_config).await;
                        });
                    }
                });
            });
    }
    
    fn draw_wol_device_card(ui: &mut egui::Ui, theme: &Theme, wol_device: &crate::config::WolDevice, network_manager: &mut NetworkManager, card_width: f32) {
        egui::Frame::none()
            .fill(theme.surface_variant)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(card_width, 120.0));
                
                ui.vertical_centered(|ui| {
                    // Device icon with status
                    let is_online = network_manager.wol_devices
                        .iter()
                        .find(|d| d.device.name == wol_device.name)
                        .map(|d| d.is_online)
                        .unwrap_or(false);
                    
                    let icon_color = if is_online { theme.success } else { theme.text_disabled };
                    ui.label(egui::RichText::new("💻").size(32.0).color(icon_color));
                    ui.add_space(8.0);
                    
                    // Device name and status
                    ui.label(egui::RichText::new(&wol_device.name).strong());
                    ui.label(egui::RichText::new(&wol_device.ip_address).color(theme.text_secondary));
                    
                    ui.add_space(4.0);
                    StatusIndicator::show(ui, theme, is_online, if is_online { "Online" } else { "Offline" });
                    
                    ui.add_space(8.0);
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("Wake")
                            .fill(theme.accent)
                            .min_size(egui::vec2(36.0, 24.0))).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                let _ = network_manager.wake_device(wol_device).await;
                            });
                        }
                        
                        if ui.add(egui::Button::new("Ping")
                            .fill(theme.secondary)
                            .min_size(egui::vec2(36.0, 24.0))).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                let _ = network_manager.check_device_status(wol_device).await;
                            });
                        }
                    });
                });
            });
    }
}