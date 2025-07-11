use eframe::egui;
use crate::config::Config;
use crate::network::{NetworkManager, VpnStatus};
use crate::ui::components::{StatusIndicator, ModernCard, Spacing, Typography};
use crate::ui::theme::{Theme, DeviceType, ActionType};

#[derive(Clone, Copy)]
enum WolAction {
    Wake,
    Ping,
}

pub struct HomePanel;

impl HomePanel {
    pub fn draw(ui: &mut egui::Ui, app: &mut crate::ui::App) {
        let theme = Theme::new();
        
        // Modern header with improved typography
        ui.vertical(|ui| {
            Typography::title(ui, &theme, "Dashboard");
            Typography::secondary(ui, &theme, "Monitor and control your network devices");
        });
        
        Spacing::lg(ui);
        
        // VPN Status Overview
        Self::draw_vpn_overview(ui, &theme, &app.config, &mut app.network_manager);
        Spacing::md(ui);
        
        // Remote Devices Grid with improved layout
        Self::draw_remote_devices(ui, &theme, app);
    }
    
    fn draw_vpn_overview(ui: &mut egui::Ui, theme: &Theme, config: &Config, network_manager: &mut NetworkManager) {
        ModernCard::show(ui, theme, "VPN Status", |ui| {
            ui.horizontal(|ui| {
                // VPN Status with modern indicator
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
                    Typography::small(ui, theme, &format!("{} VPN configs", config.vpn_configs.len()));
                });
            });
            
            // VPN Connection Controls
            if !config.vpn_configs.is_empty() {
                Spacing::md(ui);
                ui.separator();
                Spacing::sm(ui);
                
                ui.horizontal(|ui| {
                    Typography::body(ui, theme, "Quick Connect:");
                    Spacing::sm(ui);
                    
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
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(80.0, 32.0))).clicked() {
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
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(80.0, 32.0))).clicked() {
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
    
    fn draw_remote_devices(ui: &mut egui::Ui, theme: &Theme, app: &mut crate::ui::App) {
        ModernCard::show(ui, theme, "Remote Devices", |ui| {
            if app.config.rdp_configs.is_empty() && app.config.wol_devices.is_empty() {
                ui.vertical_centered(|ui| {
                    Spacing::lg(ui);
                    ui.label(egui::RichText::new("🖥️").size(32.0).color(theme.text_disabled));
                    Spacing::md(ui);
                    Typography::heading(ui, theme, "No devices configured");
                    Typography::secondary(ui, theme, "Add RDP or WoL devices to get started");
                    Spacing::lg(ui);
                });
                return;
            }
            
            // Calculate grid layout
            let available_width = ui.available_width();
            let card_width = 220.0;
            let spacing = 12.0;
            let cards_per_row = ((available_width + spacing) / (card_width + spacing)).floor() as usize;
            let cards_per_row = cards_per_row.max(1).min(4); // Max 4 cards per row for better visibility
            
            let total_devices = app.config.rdp_configs.len() + app.config.wol_devices.len();
            
            // Collect device operation actions separately to avoid borrow conflicts
            let mut pending_operations = Vec::new();
            
            // Show devices in a responsive grid
            egui::Grid::new("device_grid")
                .num_columns(cards_per_row)
                .spacing(egui::vec2(spacing, spacing))
                .show(ui, |ui| {
                    let mut device_count = 0;
                    
                    // RDP Devices
                    for rdp_config in &app.config.rdp_configs {
                        let connect_state = app.get_device_operation_state(&rdp_config.name, "connect");
                        
                        if Self::draw_rdp_device_card_with_state(ui, theme, rdp_config, connect_state) {
                            // Queue async RDP connection
                            pending_operations.push(crate::ui::DeviceOperationType::RdpConnect(rdp_config.clone()));
                        }
                        
                        device_count += 1;
                        if device_count % cards_per_row == 0 {
                            ui.end_row();
                        }
                    }
                    
                    // WOL Devices
                    for wol_device in &app.config.wol_devices {
                        let is_online = app.network_manager.wol_devices
                            .iter()
                            .find(|d| d.device.name == wol_device.name)
                            .map(|d| d.is_online)
                            .unwrap_or(false);
                        
                        let wake_state = app.get_device_operation_state(&wol_device.name, "wake");
                        let ping_state = app.get_device_operation_state(&wol_device.name, "ping");
                        
                        let action = Self::draw_wol_device_card_with_state(ui, theme, wol_device, is_online, wake_state, ping_state);
                        
                        match action {
                            Some(WolAction::Wake) => {
                                // Queue async Wake on LAN
                                pending_operations.push(crate::ui::DeviceOperationType::Wake(wol_device.clone()));
                            }
                            Some(WolAction::Ping) => {
                                // Queue async Ping
                                pending_operations.push(crate::ui::DeviceOperationType::Ping(wol_device.clone()));
                            }
                            None => {}
                        }
                        
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
            
            // Process pending operations after all borrows are done
            for operation in pending_operations {
                match &operation {
                    crate::ui::DeviceOperationType::RdpConnect(rdp_config) => {
                        app.start_device_operation(
                            rdp_config.name.clone(),
                            "connect".to_string(),
                            operation
                        );
                    }
                    crate::ui::DeviceOperationType::Wake(wol_device) => {
                        app.start_device_operation(
                            wol_device.name.clone(),
                            "wake".to_string(),
                            operation
                        );
                    }
                    crate::ui::DeviceOperationType::Ping(wol_device) => {
                        app.start_device_operation(
                            wol_device.name.clone(),
                            "ping".to_string(),
                            operation
                        );
                    }
                }
            }
            
            // Device summary
            if total_devices > 0 {
                Spacing::md(ui);
                ui.separator();
                Spacing::sm(ui);
                
                ui.horizontal(|ui| {
                    Typography::small(ui, theme, &format!("Total: {} devices", total_devices));
                    if !app.config.rdp_configs.is_empty() {
                        ui.label(egui::RichText::new("•").color(theme.text_disabled));
                        Typography::small(ui, theme, &format!("{} RDP", app.config.rdp_configs.len()));
                    }
                    if !app.config.wol_devices.is_empty() {
                        ui.label(egui::RichText::new("•").color(theme.text_disabled));
                        let online_count = app.network_manager.wol_devices.iter().filter(|d| d.is_online).count();
                        Typography::small(ui, theme, &format!("{} WoL ({} online)", app.config.wol_devices.len(), online_count));
                    }
                });
            }
        });
    }
    
    fn draw_rdp_device_card_with_state(ui: &mut egui::Ui, theme: &Theme, rdp_config: &crate::config::RdpConfig, operation_state: &crate::ui::DeviceOperationState) -> bool {
        let response = ui.allocate_response(egui::vec2(200.0, 70.0), egui::Sense::hover());
        let is_hovered = response.hovered();
        
        let (bg_color, border_color, border_width) = theme.get_card_colors(is_hovered, false);
        
        let mut clicked = false;
        
        egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(border_width, border_color))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .shadow(theme.get_shadow(is_hovered))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Device icon with type-specific background
                    let icon_bg = theme.primary.gamma_multiply(0.15);
                    egui::Frame::none()
                        .fill(icon_bg)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("🖥️")
                                    .size(20.0)
                                    .color(theme.get_device_icon_color(DeviceType::RDP, true))
                            );
                        });
                    
                    ui.add_space(12.0);
                    
                    // Device information
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&rdp_config.name)
                                .strong()
                                .size(14.0)
                                .color(theme.text_primary)
                        );
                        ui.label(
                            egui::RichText::new(format!("{}:{}", rdp_config.host, rdp_config.port))
                                .size(11.0)
                                .color(theme.text_secondary)
                        );
                        
