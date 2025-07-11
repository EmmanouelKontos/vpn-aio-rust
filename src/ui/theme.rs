use eframe::egui::Color32;

#[derive(Clone, Copy, PartialEq)]
pub enum DeviceType {
    RDP,
    WOL,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActionType {
    Primary,
    Success,
    Warning,
    Error,
    Secondary,
}

pub struct Theme {
    pub background: Color32,
    pub surface: Color32,
    pub surface_variant: Color32,
    pub primary: Color32,
    pub primary_variant: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub border: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub selection_bg: Color32,
    pub selection_text: Color32,
    pub hover_bg: Color32,
    pub loading: Color32,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            background: Color32::from_rgba_premultiplied(15, 15, 20, 255),
            surface: Color32::from_rgba_premultiplied(25, 25, 32, 255),
            surface_variant: Color32::from_rgba_premultiplied(35, 35, 45, 255),
            primary: Color32::from_rgba_premultiplied(79, 172, 254, 255),
            primary_variant: Color32::from_rgba_premultiplied(59, 152, 234, 255),
            secondary: Color32::from_rgba_premultiplied(139, 92, 246, 255),
            accent: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            text_primary: Color32::from_rgba_premultiplied(248, 250, 252, 255),
            text_secondary: Color32::from_rgba_premultiplied(148, 163, 184, 255),
            text_disabled: Color32::from_rgba_premultiplied(100, 116, 139, 255),
            border: Color32::from_rgba_premultiplied(55, 65, 81, 255),
            success: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            warning: Color32::from_rgba_premultiplied(251, 191, 36, 255),
            error: Color32::from_rgba_premultiplied(239, 68, 68, 255),
            selection_bg: Color32::from_rgba_premultiplied(79, 172, 254, 120),
            selection_text: Color32::from_rgba_premultiplied(255, 255, 255, 255),
            hover_bg: Color32::from_rgba_premultiplied(45, 55, 72, 255),
            loading: Color32::from_rgba_premultiplied(139, 92, 246, 255),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color32::from_rgba_premultiplied(248, 250, 252, 255),
            surface: Color32::from_rgba_premultiplied(255, 255, 255, 240),
            surface_variant: Color32::from_rgba_premultiplied(241, 245, 249, 200),
            primary: Color32::from_rgba_premultiplied(99, 102, 241, 255),
            primary_variant: Color32::from_rgba_premultiplied(79, 82, 221, 255),
            secondary: Color32::from_rgba_premultiplied(139, 92, 246, 255),
            accent: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            text_primary: Color32::from_rgba_premultiplied(15, 23, 42, 255),
            text_secondary: Color32::from_rgba_premultiplied(51, 65, 85, 255),
            text_disabled: Color32::from_rgba_premultiplied(148, 163, 184, 255),
            border: Color32::from_rgba_premultiplied(226, 232, 240, 255),
            success: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            warning: Color32::from_rgba_premultiplied(251, 191, 36, 255),
            error: Color32::from_rgba_premultiplied(239, 68, 68, 255),
            selection_bg: Color32::from_rgba_premultiplied(99, 102, 241, 120),
            selection_text: Color32::from_rgba_premultiplied(255, 255, 255, 255),
            hover_bg: Color32::from_rgba_premultiplied(241, 245, 249, 180),
            loading: Color32::from_rgba_premultiplied(139, 92, 246, 255),
        }
    }

    pub fn get_status_color(&self, is_connected: bool) -> Color32 {
        if is_connected {
            self.success
        } else {
            self.text_disabled
        }
    }

    pub fn get_button_color(&self, is_primary: bool) -> Color32 {
        if is_primary {
            self.primary
        } else {
            self.surface_variant
        }
    }
    
    pub fn get_button_text_color(&self, is_primary: bool) -> Color32 {
        if is_primary {
            Color32::WHITE
        } else {
            self.text_primary
        }
    }
    
    pub fn get_card_colors(&self, is_hovered: bool, is_active: bool) -> (Color32, Color32, f32) {
        let bg_color = if is_hovered {
            self.hover_bg
        } else {
            self.surface_variant
        };
        
        let border_color = if is_active {
            self.success
        } else if is_hovered {
            self.primary
        } else {
            self.border
        };
        
        let border_width = if is_hovered || is_active { 2.0 } else { 1.0 };
        
        (bg_color, border_color, border_width)
    }
    
    pub fn get_shadow(&self, is_hovered: bool) -> eframe::egui::Shadow {
        if is_hovered {
            eframe::egui::Shadow {
                offset: eframe::egui::vec2(0.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: eframe::egui::Color32::from_black_alpha(50),
            }
        } else {
            eframe::egui::Shadow {
                offset: eframe::egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: eframe::egui::Color32::from_black_alpha(25),
            }
        }
    }
    
    // Helper for consistent icon colors based on device type
    pub fn get_device_icon_color(&self, device_type: DeviceType, is_online: bool) -> eframe::egui::Color32 {
        match device_type {
            DeviceType::RDP => self.primary,
            DeviceType::WOL => {
                if is_online {
                    self.success
                } else {
                    self.text_disabled
                }
            }
        }
    }
    
    // Helper for consistent device status colors
    pub fn get_device_status_color(&self, is_online: bool) -> eframe::egui::Color32 {
        if is_online {
            self.success
        } else {
            self.text_disabled
        }
    }
    
    // Helper for consistent action button colors
    pub fn get_action_button_color(&self, action_type: ActionType) -> eframe::egui::Color32 {
        match action_type {
            ActionType::Primary => self.primary,
            ActionType::Success => self.success,
            ActionType::Warning => self.warning,
            ActionType::Error => self.error,
            ActionType::Secondary => self.surface_variant,
        }
    }
    
    // Compact spacing constants
    pub const SPACING_XS: f32 = 2.0;
    pub const SPACING_SM: f32 = 4.0;
    pub const SPACING_MD: f32 = 6.0;
    pub const SPACING_LG: f32 = 8.0;
    pub const SPACING_XL: f32 = 12.0;
    pub const SPACING_XXL: f32 = 16.0;
    
    // Compact sizes
    pub const BUTTON_HEIGHT: f32 = 28.0;
    pub const BUTTON_HEIGHT_SMALL: f32 = 22.0;
    pub const BUTTON_HEIGHT_LARGE: f32 = 32.0;
    pub const INPUT_HEIGHT: f32 = 28.0;
    pub const CARD_RADIUS: f32 = 6.0;
    pub const BUTTON_RADIUS: f32 = 4.0;
    pub const INPUT_RADIUS: f32 = 4.0;
    
    // Compact font sizes
    pub const FONT_SIZE_SMALL: f32 = 10.0;
    pub const FONT_SIZE_BODY: f32 = 12.0;
    pub const FONT_SIZE_MEDIUM: f32 = 13.0;
    pub const FONT_SIZE_LARGE: f32 = 14.0;
    pub const FONT_SIZE_HEADING: f32 = 16.0;
    pub const FONT_SIZE_TITLE: f32 = 20.0;
    
    // Compact card dimensions
    pub const CARD_MIN_HEIGHT: f32 = 45.0;
    pub const CARD_PADDING: f32 = 8.0;
    pub const SIDEBAR_WIDTH: f32 = 200.0;
    pub const DEVICE_CARD_WIDTH: f32 = 180.0;
    pub const DEVICE_CARD_HEIGHT: f32 = 50.0;
}