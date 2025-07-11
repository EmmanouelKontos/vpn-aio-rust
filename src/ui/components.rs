use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use crate::ui::theme::{Theme, DeviceType, ActionType};

pub struct GlassPanel;

impl GlassPanel {
    pub fn show<R>(
        ui: &mut egui::Ui,
        theme: &Theme,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let desired_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        
        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(
                rect,
                Rounding::same(12.0),
                theme.surface,
            );
            
            ui.painter().rect_stroke(
                rect,
                Rounding::same(12.0),
                Stroke::new(1.0, theme.border),
            );
        }
        
        let inner_rect = rect.shrink(16.0);
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner_rect).layout(*ui.layout()));
        let inner_response = add_contents(&mut child_ui);
        
        egui::InnerResponse::new(inner_response, response)
    }
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, is_connected: bool, label: &str) {
        Self::show_with_animation(ui, theme, is_connected, label, false, 0.0);
    }
    
    pub fn show_with_animation(ui: &mut egui::Ui, theme: &Theme, is_connected: bool, label: &str, is_connecting: bool, animation_time: f32) {
        ui.horizontal(|ui| {
            let color = if is_connecting {
                // Pulsing animation for connecting state
                let pulse = (animation_time * 6.0).sin() * 0.3 + 0.7;
                Color32::from_rgba_premultiplied(
                    (theme.warning.r() as f32 * pulse) as u8,
                    (theme.warning.g() as f32 * pulse) as u8,
                    (theme.warning.b() as f32 * pulse) as u8,
                    255,
                )
            } else {
                theme.get_status_color(is_connected)
            };
            
            let circle_size = 12.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(circle_size), egui::Sense::hover());
            
            ui.painter().circle_filled(
                rect.center(),
                circle_size / 2.0,
                color,
            );
            
            // Add outer ring for connecting animation
            if is_connecting {
                let ring_alpha = ((animation_time * 4.0).sin() * 0.5 + 0.5) * 100.0;
                let ring_color = Color32::from_rgba_premultiplied(
                    theme.warning.r(),
                    theme.warning.g(),
                    theme.warning.b(),
                    ring_alpha as u8,
                );
                
                ui.painter().circle_stroke(
                    rect.center(),
                    circle_size / 2.0 + 2.0,
                    Stroke::new(2.0, ring_color),
                );
            }
            
            ui.label(label);
        });
    }
}

pub struct GlassButton;

impl GlassButton {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool) -> egui::Response {
        Self::show_with_loading(ui, theme, text, is_primary, false, 0.0)
    }
    
    pub fn show_with_loading(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool, is_loading: bool, animation_time: f32) -> egui::Response {
        let button_color = if is_loading {
            theme.loading
        } else {
            theme.get_button_color(is_primary)
        };
        
        let text_color = if is_primary || is_loading { 
            Color32::WHITE
        } else { 
            theme.text_primary 
        };
        
        let display_text = if is_loading {
            // Simple loading animation with dots
            let dots = match (animation_time * 3.0) as i32 % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                _ => "...",
            };
            format!("Loading{}", dots)
        } else {
            text.to_string()
        };
        
        let button = egui::Button::new(
            egui::RichText::new(display_text).color(text_color).size(13.0)
        )
        .fill(button_color)
        .stroke(Stroke::new(if is_primary { 0.0 } else { 1.0 }, theme.border))
        .rounding(Rounding::same(8.0));
        
        let response = ui.add_sized([120.0, 35.0], button);
        
        // Add subtle glow effect for loading
        if is_loading {
            let glow_alpha = ((animation_time * 4.0).sin() * 0.3 + 0.7) * 60.0;
            let glow_color = Color32::from_rgba_premultiplied(
                theme.loading.r(),
                theme.loading.g(), 
                theme.loading.b(),
                glow_alpha as u8
            );
            
            ui.painter().rect_stroke(
                response.rect.expand(2.0),
                Rounding::same(10.0),
                Stroke::new(2.0, glow_color),
            );
        }
        
        response
    }
    
    pub fn show_compact(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool, size: Vec2) -> egui::Response {
        let button_color = theme.get_button_color(is_primary);
        let text_color = if is_primary { Color32::WHITE } else { theme.text_primary };
        
        let button = egui::Button::new(
            egui::RichText::new(text).color(text_color).size(12.0)
        )
        .fill(button_color)
        .stroke(Stroke::new(if is_primary { 0.0 } else { 1.0 }, theme.border))
        .rounding(Rounding::same(6.0));
        
        ui.add_sized(size, button)
    }
}

// Modern standardized button component
pub struct ModernButton;

impl ModernButton {
    pub fn primary(ui: &mut egui::Ui, theme: &Theme, text: &str) -> egui::Response {
        Self::show(ui, theme, text, true, egui::vec2(120.0, 28.0))
    }
    
