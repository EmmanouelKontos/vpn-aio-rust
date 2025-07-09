use eframe::egui::Color32;

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
            background: Color32::from_rgba_premultiplied(16, 16, 20, 240),
            surface: Color32::from_rgba_premultiplied(24, 24, 28, 200),
            surface_variant: Color32::from_rgba_premultiplied(40, 44, 52, 220),
            primary: Color32::from_rgba_premultiplied(99, 102, 241, 255),
            primary_variant: Color32::from_rgba_premultiplied(79, 82, 221, 255),
            secondary: Color32::from_rgba_premultiplied(139, 92, 246, 255),
            accent: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            text_primary: Color32::from_rgba_premultiplied(248, 250, 252, 255),
            text_secondary: Color32::from_rgba_premultiplied(148, 163, 184, 255),
            text_disabled: Color32::from_rgba_premultiplied(71, 85, 105, 255),
            border: Color32::from_rgba_premultiplied(75, 85, 99, 255),
            success: Color32::from_rgba_premultiplied(34, 197, 94, 255),
            warning: Color32::from_rgba_premultiplied(251, 191, 36, 255),
            error: Color32::from_rgba_premultiplied(239, 68, 68, 255),
            selection_bg: Color32::from_rgba_premultiplied(99, 102, 241, 120),
            selection_text: Color32::from_rgba_premultiplied(255, 255, 255, 255),
            hover_bg: Color32::from_rgba_premultiplied(55, 65, 81, 180),
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
}