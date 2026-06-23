// Glint - UI module
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

mod gallery;
pub mod icons;
mod toolbar;
mod viewer;
mod window;

pub use gallery::GalleryPanel;
pub use icons::IconCache;
pub use toolbar::ToolbarPanel;
pub use viewer::ViewerPanel;
pub use window::AppTheme;

pub use icons::Icon;

/// Theming constants for the Glint UI
pub mod theme {
    use egui::Color32;

    /// Dark theme background
    pub const DARK_BG: Color32 = Color32::from_rgb(18, 18, 18);
    /// Dark theme surface
    pub const DARK_SURFACE: Color32 = Color32::from_rgb(28, 28, 28);
    /// Dark theme text
    pub const DARK_TEXT: Color32 = Color32::from_rgb(220, 220, 220);
    /// Dark theme accent
    pub const DARK_ACCENT: Color32 = Color32::from_rgb(70, 130, 255);
    /// Dark theme secondary text
    pub const DARK_TEXT_SECONDARY: Color32 = Color32::from_rgb(150, 150, 150);

    /// Light theme background
    pub const LIGHT_BG: Color32 = Color32::from_rgb(240, 240, 240);
    /// Light theme surface
    pub const LIGHT_SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    /// Light theme text
    pub const LIGHT_TEXT: Color32 = Color32::from_rgb(30, 30, 30);
    /// Light theme accent
    pub const LIGHT_ACCENT: Color32 = Color32::from_rgb(0, 90, 230);
    /// Light theme secondary text
    pub const LIGHT_TEXT_SECONDARY: Color32 = Color32::from_rgb(100, 100, 100);
}