    pub fn secondary(ui: &mut egui::Ui, theme: &Theme, text: &str) -> egui::Response {
        Self::show(ui, theme, text, false, egui::vec2(120.0, 28.0))
    }
    
    pub fn small(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool) -> egui::Response {
        Self::show(ui, theme, text, is_primary, egui::vec2(80.0, 22.0))
    }
    
    pub fn large(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool) -> egui::Response {
        Self::show(ui, theme, text, is_primary, egui::vec2(140.0, 32.0))
    }
    
    fn show(ui: &mut egui::Ui, theme: &Theme, text: &str, is_primary: bool, size: egui::Vec2) -> egui::Response {
        let button_color = theme.get_button_color(is_primary);
        let text_color = theme.get_button_text_color(is_primary);
        
        let button = egui::Button::new(
            egui::RichText::new(text).color(text_color).size(12.0)
        )
        .fill(button_color)
        .stroke(egui::Stroke::new(if is_primary { 0.0 } else { 1.0 }, theme.border))
        .rounding(egui::Rounding::same(4.0));
        
        ui.add_sized(size, button)
    }
}

// Modern card component with standardized styling
pub struct ModernCard;

impl ModernCard {
    pub fn show<R>(
        ui: &mut egui::Ui,
        theme: &Theme,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        Self::show_with_options(ui, theme, title, false, false, add_contents)
    }
    
    pub fn show_hoverable<R>(
        ui: &mut egui::Ui,
        theme: &Theme,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        Self::show_with_options(ui, theme, title, true, false, add_contents)
    }
    
    pub fn show_with_options<R>(
        ui: &mut egui::Ui,
        theme: &Theme,
        title: &str,
        is_hoverable: bool,
        is_active: bool,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_response(available_rect.size(), egui::Sense::hover());
        let is_hovered = is_hoverable && response.hovered();
        
        let (bg_color, border_color, border_width) = theme.get_card_colors(is_hovered, is_active);
        
        egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(border_width, border_color))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(8.0))
            .shadow(theme.get_shadow(is_hovered))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if !title.is_empty() {
                        ui.label(
                            egui::RichText::new(title)
                                .size(16.0)
                                .color(theme.text_primary)
                                .strong()
                        );
                        ui.add_space(4.0);
                    }
                    add_contents(ui)
                }).inner
            }).inner
    }
}

// Standardized spacing helper
pub struct Spacing;

impl Spacing {
    pub fn xs(ui: &mut egui::Ui) {
        ui.add_space(2.0);
    }
    
    pub fn sm(ui: &mut egui::Ui) {
        ui.add_space(4.0);
    }
    
    pub fn md(ui: &mut egui::Ui) {
        ui.add_space(6.0);
    }
    
    pub fn lg(ui: &mut egui::Ui) {
        ui.add_space(8.0);
    }
    
    pub fn xl(ui: &mut egui::Ui) {
        ui.add_space(12.0);
    }
    
    pub fn xxl(ui: &mut egui::Ui) {
        ui.add_space(16.0);
    }
}

// Standardized typography
pub struct Typography;

impl Typography {
    pub fn title(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(20.0)
                .color(theme.text_primary)
                .strong()
        );
    }
    
    pub fn heading(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(16.0)
                .color(theme.text_primary)
                .strong()
        );
    }
    
    pub fn body(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(12.0)
                .color(theme.text_primary)
        );
    }
    
    pub fn secondary(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(12.0)
                .color(theme.text_secondary)
        );
    }
    
    pub fn small(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(10.0)
                .color(theme.text_secondary)
        );
    }
    
    pub fn disabled(ui: &mut egui::Ui, theme: &Theme, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(12.0)
                .color(theme.text_disabled)
        );
    }
}

pub struct Card;

impl Card {
    pub fn show<R>(
        ui: &mut egui::Ui,
        theme: &Theme,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        egui::Frame::none()
            .fill(theme.surface)
            .stroke(Stroke::new(1.0, theme.border))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(18.0)
                            .color(theme.text_primary)
                            .strong()
                    );
                    ui.add_space(12.0);
                    add_contents(ui)
                }).inner
            }).inner
    }
}

pub struct InputField;

