use eframe::egui::{self, FontFamily, FontId, Rounding, Stroke, TextStyle, ColorImage, TextureHandle};
use crate::config::{Config, VpnType};
use crate::network::NetworkManager;
use crate::system::{SystemInfo, installer::PackageInstaller, updater::{AppUpdater, UpdateInfo}};

#[derive(Debug, Clone)]
pub enum DeviceOperationState {
    Idle,
    Loading,
    Success(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DeviceOperationResult {
    pub device_name: String,
    pub operation: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum DeviceOperationType {
    Wake(crate::config::WolDevice),
    Ping(crate::config::WolDevice),
    RdpConnect(crate::config::RdpConfig),
}

pub mod theme;
pub mod components;
pub mod panels;

use theme::Theme;
use panels::{HomePanel, VpnPanel, RemotePanel, SettingsPanel};
use components::{ModernButton, Spacing, Typography};

pub struct App {
    config: Config,
    network_manager: NetworkManager,
    theme: Theme,
    current_panel: Panel,
    show_settings: bool,
    error_message: Option<String>,
    system_info: SystemInfo,
    package_installer: PackageInstaller,
    app_updater: AppUpdater,
    update_info: Option<UpdateInfo>,
    logo_texture: Option<TextureHandle>,
    // Input field state
    new_vpn_name: String,
    new_vpn_config_path: String,
    new_vpn_username: String,
    new_vpn_password: String,
    new_vpn_type: VpnType,
    new_rdp_name: String,
    new_rdp_host: String,
    new_rdp_port: String,
    new_rdp_username: String,
    new_rdp_password: String,
    new_rdp_domain: String,
    new_wol_name: String,
    new_wol_mac: String,
    new_wol_ip: String,
    new_wol_port: String,
    // Feedback states
    is_connecting: bool,
    connection_feedback: Option<String>,
    loading_actions: std::collections::HashSet<String>,
    animation_time: f32,
    checking_updates: bool,
    installing_update: bool,
    update_progress: String,
    update_notification: Option<String>,
    last_update_check: std::time::Instant,
    update_check_receiver: Option<std::sync::mpsc::Receiver<Result<crate::system::updater::UpdateInfo, String>>>,
    update_check_timeout: std::time::Instant,
    // Device operation feedback
    device_operations: std::collections::HashMap<String, DeviceOperationState>,
    device_feedback_receiver: Option<std::sync::mpsc::Receiver<DeviceOperationResult>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Home,
    Vpn,
    Remote,
    Settings,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, String> {
        use log::{info, warn};
        
        info!("Detecting system information...");
        let system_info = SystemInfo::detect().unwrap_or_else(|e| {
            warn!("Failed to detect system info: {}", e);
            SystemInfo {
                distribution: "Unknown".to_string(),
                package_manager: crate::system::PackageManager::Unknown,
                dependencies: Vec::new(),
            }
        });
        
        info!("Detected system: {}", system_info.distribution);
        let package_installer = PackageInstaller::new(&system_info);
        let app_updater = AppUpdater::new("EmmanouelKontos", "vpn-aio-rust", env!("CARGO_PKG_VERSION"));
        
        info!("Loading configuration...");
        let config = Config::load().unwrap_or_else(|e| {
            warn!("Failed to load config: {}, using default", e);
            Config::default()
        });
        
        info!("Initializing network manager...");
        let mut network_manager = NetworkManager::new();
        
        // Initialize VPN status and WoL devices based on current system state
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let _ = runtime.block_on(async {
            network_manager.initialize(&config.vpn_configs, &config.wol_devices).await
        });
        
        let mut app = Self {
            config,
            network_manager,
            theme: Theme::new(),
            current_panel: Panel::Home,
            show_settings: false,
            error_message: None,
            system_info,
            package_installer,
            app_updater,
            update_info: None,
            logo_texture: None,
            // Initialize input fields
            new_vpn_name: String::new(),
            new_vpn_config_path: String::new(),
            new_vpn_username: String::new(),
            new_vpn_password: String::new(),
            new_vpn_type: VpnType::OpenVpn,
            new_rdp_name: String::new(),
            new_rdp_host: String::new(),
            new_rdp_port: String::from("3389"),
            new_rdp_username: String::new(),
            new_rdp_password: String::new(),
            new_rdp_domain: String::new(),
            new_wol_name: String::new(),
            new_wol_mac: String::new(),
            new_wol_ip: String::new(),
            new_wol_port: String::from("9"),
            // Initialize feedback states
            is_connecting: false,
            connection_feedback: None,
            loading_actions: std::collections::HashSet::new(),
            animation_time: 0.0,
            checking_updates: false,
            installing_update: false,
            update_progress: String::new(),
            update_notification: None,
            last_update_check: std::time::Instant::now(),
            update_check_receiver: None,
            update_check_timeout: std::time::Instant::now(),
            // Initialize device operation states
            device_operations: std::collections::HashMap::new(),
            device_feedback_receiver: None,
        };

        // Auto-connect to VPN if enabled
        if app.config.auto_connect_vpn && !app.config.vpn_configs.is_empty() {
            info!("Auto-connecting to VPN...");
            if let Some(vpn_config) = app.config.vpn_configs.first() {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let _ = runtime.block_on(async {
                    app.network_manager.connect_vpn(vpn_config).await
                });
            }
        }

        info!("Setting up fonts and styles...");
        app.setup_fonts(cc);
        app.setup_style(cc);
        
        info!("Loading logo texture...");
        app.load_logo_texture(cc);
        
        info!("Checking for updates...");
        app.schedule_update_check();
        
        info!("Application initialized successfully");
        Ok(app)
    }

    fn setup_fonts(&self, cc: &eframe::CreationContext<'_>) {
        let fonts = egui::FontDefinitions::default();
        
        // fonts.font_data.insert(
        //     "Inter".to_owned(),
        //     egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter-Regular.ttf"))
        //         .unwrap_or_else(|_| egui::FontData::default()),
        // );

        // fonts.families.entry(FontFamily::Proportional)
        //     .or_default()
        //     .insert(0, "Inter".to_owned());

        cc.egui_ctx.set_fonts(fonts);
    }

    fn setup_style(&self, cc: &eframe::CreationContext<'_>) {
        let mut style = (*cc.egui_ctx.style()).clone();
        
        style.visuals.dark_mode = self.config.dark_mode;
        style.visuals.window_fill = self.theme.background;
        style.visuals.panel_fill = self.theme.surface;
        style.visuals.window_stroke = Stroke::new(1.0, self.theme.border);
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.menu_rounding = Rounding::same(8.0);
        style.visuals.button_frame = true;
        style.visuals.collapsing_header_frame = true;
        
        // Better text input styling
        style.visuals.extreme_bg_color = self.theme.surface_variant;
        style.visuals.code_bg_color = self.theme.surface_variant;
        style.visuals.text_cursor.stroke.color = self.theme.primary;
        style.visuals.selection.bg_fill = self.theme.selection_bg;
        style.visuals.selection.stroke.color = self.theme.selection_text;
        
        // Improved text selection visibility
        style.visuals.widgets.inactive.weak_bg_fill = self.theme.surface_variant;
        style.visuals.widgets.hovered.weak_bg_fill = self.theme.hover_bg;
        style.visuals.widgets.active.weak_bg_fill = self.theme.selection_bg;
        
        // Enhanced text input styling
        style.visuals.widgets.inactive.bg_fill = self.theme.surface_variant;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.theme.border);
        style.visuals.widgets.inactive.fg_stroke.color = self.theme.text_primary;
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.expansion = 4.0;
        
        style.visuals.widgets.hovered.bg_fill = self.theme.surface_variant;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, self.theme.primary);
        style.visuals.widgets.hovered.fg_stroke.color = self.theme.text_primary;
        style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
        style.visuals.widgets.hovered.expansion = 4.0;
        
        style.visuals.widgets.active.bg_fill = self.theme.surface_variant;
        style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, self.theme.primary);
        style.visuals.widgets.active.fg_stroke.color = self.theme.text_primary;
        style.visuals.widgets.active.rounding = Rounding::same(8.0);
        style.visuals.widgets.active.expansion = 4.0;
        
        // Better button styling
        style.visuals.widgets.noninteractive.bg_fill = self.theme.surface_variant;
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.theme.border);
        style.visuals.widgets.noninteractive.fg_stroke.color = self.theme.text_primary;
        style.visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
        
