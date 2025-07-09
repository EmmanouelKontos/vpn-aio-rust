use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use crate::ui::theme::Theme;

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
            theme.text_primary 
        } else { 
            theme.text_secondary 
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
            egui::RichText::new(display_text).color(text_color)
        )
        .fill(button_color)
        .stroke(Stroke::new(1.0, theme.border))
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
            ui.label(egui::RichText::new(label).color(theme.text_secondary));
            ui.add_space(4.0);
            
            let text_edit = egui::TextEdit::singleline(value)
                .hint_text(placeholder)
                .desired_width(ui.available_width())
                .font(egui::TextStyle::Body);
            
            ui.add(text_edit);
        });
    }
    
    pub fn show_password(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &mut String, placeholder: &str) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).color(theme.text_secondary));
            ui.add_space(4.0);
            
            let text_edit = egui::TextEdit::singleline(value)
                .hint_text(placeholder)
                .desired_width(ui.available_width())
                .password(true)
                .font(egui::TextStyle::Body);
            
            ui.add(text_edit);
        });
    }
}