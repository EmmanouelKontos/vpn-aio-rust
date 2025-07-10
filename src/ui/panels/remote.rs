use eframe::egui;
use crate::config::{Config, RdpConfig, WolDevice};
use crate::network::NetworkManager;
use crate::ui::components::{Card, GlassButton, StatusIndicator};
use crate::ui::theme::Theme;

pub struct RemotePanel;

impl RemotePanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, network_manager: &mut NetworkManager,
                new_rdp_name: &mut String, new_rdp_host: &mut String, new_rdp_port: &mut String,
                new_rdp_username: &mut String, new_rdp_password: &mut String, new_rdp_domain: &mut String,
                new_wol_name: &mut String, new_wol_mac: &mut String,
                new_wol_ip: &mut String, new_wol_port: &mut String) {
        let theme = Theme::new();
        
        ui.heading("Remote Access");
        ui.add_space(20.0);
        
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() * 0.5 - 8.0);
                    
                    Self::draw_rdp_section(ui, &theme, config, new_rdp_name, new_rdp_host, new_rdp_port,
                                         new_rdp_username, new_rdp_password, new_rdp_domain);
                });
            });
            
            ui.add_space(16.0);
            
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    
                    Self::draw_wol_section(ui, &theme, config, network_manager, new_wol_name, new_wol_mac, new_wol_ip, new_wol_port);
                });
            });
        });
    }
    
    fn draw_rdp_section(ui: &mut egui::Ui, theme: &Theme, config: &mut Config,
                       new_rdp_name: &mut String, new_rdp_host: &mut String, new_rdp_port: &mut String,
                       new_rdp_username: &mut String, new_rdp_password: &mut String, new_rdp_domain: &mut String) {
        
        // RDP Connections List
        Card::show(ui, theme, "Remote Desktop (RDP)", |ui| {
            if config.rdp_configs.is_empty() {
                ui.label(egui::RichText::new("No RDP connections configured").color(theme.text_secondary));
            } else {
                let mut to_remove = None;
                
                for (index, rdp_config) in config.rdp_configs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&rdp_config.name).strong());
                            ui.label(egui::RichText::new(format!("{}:{}", rdp_config.host, rdp_config.port)).color(theme.text_secondary));
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").clicked() {
                                to_remove = Some(index);
                            }
                            
                            if GlassButton::show(ui, theme, "Connect", true).clicked() {
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                runtime.block_on(async {
                                    match crate::network::rdp::connect(rdp_config).await {
                                        Ok(_) => log::info!("RDP connection initiated successfully"),
                                        Err(e) => log::error!("RDP connection failed: {}", e),
                                    }
                                });
                            }
                            
                            #[cfg(windows)]
                            if ui.small_button("🧪").clicked() {
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                runtime.block_on(async {
                                    match crate::network::rdp::test_mstsc_basic().await {
                                        Ok(_) => log::info!("mstsc test passed"),
                                        Err(e) => log::error!("mstsc test failed: {}", e),
                                    }
                                });
                            }
                        });
                    });
                    ui.separator();
                }
                
                if let Some(index) = to_remove {
                    config.rdp_configs.remove(index);
                }
            }
        });
        
        ui.add_space(16.0);
        
        // Add RDP Connection
        Card::show(ui, theme, "Add RDP Connection", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(new_rdp_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.text_edit_singleline(new_rdp_host);
            });
            
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(new_rdp_port);
            });
            
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(new_rdp_username);
            });
            
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(new_rdp_password).password(true));
            });
            
            ui.horizontal(|ui| {
                ui.label("Domain:");
                ui.text_edit_singleline(new_rdp_domain);
            });
            
            ui.add_space(12.0);
            
            if GlassButton::show(ui, theme, "Add Connection", true).clicked() {
                if !new_rdp_name.is_empty() && !new_rdp_host.is_empty() {
                    let port = new_rdp_port.parse::<u16>().unwrap_or(3389);
                    let domain = if new_rdp_domain.is_empty() { None } else { Some(new_rdp_domain.clone()) };
                    
                    config.rdp_configs.push(RdpConfig {
                        name: new_rdp_name.clone(),
                        host: new_rdp_host.clone(),
                        port,
                        username: new_rdp_username.clone(),
                        password: new_rdp_password.clone(),
                        domain,
                    });
                    
                    // Clear input fields
                    new_rdp_name.clear();
                    new_rdp_host.clear();
                    *new_rdp_port = String::from("3389");
                    new_rdp_username.clear();
                    new_rdp_password.clear();
                    new_rdp_domain.clear();
                }
            }
        });
    }
    
    fn draw_wol_section(ui: &mut egui::Ui, theme: &Theme, config: &mut Config, network_manager: &mut NetworkManager,
                       new_wol_name: &mut String, new_wol_mac: &mut String,
                       new_wol_ip: &mut String, new_wol_port: &mut String) {
        
        // WOL Devices List
        Card::show(ui, theme, "Wake-on-LAN Devices", |ui| {
            if config.wol_devices.is_empty() {
                ui.label(egui::RichText::new("No WOL devices configured").color(theme.text_secondary));
            } else {
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
                                    let _ = network_manager.check_device_status(device).await;
                                });
                            }
                            
                            if GlassButton::show(ui, theme, "Wake", true).clicked() {
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                runtime.block_on(async {
                                    let _ = network_manager.wake_device(device).await;
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
            }
        });
        
        ui.add_space(16.0);
        
        // Add WOL Device
        Card::show(ui, theme, "Add WOL Device", |ui| {
            ui.horizontal(|ui| {
                ui.label("Device Name:");
                ui.text_edit_singleline(new_wol_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("MAC Address:");
                ui.text_edit_singleline(new_wol_mac);
            });
            
            ui.horizontal(|ui| {
                ui.label("IP Address:");
                ui.text_edit_singleline(new_wol_ip);
            });
            
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(new_wol_port);
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
}