        // Improved spacing
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.menu_margin = egui::Margin::same(8.0);
        style.spacing.indent = 20.0;
        style.spacing.combo_width = 100.0;
        style.spacing.text_edit_width = 200.0;
        
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );

        cc.egui_ctx.set_style(style);
    }
    
    fn load_logo_texture(&mut self, cc: &eframe::CreationContext<'_>) {
        let logo_bytes = include_bytes!("../../assets/vpn-aio.png");
        
        match image::load_from_memory(logo_bytes) {
            Ok(dynamic_image) => {
                let image_buffer = dynamic_image.to_rgba8();
                let size = [image_buffer.width() as usize, image_buffer.height() as usize];
                let pixels = image_buffer.as_flat_samples();
                
                let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                
                self.logo_texture = Some(cc.egui_ctx.load_texture(
                    "logo",
                    color_image,
                    egui::TextureOptions::LINEAR
                ));
                
                log::info!("Logo texture loaded successfully");
            }
            Err(e) => {
                log::warn!("Failed to load logo texture: {}", e);
            }
        }
    }
    
    fn schedule_update_check(&mut self) {
        if self.checking_updates || self.update_info.is_some() {
            return;
        }
        
        self.checking_updates = true;
        self.update_check_timeout = std::time::Instant::now();
        let app_updater = self.app_updater.clone();
        
        // Use a channel to communicate results back
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match app_updater.check_for_updates().await {
                    Ok(info) => {
                        let _ = tx.send(Ok(info));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                    }
                }
            });
        });
        
        // Store the receiver for polling in the main thread
        self.update_check_receiver = Some(rx);
    }
    
    fn poll_update_check(&mut self) {
        if let Some(receiver) = &self.update_check_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    // Update check completed
                    self.checking_updates = false;
                    self.update_check_receiver = None;
                    
                    match result {
                        Ok(info) => {
                            if info.update_available {
                                log::info!("Update available: {} -> {}", info.current_version, info.latest_version);
                                self.update_info = Some(info.clone());
                                self.update_notification = Some(format!("Update available: v{}", info.latest_version));
                            } else {
                                log::info!("No updates available, current version {} is latest", info.current_version);
                                self.update_info = Some(info);
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to check for updates: {}", e);
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still waiting for result, check for timeout
                    if self.update_check_timeout.elapsed().as_secs() > 30 {
                        log::warn!("Update check timed out after 30 seconds");
                        self.checking_updates = false;
                        self.update_check_receiver = None;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected, stop checking
                    log::warn!("Update check channel disconnected");
                    self.checking_updates = false;
                    self.update_check_receiver = None;
                }
            }
        }
    }
    
    fn start_device_operation(&mut self, device_name: String, operation: String, operation_type: DeviceOperationType) {
        // Set device state to loading
        self.device_operations.insert(
            format!("{}_{}", device_name, operation), 
            DeviceOperationState::Loading
        );
        
        // Use a channel to communicate results back
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        
        match operation_type {
            DeviceOperationType::Wake(wol_device) => {
                let mut network_manager = self.network_manager.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match network_manager.wake_device(&wol_device).await {
                            Ok(_) => {
                                let _ = tx.send(DeviceOperationResult {
                                    device_name: device_name.clone(),
                                    operation: operation.clone(),
                                    success: true,
                                    message: format!("Wake-on-LAN packet sent to {}", device_name),
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(DeviceOperationResult {
                                    device_name: device_name.clone(),
                                    operation: operation.clone(),
                                    success: false,
                                    message: format!("Failed to wake {}: {}", device_name, e),
                                });
                            }
                        }
                    });
                });
            }
            DeviceOperationType::Ping(wol_device) => {
                let mut network_manager = self.network_manager.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let is_online = network_manager.check_device_status(&wol_device).await;
                        let _ = tx.send(DeviceOperationResult {
                            device_name: device_name.clone(),
                            operation: operation.clone(),
                            success: true,
                            message: format!("{} is {}", device_name, if is_online { "online" } else { "offline" }),
                        });
                    });
                });
            }
            DeviceOperationType::RdpConnect(rdp_config) => {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match crate::network::rdp::connect(&rdp_config).await {
                            Ok(_) => {
                                let _ = tx.send(DeviceOperationResult {
                                    device_name: device_name.clone(),
                                    operation: operation.clone(),
                                    success: true,
                                    message: format!("RDP connection initiated to {}", device_name),
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(DeviceOperationResult {
                                    device_name: device_name.clone(),
                                    operation: operation.clone(),
                                    success: false,
                                    message: format!("Failed to connect to {}: {}", device_name, e),
                                });
                            }
                        }
                    });
                });
            }
        }
        
        // Store the receiver for polling in the main thread
        self.device_feedback_receiver = Some(rx);
    }
    
    fn poll_device_operations(&mut self) {
        if let Some(receiver) = &self.device_feedback_receiver {
            match receiver.try_recv() {
                Ok(result) => {
                    // Operation completed
                    let key = format!("{}_{}", result.device_name, result.operation);
                    
                    if result.success {
                        self.device_operations.insert(key, DeviceOperationState::Success(result.message.clone()));
                        self.connection_feedback = Some(result.message);
                    } else {
                        self.device_operations.insert(key, DeviceOperationState::Error(result.message.clone()));
                        self.connection_feedback = Some(result.message);
                    }
                    
                    // Reset the animation timer for feedback display
                    self.animation_time = 0.0;
                    
                    // Keep the receiver for potential future operations
                    // (Don't set to None like with update check)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still waiting for result, nothing to do
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected, reset
                    self.device_feedback_receiver = None;
                }
            }
        }
    }
    
    fn get_device_operation_state(&self, device_name: &str, operation: &str) -> &DeviceOperationState {
        let key = format!("{}_{}", device_name, operation);
        self.device_operations.get(&key).unwrap_or(&DeviceOperationState::Idle)
    }

    fn draw_sidebar(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            Spacing::md(ui);
            
            // Display logo if available
            if let Some(logo_texture) = &self.logo_texture {
                ui.horizontal(|ui| {
                    Spacing::sm(ui);
                    ui.add(egui::Image::new(logo_texture)
                        .max_width(40.0)
                        .max_height(40.0)
                        .rounding(egui::Rounding::same(6.0)));
                    Spacing::sm(ui);
                    ui.vertical(|ui| {
                        Typography::heading(ui, &self.theme, "VPN Manager");
                        Typography::small(ui, &self.theme, "All-in-One");
                    });
                });
            } else {
                ui.horizontal(|ui| {
                    Spacing::sm(ui);
                    ui.vertical(|ui| {
                        Typography::heading(ui, &self.theme, "VPN Manager");
                        Typography::small(ui, &self.theme, "All-in-One");
                    });
                });
            }
            Spacing::lg(ui);

            let button_size = egui::vec2(180.0, 28.0);
            
            // Navigation buttons with consistent styling
            let home_selected = self.current_panel == Panel::Home;
            if self.draw_nav_button(ui, "🏠 Home", button_size, home_selected) {
                self.current_panel = Panel::Home;
            }
            Spacing::xs(ui);
            
            let vpn_selected = self.current_panel == Panel::Vpn;
            if self.draw_nav_button(ui, "🔒 VPN", button_size, vpn_selected) {
                self.current_panel = Panel::Vpn;
            }
            Spacing::xs(ui);
            
            let remote_selected = self.current_panel == Panel::Remote;
            if self.draw_nav_button(ui, "🖥️ Remote", button_size, remote_selected) {
                self.current_panel = Panel::Remote;
            }
            
            Spacing::sm(ui);
            
            // Show update indicator on Settings button if update is available
            let settings_text = if let Some(update) = &self.update_info {
                if update.update_available {
                    "⚙️ Settings 🔴"
                } else {
                    "⚙️ Settings"
                }
            } else if self.checking_updates {
                "⚙️ Settings ⏳"
            } else {
                "⚙️ Settings"
            };
            
            let settings_selected = self.current_panel == Panel::Settings;
            if self.draw_nav_button(ui, settings_text, button_size, settings_selected) {
                self.current_panel = Panel::Settings;
            }
        });
    }
    
    fn draw_nav_button(&self, ui: &mut egui::Ui, text: &str, size: egui::Vec2, is_selected: bool) -> bool {
        let button_color = if is_selected {
            self.theme.primary
        } else {
            self.theme.surface_variant
        };
        
        let text_color = if is_selected {
            egui::Color32::WHITE
        } else {
            self.theme.text_primary
        };
        
        let button = egui::Button::new(
            egui::RichText::new(text).color(text_color).size(12.0)
        )
        .fill(button_color)
        .stroke(egui::Stroke::new(if is_selected { 0.0 } else { 1.0 }, self.theme.border))
        .rounding(egui::Rounding::same(4.0));
        
        ui.add_sized(size, button).clicked()
    }

    fn draw_main_content(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        match self.current_panel {
            Panel::Home => {
                HomePanel::draw(ui, self);
            }
            Panel::Vpn => {
                VpnPanel::draw(ui, &mut self.config, &mut self.network_manager, 
                    &mut self.new_vpn_name, &mut self.new_vpn_config_path, 
                    &mut self.new_vpn_username, &mut self.new_vpn_password, 
                    &mut self.new_vpn_type, &self.loading_actions, self.animation_time);
            }
            Panel::Remote => {
                RemotePanel::draw(ui, &mut self.config, &mut self.network_manager,
                    &mut self.new_rdp_name, &mut self.new_rdp_host, &mut self.new_rdp_port,
                    &mut self.new_rdp_username, &mut self.new_rdp_password, &mut self.new_rdp_domain,
                    &mut self.new_wol_name, &mut self.new_wol_mac, 
                    &mut self.new_wol_ip, &mut self.new_wol_port);
            }
            Panel::Settings => {
                SettingsPanel::draw(ui, &mut self.config, &mut self.system_info, &self.package_installer, &self.app_updater, &mut self.update_info, &mut self.checking_updates, &mut self.installing_update, &mut self.update_progress);
            }
        }
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.error_message = Some(format!("Failed to save config: {}", e));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Update animation time
        self.animation_time += 0.016; // ~60 FPS
        
        // Clear feedback after 3 seconds
        if let Some(_) = &self.connection_feedback {
            if self.animation_time > 3.0 {
                self.connection_feedback = None;
                self.animation_time = 0.0;
            }
        }
        
        // Clear update notifications after 10 seconds
        if let Some(_) = &self.update_notification {
            if self.last_update_check.elapsed().as_secs() > 10 {
                self.update_notification = None;
            }
        }
        
        // Poll update check results
        self.poll_update_check();
        
        // Poll device operation results
        self.poll_device_operations();
        
        // Check for updates periodically (every 24 hours)
        if self.last_update_check.elapsed().as_secs() > 86400 && !self.checking_updates {
            self.schedule_update_check();
            self.last_update_check = std::time::Instant::now();
        }

        // Refresh VPN status periodically (every 10 seconds)
        if self.animation_time.rem_euclid(10.0) < 0.1 && !self.config.vpn_configs.is_empty() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _ = runtime.block_on(async {
                self.network_manager.refresh_vpn_status(&self.config.vpn_configs).await
            });
        }
        
        // Sync WoL devices with config changes
        self.network_manager.sync_wol_devices(&self.config.wol_devices);
        
        // Quick update device statuses more frequently (every 10 seconds)
        if self.animation_time.rem_euclid(10.0) < 0.1 && !self.config.wol_devices.is_empty() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _ = runtime.block_on(async {
                self.network_manager.quick_update_device_statuses().await
            });
        }
        
        // Full device status update less frequently (every 60 seconds)
        if self.animation_time.rem_euclid(60.0) < 0.1 && !self.config.wol_devices.is_empty() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let _ = runtime.block_on(async {
                self.network_manager.update_device_statuses().await
            });
        }

        // Removed automatic device status updates to prevent CMD spawning issues
        // Status updates will be manual or triggered by user actions only

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(200.0)
            .max_width(200.0)
            .show(ctx, |ui| {
                ui.style_mut().visuals.panel_fill = self.theme.surface;
                self.draw_sidebar(ctx, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.panel_fill = self.theme.background;
            self.draw_main_content(ctx, ui);
        });

        if let Some(error) = &self.error_message.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        self.error_message = None;
                    }
                });
        }

        // Show feedback notifications
        if let Some(feedback) = &self.connection_feedback {
            egui::Window::new("Status")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("ℹ️");
                        ui.label(feedback);
                    });
                });
        }
        
        // Show update notifications
        if let Some(update_msg) = self.update_notification.clone() {
            let mut should_close = false;
            let mut should_view = false;
            
            egui::Window::new("Update Available")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🎉");
                        ui.label(&update_msg);
                        if ui.small_button("View").clicked() {
                            should_view = true;
                            should_close = true;
                        }
                        if ui.small_button("Dismiss").clicked() {
                            should_close = true;
                        }
                    });
                });
            
            if should_view {
                self.current_panel = Panel::Settings;
            }
            if should_close {
                self.update_notification = None;
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_config();
    }
}