                        // Connection type badge
                        egui::Frame::none()
                            .fill(theme.primary.gamma_multiply(0.2))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("RDP")
                                        .size(9.0)
                                        .color(theme.primary)
                                );
                            });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (button_text, button_color, button_enabled) = match operation_state {
                            crate::ui::DeviceOperationState::Idle => ("Connect", theme.get_action_button_color(ActionType::Primary), true),
                            crate::ui::DeviceOperationState::Loading => ("Connecting...", theme.loading, false),
                            crate::ui::DeviceOperationState::Success(_) => ("Connected ✓", theme.success, true),
                            crate::ui::DeviceOperationState::Error(_) => ("Failed ✗", theme.error, true),
                        };
                        
                        if ui.add_enabled(button_enabled,
                            egui::Button::new(button_text)
                                .fill(button_color)
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(80.0, 30.0))
                        ).clicked() && button_enabled {
                            clicked = true;
                        }
                        
                        // Show operation feedback as tooltip
                        if let crate::ui::DeviceOperationState::Success(msg) | crate::ui::DeviceOperationState::Error(msg) = operation_state {
                            if ui.rect_contains_pointer(ui.max_rect()) {
                                egui::show_tooltip_text(ui.ctx(), egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tooltip")), egui::Id::new(format!("rdp_tooltip_{}", rdp_config.name)), msg);
                            }
                        }
                    });
                });
            });
        
        clicked
    }
    
    fn draw_wol_device_card_with_state(
        ui: &mut egui::Ui, 
        theme: &Theme, 
        wol_device: &crate::config::WolDevice, 
        is_online: bool,
        wake_state: &crate::ui::DeviceOperationState,
        ping_state: &crate::ui::DeviceOperationState
    ) -> Option<WolAction> {
        let response = ui.allocate_response(egui::vec2(200.0, 70.0), egui::Sense::hover());
        let is_hovered = response.hovered();
        
        let (bg_color, border_color, border_width) = theme.get_card_colors(is_hovered, is_online);
        
        let mut action = None;
        
        egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(border_width, border_color))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .shadow(theme.get_shadow(is_hovered))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Device icon with status-specific background
                    let icon_bg = if is_online {
                        theme.success.gamma_multiply(0.15)
                    } else {
                        theme.text_disabled.gamma_multiply(0.15)
                    };
                    
                    egui::Frame::none()
                        .fill(icon_bg)
                        .rounding(egui::Rounding::same(6.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("💻")
                                    .size(20.0)
                                    .color(theme.get_device_icon_color(DeviceType::WOL, is_online))
                            );
                        });
                    
                    ui.add_space(12.0);
                    
                    // Device information
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&wol_device.name)
                                .strong()
                                .size(14.0)
                                .color(theme.text_primary)
                        );
                        ui.label(
                            egui::RichText::new(&wol_device.ip_address)
                                .size(11.0)
                                .color(theme.text_secondary)
                        );
                        
                        // Status badge
                        let status_bg = if is_online {
                            theme.success.gamma_multiply(0.2)
                        } else {
                            theme.text_disabled.gamma_multiply(0.2)
                        };
                        let status_color = theme.get_device_status_color(is_online);
                        let status_text = if is_online { "Online" } else { "Offline" };
                        
                        egui::Frame::none()
                            .fill(status_bg)
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("●")
                                            .size(8.0)
                                            .color(status_color)
                                    );
                                    ui.label(
                                        egui::RichText::new(status_text)
                                            .size(9.0)
                                            .color(status_color)
                                    );
                                });
                            });
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            // Wake button with state
                            let (wake_text, wake_color, wake_enabled) = match wake_state {
                                crate::ui::DeviceOperationState::Idle => ("Wake", theme.get_action_button_color(ActionType::Success), true),
                                crate::ui::DeviceOperationState::Loading => ("Waking...", theme.loading, false),
                                crate::ui::DeviceOperationState::Success(_) => ("Sent ✓", theme.success, true),
                                crate::ui::DeviceOperationState::Error(_) => ("Failed ✗", theme.error, true),
                            };
                            
                            if ui.add_enabled(wake_enabled,
                                egui::Button::new(wake_text)
                                    .fill(wake_color)
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(60.0, 28.0))
                            ).clicked() && wake_enabled {
                                action = Some(WolAction::Wake);
                            }
                            
                            // Ping button with state
                            let (ping_text, ping_color, ping_enabled) = match ping_state {
                                crate::ui::DeviceOperationState::Idle => ("Ping", theme.get_action_button_color(ActionType::Secondary), true),
                                crate::ui::DeviceOperationState::Loading => ("Pinging...", theme.loading, false),
                                crate::ui::DeviceOperationState::Success(_) => ("Ping ✓", theme.success, true),
                                crate::ui::DeviceOperationState::Error(_) => ("Failed ✗", theme.error, true),
                            };
                            
                            if ui.add_enabled(ping_enabled,
                                egui::Button::new(ping_text)
                                    .fill(ping_color)
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(60.0, 28.0))
                            ).clicked() && ping_enabled {
                                action = Some(WolAction::Ping);
                            }
                        });
                        
                        // Show operation feedback as tooltips
                        if let crate::ui::DeviceOperationState::Success(msg) | crate::ui::DeviceOperationState::Error(msg) = wake_state {
                            if ui.rect_contains_pointer(ui.max_rect()) {
                                egui::show_tooltip_text(ui.ctx(), egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tooltip")), egui::Id::new(format!("wake_tooltip_{}", wol_device.name)), msg);
                            }
                        }
                        if let crate::ui::DeviceOperationState::Success(msg) | crate::ui::DeviceOperationState::Error(msg) = ping_state {
                            if ui.rect_contains_pointer(ui.max_rect()) {
                                egui::show_tooltip_text(ui.ctx(), egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tooltip")), egui::Id::new(format!("ping_tooltip_{}", wol_device.name)), msg);
                            }
                        }
                    });
                });
            });
        
        action
    }
}