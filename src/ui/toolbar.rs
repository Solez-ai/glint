// Glint - Toolbar panel
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::app::AppMessage;
use crate::ui::icons::{Icon, IconCache};
use crossbeam_channel::Sender;
use egui::{Color32, Vec2, RichText};

/// Main toolbar panel with Unicode icons
pub struct ToolbarPanel {
    pub is_gallery_active: bool,
    pub is_editor_active: bool,
}

impl ToolbarPanel {
    pub fn new() -> Self {
        Self {
            is_gallery_active: false,
            is_editor_active: false,
        }
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        tx: &Sender<AppMessage>,
        icons: &IconCache,
        theme_name: &str,
        text_color: Color32,
    ) {
        ui.horizontal(|ui| {
            ui.set_min_height(40.0);
            ui.add_space(8.0);

            // --- File operations ---
            if icons.icon_button_sized(ui, &Icon::Open, text_color, Vec2::new(36.0, 28.0), "Open File (Ctrl+O)").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &[
                        "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif",
                        "webp", "ico", "avif", "heic", "heif",
                    ])
                    .pick_file()
                {
                    let _ = tx.send(AppMessage::OpenFile(path));
                }
            }
            if icons.icon_button_sized(ui, &Icon::Folder, text_color, Vec2::new(36.0, 28.0), "Open Folder (Ctrl+D)").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let _ = tx.send(AppMessage::OpenDirectory(path));
                }
            }

            ui.separator();
            ui.add_space(4.0);

            // --- Navigation ---
            if icons.icon_button_sized(ui, &Icon::Prev, text_color, Vec2::new(30.0, 28.0), "Previous (Left Arrow)").clicked() {
                let _ = tx.send(AppMessage::PreviousImage);
            }
            if icons.icon_button_sized(ui, &Icon::Next, text_color, Vec2::new(30.0, 28.0), "Next (Right Arrow)").clicked() {
                let _ = tx.send(AppMessage::NextImage);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // --- Zoom ---
            if icons.icon_button_sized(ui, &Icon::ZoomIn, text_color, Vec2::new(30.0, 28.0), "Zoom In (Ctrl++)").clicked() {
                let _ = tx.send(AppMessage::ZoomIn);
            }
            if icons.icon_button_sized(ui, &Icon::ZoomOut, text_color, Vec2::new(30.0, 28.0), "Zoom Out (Ctrl+-)").clicked() {
                let _ = tx.send(AppMessage::ZoomOut);
            }
            if icons.icon_button_sized(ui, &Icon::Fit, text_color, Vec2::new(36.0, 28.0), "Fit to View (Ctrl+0)").clicked() {
                let _ = tx.send(AppMessage::ZoomFit);
            }
            if ui.add(
                egui::Button::new(egui::RichText::new("1:1").size(11.0))
                    .min_size(Vec2::new(36.0, 28.0)),
            ).on_hover_text("Actual Size (Ctrl+1)").clicked() {
                let _ = tx.send(AppMessage::ZoomActual);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // --- View toggles ---
            let gallery_btn = self.icon_toggle_button(ui, &Icon::Gallery, icons, self.is_gallery_active, text_color);
            let editor_btn = self.icon_toggle_button(ui, &Icon::Editor, icons, self.is_editor_active, text_color);

            if gallery_btn.clicked() {
                self.is_gallery_active = !self.is_gallery_active;
                let _ = tx.send(AppMessage::ToggleGallery);
            }
            if editor_btn.clicked() {
                self.is_editor_active = !self.is_editor_active;
                let _ = tx.send(AppMessage::ToggleEditing);
            }

            if icons.icon_button_sized(ui, &Icon::Slideshow, text_color, Vec2::new(36.0, 28.0), "Slideshow (F5)").clicked() {
                let _ = tx.send(AppMessage::ToggleSlideshow);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // --- Display controls ---
            if icons.icon_button_sized(ui, &Icon::Fullscreen, text_color, Vec2::new(36.0, 28.0), "Fullscreen (F11)").clicked() {
                let _ = tx.send(AppMessage::ToggleFullscreen);
            }

            if icons.icon_button_sized(ui, &Icon::RotateLeft, text_color, Vec2::new(30.0, 28.0), "Rotate Left ([)").clicked() {
                let _ = tx.send(AppMessage::RotateLeft);
            }
            if icons.icon_button_sized(ui, &Icon::RotateRight, text_color, Vec2::new(30.0, 28.0), "Rotate Right (])").clicked() {
                let _ = tx.send(AppMessage::RotateRight);
            }

            // Right side: theme toggle
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                let theme_icon = match theme_name {
                    "Dark" => &Icon::ThemeDark,
                    "Light" => &Icon::ThemeLight,
                    _ => &Icon::ThemeDark,
                };
                if icons.icon_button_sized(ui, theme_icon, text_color, Vec2::new(36.0, 28.0), format!("Theme: {}  (Ctrl+T)", theme_name).as_str()).clicked() {
                    let _ = tx.send(AppMessage::ToggleTheme);
                }
                ui.add_space(4.0);
            });

            ui.add_space(8.0);
        });
    }

    fn icon_toggle_button(
        &self,
        ui: &mut egui::Ui,
        icon: &Icon,
        icons: &IconCache,
        active: bool,
        tint: Color32,
    ) -> egui::Response {
        let text = RichText::new(icon.symbol())
            .size(icons.icon_size().x + 4.0)
            .color(tint)
            .strong();
        let btn = egui::Button::new(text).selected(active);
        ui.add_sized(Vec2::new(42.0, 28.0), btn)
    }
}
