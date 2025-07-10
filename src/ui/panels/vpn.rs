use eframe::egui;
use crate::config::{Config, VpnConfig, VpnType};
use crate::network::{NetworkManager, VpnStatus};
use crate::ui::components::{Card, GlassButton, StatusIndicator};
use crate::ui::theme::Theme;

pub struct VpnPanel;

impl VpnPanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, network_manager: &mut NetworkManager,
                new_vpn_name: &mut String, new_vpn_config_path: &mut String,
                new_vpn_username: &mut String, new_vpn_password: &mut String,
                new_vpn_type: &mut VpnType, loading_actions: &std::collections::HashSet<String>,
                animation_time: f32) {
        let theme = Theme::new();
        
        ui.heading("VPN Management");
        ui.add_space(20.0);
        
        Self::draw_status_card(ui, &theme, network_manager, animation_time);
        ui.add_space(16.0);
        
        Self::draw_connections_card(ui, &theme, config, network_manager, loading_actions, animation_time);
        ui.add_space(16.0);
        
        Self::draw_add_connection_card(ui, &theme, config, new_vpn_name, new_vpn_config_path,
                                      new_vpn_username, new_vpn_password, new_vpn_type);
    }
    
    fn draw_status_card(ui: &mut egui::Ui, theme: &Theme, network_manager: &NetworkManager, animation_time: f32) {
        Card::show(ui, theme, "VPN Status", |ui| {
            match &network_manager.vpn_status {
                VpnStatus::Disconnected => {
                    StatusIndicator::show_with_animation(ui, theme, false, "Disconnected", false, animation_time);
                }
                VpnStatus::Connecting => {
                    StatusIndicator::show_with_animation(ui, theme, false, "Connecting...", true, animation_time);
                }
                VpnStatus::Connected(name) => {
                    StatusIndicator::show_with_animation(ui, theme, true, &format!("Connected to {}", name), false, animation_time);
                }
                VpnStatus::Error(err) => {
                    ui.label(egui::RichText::new(format!("Error: {}", err)).color(theme.error));
                }
            }
        });
    }
    
    fn draw_connections_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config, network_manager: &mut NetworkManager, loading_actions: &std::collections::HashSet<String>, animation_time: f32) {
        Card::show(ui, theme, "VPN Connections", |ui| {
            if config.vpn_configs.is_empty() {
                ui.label(egui::RichText::new("No VPN configurations found").color(theme.text_secondary));
                return;
            }
            
            let mut to_remove = None;
            
            for (index, vpn_config) in config.vpn_configs.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(&vpn_config.name);
                        let vpn_type_str = match vpn_config.vpn_type {
                            VpnType::OpenVpn => "OpenVPN",
                            VpnType::WireGuard => "WireGuard",
                        };
                        ui.label(egui::RichText::new(vpn_type_str).color(theme.text_secondary));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            to_remove = Some(index);
                        }
                        
                        let is_connected = matches!(
                            &network_manager.vpn_status,
                            VpnStatus::Connected(name) if name == &vpn_config.name
                        );
                        
                        // Show connection status for WireGuard
                        if vpn_config.vpn_type == VpnType::WireGuard {
                            let runtime = tokio::runtime::Runtime::new().unwrap();
                            let is_actually_connected = runtime.block_on(async {
                                network_manager.check_vpn_status(vpn_config).await.unwrap_or(false)
                            });
                            
                            if is_actually_connected && !is_connected {
                                network_manager.vpn_status = VpnStatus::Connected(vpn_config.name.clone());
                            } else if !is_actually_connected && is_connected {
                                network_manager.vpn_status = VpnStatus::Disconnected;
                            }
                        }
                        
                        let is_connecting = matches!(
                            &network_manager.vpn_status,
                            VpnStatus::Connecting
                        );
                        
                        let disconnect_action = format!("disconnect_{}", vpn_config.name);
                        let connect_action = format!("connect_{}", vpn_config.name);
                        
                        if is_connected {
                            let is_loading = loading_actions.contains(&disconnect_action);
                            if GlassButton::show_with_loading(ui, theme, "Disconnect", false, is_loading, animation_time).clicked() {
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                runtime.block_on(async {
                                    let _ = network_manager.disconnect_vpn(vpn_config).await;
                                });
                            }
                        } else {
                            let is_loading = loading_actions.contains(&connect_action) || is_connecting;
                            if GlassButton::show_with_loading(ui, theme, "Connect", true, is_loading, animation_time).clicked() && !is_loading {
                                let runtime = tokio::runtime::Runtime::new().unwrap();
                                runtime.block_on(async {
                                    let _ = network_manager.connect_vpn(vpn_config).await;
                                });
                            }
                        }
                    });
                });
                ui.separator();
            }
            
            if let Some(index) = to_remove {
                config.vpn_configs.remove(index);
            }
        });
    }
    
    fn draw_add_connection_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config,
                               new_vpn_name: &mut String, new_vpn_config_path: &mut String,
                               new_vpn_username: &mut String, new_vpn_password: &mut String,
                               new_vpn_type: &mut VpnType) {
        Card::show(ui, theme, "Add VPN Connection", |ui| {
            ui.label("Add new VPN connection configuration");
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("VPN Type:");
                ui.add_space(8.0);
                ui.selectable_value(new_vpn_type, VpnType::OpenVpn, "OpenVPN");
                ui.selectable_value(new_vpn_type, VpnType::WireGuard, "WireGuard");
            });
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(new_vpn_name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Config Path:");
                ui.text_edit_singleline(new_vpn_config_path);
                
                if ui.button("Browse").clicked() {
                    let file_filter = match new_vpn_type {
                        VpnType::OpenVpn => &["ovpn"],
                        VpnType::WireGuard => &["conf"],
                    };
                    
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("VPN Config", file_filter)
                        .pick_file()
                    {
                        *new_vpn_config_path = path.display().to_string();
                    }
                }
            });
            
            if *new_vpn_type == VpnType::OpenVpn {
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(new_vpn_username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(new_vpn_password).password(true));
                });
            }
            
            ui.add_space(12.0);
            
            if GlassButton::show(ui, theme, "Add Connection", true).clicked() {
                if !new_vpn_name.is_empty() && !new_vpn_config_path.is_empty() {
                    config.vpn_configs.push(VpnConfig {
                        name: new_vpn_name.clone(),
                        config_path: new_vpn_config_path.clone(),
                        username: new_vpn_username.clone(),
                        password: new_vpn_password.clone(),
                        auto_connect: false,
                        vpn_type: new_vpn_type.clone(),
                    });
                    
                    // Clear input fields
                    new_vpn_name.clear();
                    new_vpn_config_path.clear();
                    new_vpn_username.clear();
                    new_vpn_password.clear();
                    *new_vpn_type = VpnType::OpenVpn;
                }
            }
        });
    }
}