use eframe::egui;
use crate::config::Config;
use crate::network::{NetworkManager, VpnStatus};
use crate::ui::components::{Card, StatusIndicator, GlassButton};
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
    
    fn draw_vpn_overview(ui: &mut egui::Ui, theme: &Theme, config: &Config, network_manager: &mut NetworkManager) {
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
            
            // VPN Connection Controls
            if !config.vpn_configs.is_empty() {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Quick Connect:");
                    ui.add_space(8.0);
                    
                    // VPN selector
                    let mut selected_vpn = None;
                    for (index, vpn_config) in config.vpn_configs.iter().enumerate() {
                        if ui.selectable_label(false, &vpn_config.name).clicked() {
                            selected_vpn = Some(index);
                        }
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connect/Disconnect button
                        let is_connected = matches!(&network_manager.vpn_status, VpnStatus::Connected(_));
                        let is_connecting = matches!(&network_manager.vpn_status, VpnStatus::Connecting);
                        
                        if is_connected {
                            if ui.add(egui::Button::new("Disconnect")
                                .fill(theme.error)
                                .min_size(egui::vec2(80.0, 28.0))).clicked() {
                                if let Some(vpn_config) = config.vpn_configs.first() {
                                    let runtime = tokio::runtime::Runtime::new().unwrap();
                                    runtime.block_on(async {
                                        let _ = network_manager.disconnect_vpn(vpn_config).await;
                                    });
                                }
                            }
                        } else if !config.vpn_configs.is_empty() {
                            let button_text = if is_connecting { "Connecting..." } else { "Connect" };
                            let button_enabled = !is_connecting;
                            
                            if ui.add_enabled(button_enabled, egui::Button::new(button_text)
                                .fill(theme.primary)
                                .min_size(egui::vec2(80.0, 28.0))).clicked() {
                                if let Some(vpn_config) = config.vpn_configs.first() {
                                    let runtime = tokio::runtime::Runtime::new().unwrap();
                                    runtime.block_on(async {
                                        let _ = network_manager.connect_vpn(vpn_config).await;
                                    });
                                }
                            }
                        }
                    });
                    
                    if let Some(selected_index) = selected_vpn {
                        if let Some(vpn_config) = config.vpn_configs.get(selected_index) {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                let _ = network_manager.connect_vpn(vpn_config).await;
                            });
                        }
                    }
                });
            }
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
            
            // Create a responsive grid layout for devices
            let available_width = ui.available_width();
            let card_width = 180.0;
            let card_height = 60.0;
            let spacing = 8.0;
            let cards_per_row = ((available_width + spacing) / (card_width + spacing)).floor() as usize;
            let cards_per_row = cards_per_row.max(1);
            
            // Calculate actual card width to fill available space
            let actual_card_width = (available_width - (cards_per_row as f32 - 1.0) * spacing) / cards_per_row as f32;
            
            let mut device_count = 0;
            let total_devices = config.rdp_configs.len() + config.wol_devices.len();
            
            // Use a grid layout for better responsiveness
            egui::Grid::new("remote_devices_grid")
                .num_columns(cards_per_row)
                .spacing(egui::vec2(spacing, spacing))
                .show(ui, |ui| {
                    // RDP Devices
                    for rdp_config in &config.rdp_configs {
                        Self::draw_rdp_device_card_compact(ui, theme, rdp_config, actual_card_width, card_height);
                        device_count += 1;
                        if device_count % cards_per_row == 0 {
                            ui.end_row();
                        }
                    }
                    
                    // WOL Devices
                    for wol_device in &config.wol_devices {
                        Self::draw_wol_device_card_compact(ui, theme, wol_device, network_manager, actual_card_width, card_height);
                        device_count += 1;
                        if device_count % cards_per_row == 0 {
                            ui.end_row();
                        }
                    }
                    
                    // End the last row if needed
                    if device_count % cards_per_row != 0 {
                        ui.end_row();
                    }
                });
        });
    }
    
    fn draw_rdp_device_card_compact(ui: &mut egui::Ui, theme: &Theme, rdp_config: &crate::config::RdpConfig, card_width: f32, card_height: f32) {
        egui::Frame::none()
            .fill(theme.surface_variant)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(card_width, card_height));
                ui.set_max_size(egui::vec2(card_width, card_height));
                
                ui.horizontal(|ui| {
                    // Device icon
                    ui.label(egui::RichText::new("🖥️").size(20.0));
                    ui.add_space(4.0);
                    
                    ui.vertical(|ui| {
                        // Device name
                        ui.label(egui::RichText::new(&rdp_config.name).strong().size(12.0));
                        ui.label(egui::RichText::new(format!("{}:{}", rdp_config.host, rdp_config.port)).color(theme.text_secondary).size(10.0));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Connect button
                        if ui.add(egui::Button::new("Connect")
                            .fill(theme.primary)
                            .min_size(egui::vec2(60.0, 24.0))).clicked() {
                            // RDP connection logic
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                match crate::network::rdp::connect(rdp_config).await {
                                    Ok(_) => {
                                        log::info!("RDP connection initiated to {}", rdp_config.name);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to connect to RDP {}: {}", rdp_config.name, e);
                                    }
                                }
                            });
                        }
                    });
                });
            });
    }
    
    fn draw_wol_device_card_compact(ui: &mut egui::Ui, theme: &Theme, wol_device: &crate::config::WolDevice, network_manager: &mut NetworkManager, card_width: f32, card_height: f32) {
        egui::Frame::none()
            .fill(theme.surface_variant)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(card_width, card_height));
                ui.set_max_size(egui::vec2(card_width, card_height));
                
                ui.horizontal(|ui| {
                    // Device icon with status
                    let is_online = network_manager.wol_devices
                        .iter()
                        .find(|d| d.device.name == wol_device.name)
                        .map(|d| d.is_online)
                        .unwrap_or(false);
                    
                    let icon_color = if is_online { theme.success } else { theme.text_disabled };
                    ui.label(egui::RichText::new("💻").size(20.0).color(icon_color));
                    ui.add_space(4.0);
                    
                    ui.vertical(|ui| {
                        // Device name and IP
                        ui.label(egui::RichText::new(&wol_device.name).strong().size(12.0));
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&wol_device.ip_address).color(theme.text_secondary).size(10.0));
                            ui.add_space(4.0);
                            // Status indicator
                            let status_color = if is_online { theme.success } else { theme.error };
                            ui.label(egui::RichText::new("●").color(status_color).size(8.0));
                            ui.label(egui::RichText::new(if is_online { "Online" } else { "Offline" }).color(status_color).size(9.0));
                        });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Action buttons
                        if ui.add(egui::Button::new("Wake")
                            .fill(theme.accent)
                            .min_size(egui::vec2(40.0, 20.0))).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                match network_manager.wake_device(wol_device).await {
                                    Ok(_) => {
                                        log::info!("WoL packet sent successfully to {}", wol_device.name);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to send WoL packet to {}: {}", wol_device.name, e);
                                    }
                                }
                            });
                        }
                        
                        if ui.add(egui::Button::new("Ping")
                            .fill(theme.secondary)
                            .min_size(egui::vec2(40.0, 20.0))).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                let is_online = network_manager.check_device_status(wol_device).await;
                                log::info!("Device {} is {}", wol_device.name, if is_online { "online" } else { "offline" });
                            });
                        }
                    });
                });
            });
    }
}