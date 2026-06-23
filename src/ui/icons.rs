use egui::{Color32, Vec2, RichText};

/// Icon identifiers for all toolbar/UI icons
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Icon {
    Open,
    Folder,
    Prev,
    Next,
    ZoomIn,
    ZoomOut,
    Fit,
    Gallery,
    Editor,
    Slideshow,
    Fullscreen,
    RotateLeft,
    RotateRight,
    ThemeDark,
    ThemeLight,
}

impl Icon {
    /// Get the text label for this icon.
    /// Uses ONLY ASCII/guaranteed Unicode that renders in every font.
    pub fn symbol(&self) -> &'static str {
        match self {
            Icon::Open => "\u{2191}",       // ↑
            Icon::Folder => "\u{25A1}",     // □
            Icon::Prev => "\u{2190}",       // ←
            Icon::Next => "\u{2192}",       // →
            Icon::ZoomIn => "+",
            Icon::ZoomOut => "\u{2212}",    // −
            Icon::Fit => "\u{2194}",        // ↔
            Icon::Gallery => "\u{25A6}",    // ▦
            Icon::Editor => "\u{270E}",     // ✎
            Icon::Slideshow => "\u{25B6}",  // ▶
            Icon::Fullscreen => "\u{26F6}", // ⛶
            Icon::RotateLeft => "\u{21BA}", // ↺
            Icon::RotateRight => "\u{21BB}", // ↻
            Icon::ThemeDark => "\u{263E}",  // ☾
            Icon::ThemeLight => "\u{2600}", // ☀
        }
    }
}

/// Icon cache that renders icons using clean Unicode symbols.
/// No emoji — uses only geometric/arrow/misc symbols from Segoe UI.
pub struct IconCache {
    icon_size: f32,
}

impl IconCache {
    pub fn new(_ctx: &egui::Context, size: f32) -> Self {
        Self { icon_size: size }
    }

    pub fn icon_size(&self) -> Vec2 {
        Vec2::splat(self.icon_size)
    }

    /// Render an icon button with a specific minimum size.
    pub fn icon_button_sized(
        &self,
        ui: &mut egui::Ui,
        icon: &Icon,
        tint: Color32,
        min_size: Vec2,
        tooltip: &str,
    ) -> egui::Response {
        let text = RichText::new(icon.symbol())
            .size(self.icon_size + 4.0)
            .color(tint)
            .strong();
        ui.add_sized(min_size, egui::Button::new(text))
            .on_hover_text(tooltip)
    }

    /// Render an icon button with default sizing.
    pub fn icon_button(
        &self,
        ui: &mut egui::Ui,
        icon: &Icon,
        tint: Color32,
        tooltip: &str,
    ) -> egui::Response {
        let text = RichText::new(icon.symbol())
            .size(self.icon_size + 4.0)
            .color(tint)
            .strong();
        ui.add_sized(Vec2::new(28.0, 24.0), egui::Button::new(text))
            .on_hover_text(tooltip)
    }
}
