// Glint - Window and theme management
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppTheme {
    Dark,
    Light,
    Amoled,
}

impl AppTheme {
    /// Get the background color for this theme
    pub fn background_color(&self) -> Color32 {
        match self {
            AppTheme::Dark => super::theme::DARK_BG,
            AppTheme::Light => super::theme::LIGHT_BG,
            AppTheme::Amoled => Color32::BLACK,
        }
    }

    /// Get the surface color for this theme
    pub fn surface_color(&self) -> Color32 {
        match self {
            AppTheme::Dark => super::theme::DARK_SURFACE,
            AppTheme::Light => super::theme::LIGHT_SURFACE,
            AppTheme::Amoled => Color32::from_rgb(5, 5, 5),
        }
    }

    /// Get the text color for this theme
    pub fn text_color(&self) -> Color32 {
        match self {
            AppTheme::Dark | AppTheme::Amoled => super::theme::DARK_TEXT,
            AppTheme::Light => super::theme::LIGHT_TEXT,
        }
    }

    /// Get the accent color for this theme
    pub fn accent_color(&self) -> Color32 {
        match self {
            AppTheme::Dark | AppTheme::Amoled => super::theme::DARK_ACCENT,
            AppTheme::Light => super::theme::LIGHT_ACCENT,
        }
    }

    /// Get the secondary text color for this theme
    pub fn text_secondary_color(&self) -> Color32 {
        match self {
            AppTheme::Dark | AppTheme::Amoled => super::theme::DARK_TEXT_SECONDARY,
            AppTheme::Light => super::theme::LIGHT_TEXT_SECONDARY,
        }
    }

    /// Apply the theme to an egui context
    pub fn apply_to_context(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.visuals.dark_mode = matches!(self, AppTheme::Dark | AppTheme::Amoled);

        style.visuals.panel_fill = self.surface_color();
        style.visuals.window_fill = self.background_color();
        style.visuals.faint_bg_color = self.background_color();
        style.visuals.extreme_bg_color = self.surface_color();

        style.visuals.widgets.noninteractive.bg_fill = self.surface_color();
        style.visuals.widgets.noninteractive.fg_stroke.color = self.text_color();
        style.visuals.widgets.inactive.bg_fill = self.surface_color();
        style.visuals.widgets.active.bg_fill = self.accent_color();
        style.visuals.widgets.hovered.bg_fill = self.accent_color();

        style.visuals.hyperlink_color = self.accent_color();
        style.visuals.selection.bg_fill = self.accent_color();

        ctx.set_style(style);
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::Dark
    }
}
