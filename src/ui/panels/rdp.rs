use eframe::egui;
use crate::config::{Config, RdpConfig};
use crate::network::NetworkManager;
use crate::ui::components::{Card, GlassButton, InputField};
use crate::ui::theme::Theme;

pub struct RdpPanel;

impl RdpPanel {
    pub fn draw(ui: &mut egui::Ui, config: &mut Config, _network_manager: &mut NetworkManager,
                new_rdp_name: &mut String, new_rdp_host: &mut String, new_rdp_port: &mut String,
                new_rdp_username: &mut String, new_rdp_password: &mut String, new_rdp_domain: &mut String) {
        let theme = Theme::new();
        
        ui.heading("RDP Connections");
        ui.add_space(20.0);
        
        Self::draw_connections_card(ui, &theme, config);
        ui.add_space(16.0);
        
        Self::draw_add_connection_card(ui, &theme, config, new_rdp_name, new_rdp_host, new_rdp_port,
                                      new_rdp_username, new_rdp_password, new_rdp_domain);
    }
    
    fn draw_connections_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config) {
        Card::show(ui, theme, "RDP Connections", |ui| {
            if config.rdp_configs.is_empty() {
                ui.label(egui::RichText::new("No RDP configurations found").color(theme.text_secondary));
                return;
            }
            
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
                                let _ = crate::network::rdp::connect(rdp_config).await;
                            });
                        }
                    });
                });
                ui.separator();
            }
            
            if let Some(index) = to_remove {
                config.rdp_configs.remove(index);
            }
        });
    }
    
    fn draw_add_connection_card(ui: &mut egui::Ui, theme: &Theme, config: &mut Config,
                               new_rdp_name: &mut String, new_rdp_host: &mut String, new_rdp_port: &mut String,
                               new_rdp_username: &mut String, new_rdp_password: &mut String, new_rdp_domain: &mut String) {
        Card::show(ui, theme, "Add RDP Connection", |ui| {
            ui.label("Add new RDP connection configuration");
            ui.add_space(8.0);
            
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
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
                });
                
                columns[1].vertical(|ui| {
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
                });
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
}