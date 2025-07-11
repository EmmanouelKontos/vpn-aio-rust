use eframe::egui;
use crate::config::{Config, WolDevice};
use crate::network::{NetworkManager, WolDeviceStatus};
use crate::ui::components::{Card, GlassButton, InputField, StatusIndicator};
use crate::ui::theme::Theme;

pub struct WolPanel;

impl WolPanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, network_manager: &mut NetworkManager,
                new_wol_name: &mut String, new_wol_mac: &mut String,
                new_wol_ip: &mut String, new_wol_port: &mut String) {
        let theme = Theme::new();
        
        ui.heading("Wake-on-LAN");
        ui.add_space(20.0);
        
        Self::draw_devices_card(ui, &theme, config, network_manager);
        ui.add_space(16.0);
        
        Self::draw_add_device_card(ui, &theme, config, new_wol_name, new_wol_mac, new_wol_ip, new_wol_port);
        ui.add_space(16.0);
        
        Self::draw_network_scan_card(ui, &theme);
    }
    
    fn draw_devices_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config, network_manager: &mut NetworkManager) {
        Card::show(ui, theme, "WOL Devices", |ui| {
            if config.wol_devices.is_empty() {
                ui.label(egui::RichText::new("No WOL devices configured").color(theme.text_secondary));
                return;
            }
            
            let mut to_remove = None;
            
            for (index, device) in config.wol_devices.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&device.name).strong());
                        ui.label(egui::RichText::new(format!("IP: {}", device.ip_address)).color(theme.text_secondary));
                        ui.label(egui::RichText::new(format!("MAC: {}", device.mac_address)).color(theme.text_secondary));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            to_remove = Some(index);
                        }
                        
                        if GlassButton::show(ui, theme, "Ping", false).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                let is_online = network_manager.check_device_status(device).await;
                                log::info!("Device {} is {}", device.name, if is_online { "online" } else { "offline" });
                            });
                        }
                        
                        if GlassButton::show(ui, theme, "Wake Up", true).clicked() {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            runtime.block_on(async {
                                match network_manager.wake_device(device).await {
                                    Ok(_) => {
                                        log::info!("WoL packet sent successfully to {}", device.name);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to send WoL packet to {}: {}", device.name, e);
                                    }
                                }
                            });
                        }
                        
                        let is_online = network_manager.wol_devices
                            .iter()
                            .find(|d| d.device.name == device.name)
                            .map(|d| d.is_online)
                            .unwrap_or(false);
                        
                        StatusIndicator::show(ui, theme, is_online, if is_online { "Online" } else { "Offline" });
                    });
                });
                ui.separator();
            }
            
            if let Some(index) = to_remove {
                config.wol_devices.remove(index);
            }
        });
    }
    
    fn draw_add_device_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config,
                           new_wol_name: &mut String, new_wol_mac: &mut String,
                           new_wol_ip: &mut String, new_wol_port: &mut String) {
        Card::show(ui, theme, "Add WOL Device", |ui| {
            ui.label("Add new Wake-on-LAN device");
            ui.add_space(8.0);
            
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Device Name:");
                        ui.text_edit_singleline(new_wol_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("MAC Address:");
                        ui.text_edit_singleline(new_wol_mac);
                    });
                });
                
                columns[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("IP Address:");
                        ui.text_edit_singleline(new_wol_ip);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        ui.text_edit_singleline(new_wol_port);
                    });
                });
            });
            
            ui.add_space(12.0);
            
            if GlassButton::show(ui, theme, "Add Device", true).clicked() {
                if !new_wol_name.is_empty() && !new_wol_mac.is_empty() {
                    let port = new_wol_port.parse::<u16>().unwrap_or(9);
                    let ip_address = if new_wol_ip.is_empty() { 
                        "255.255.255.255".to_string() 
                    } else { 
                        new_wol_ip.clone() 
                    };
                    
                    config.wol_devices.push(WolDevice {
                        name: new_wol_name.clone(),
                        mac_address: new_wol_mac.clone(),
                        ip_address,
                        port,
                    });
                    
                    // Clear input fields
                    new_wol_name.clear();
                    new_wol_mac.clear();
                    new_wol_ip.clear();
                    *new_wol_port = String::from("9");
                }
            }
        });
    }
    
    fn draw_network_scan_card(ui: &mut egui::Ui, theme: &Theme) {
        Card::show(ui, theme, "Network Scanner", |ui| {
            ui.label(egui::RichText::new("Scan your network to find active devices").color(theme.text_secondary));
            ui.add_space(8.0);
            
            if GlassButton::show(ui, theme, "Scan Network", true).clicked() {
                // Placeholder for network scanning functionality
                // Would need proper async state management
            }
            
            ui.add_space(12.0);
            ui.label("Network scanning functionality would appear here");
        });
    }
}