impl InputField {
    pub fn show(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &mut String, placeholder: &str) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).color(theme.text_secondary).size(12.0));
            ui.add_space(4.0);
            
            let response = ui.allocate_response(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
            let is_focused = ui.memory(|mem| mem.has_focus(response.id));
            
            // Custom frame for better text input styling
            let bg_color = if is_focused { theme.surface_variant } else { theme.surface_variant };
            let border_color = if is_focused { theme.primary } else { theme.border };
            let border_width = if is_focused { 2.0 } else { 1.0 };
            
            egui::Frame::none()
                .fill(bg_color)
                .stroke(Stroke::new(border_width, border_color))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::singleline(value)
                        .hint_text(egui::RichText::new(placeholder).color(theme.text_disabled).size(13.0))
                        .desired_width(ui.available_width())
                        .font(egui::TextStyle::Body)
                        .frame(false); // Remove default frame
                    
                    ui.add(text_edit);
                });
        });
    }
    
    pub fn show_password(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &mut String, placeholder: &str) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).color(theme.text_secondary).size(12.0));
            ui.add_space(4.0);
            
            let response = ui.allocate_response(egui::vec2(ui.available_width(), 32.0), egui::Sense::click());
            let is_focused = ui.memory(|mem| mem.has_focus(response.id));
            
            // Custom frame for better text input styling
            let bg_color = if is_focused { theme.surface_variant } else { theme.surface_variant };
            let border_color = if is_focused { theme.primary } else { theme.border };
            let border_width = if is_focused { 2.0 } else { 1.0 };
            
            egui::Frame::none()
                .fill(bg_color)
                .stroke(Stroke::new(border_width, border_color))
                .rounding(Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::singleline(value)
                        .hint_text(egui::RichText::new(placeholder).color(theme.text_disabled).size(13.0))
                        .desired_width(ui.available_width())
                        .password(true)
                        .font(egui::TextStyle::Body)
                        .frame(false); // Remove default frame
                    
                    ui.add(text_edit);
                });
        });
    }
    
    pub fn show_inline(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &mut String, placeholder: &str) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(label).color(theme.text_secondary).size(12.0));
            ui.add_space(8.0);
            
            let response = ui.allocate_response(egui::vec2(200.0, 28.0), egui::Sense::click());
            let is_focused = ui.memory(|mem| mem.has_focus(response.id));
            
            let bg_color = if is_focused { theme.surface_variant } else { theme.surface_variant };
            let border_color = if is_focused { theme.primary } else { theme.border };
            let border_width = if is_focused { 2.0 } else { 1.0 };
            
            egui::Frame::none()
                .fill(bg_color)
                .stroke(Stroke::new(border_width, border_color))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::singleline(value)
                        .hint_text(egui::RichText::new(placeholder).color(theme.text_disabled).size(12.0))
                        .desired_width(ui.available_width())
                        .font(egui::TextStyle::Body)
                        .frame(false);
                    
                    ui.add(text_edit);
                });
        });
    }
}

// Modern device card component with consistent styling
pub struct DeviceCard;

impl DeviceCard {
    pub fn show_rdp<F>(
        ui: &mut egui::Ui,
        theme: &Theme,
        name: &str,
        host: &str,
        port: u16,
        on_connect: F,
    ) -> egui::Response
    where
        F: FnOnce(),
    {
        let response = ui.allocate_response(egui::vec2(200.0, 70.0), egui::Sense::hover());
        let is_hovered = response.hovered();
        
        let (bg_color, border_color, border_width) = theme.get_card_colors(is_hovered, false);
        
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
                            egui::RichText::new(name)
                                .strong()
                                .size(14.0)
                                .color(theme.text_primary)
                        );
                        ui.label(
                            egui::RichText::new(format!("{}:{}", host, port))
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
                        if ui.add(
                            egui::Button::new("Connect")
                                .fill(theme.get_action_button_color(ActionType::Primary))
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(70.0, 30.0))
                        ).clicked() {
                            on_connect();
                        }
                    });
                });
            });
        
        response
    }
    
    pub fn show_wol<F1, F2>(
        ui: &mut egui::Ui,
        theme: &Theme,
        name: &str,
        ip_address: &str,
        is_online: bool,
        on_wake: F1,
        on_ping: F2,
    ) -> egui::Response
    where
        F1: FnOnce(),
        F2: FnOnce(),
    {
        let response = ui.allocate_response(egui::vec2(200.0, 70.0), egui::Sense::hover());
        let is_hovered = response.hovered();
        
        let (bg_color, border_color, border_width) = theme.get_card_colors(is_hovered, is_online);
        
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
                            egui::RichText::new(name)
                                .strong()
                                .size(14.0)
                                .color(theme.text_primary)
                        );
                        ui.label(
                            egui::RichText::new(ip_address)
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
                            if ui.add(
                                egui::Button::new("Wake")
                                    .fill(theme.get_action_button_color(ActionType::Success))
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(50.0, 28.0))
                            ).clicked() {
                                on_wake();
                            }
                            
                            if ui.add(
                                egui::Button::new("Ping")
                                    .fill(theme.get_action_button_color(ActionType::Secondary))
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(50.0, 28.0))
                            ).clicked() {
                                on_ping();
                            }
                        });
                    });
                });
            });
        
        response
    }
}