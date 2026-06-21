// Glint - Toolbar panel
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::app::AppMessage;
use crossbeam_channel::Sender;
use egui::{Button, Color32, RichText, Vec2};

/// The main toolbar panel rendered at the top of the application
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

    /// Render the toolbar
    pub fn render(&mut self, ui: &mut egui::Ui, tx: &Sender<AppMessage>, theme_cycle: &str) {
        ui.horizontal(|ui| {
            ui.set_min_height(40.0);

            // File operations
            if self.icon_tool_button(ui, "+", "Open File (Ctrl+O)").clicked() {
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
            if self.icon_tool_button(ui, "--", "Open Folder (Ctrl+D)").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    let _ = tx.send(AppMessage::OpenDirectory(path));
                }
            }

            ui.separator();

            // Navigation controls
            if self.icon_tool_button(ui, "<", "Previous").clicked() {
                let _ = tx.send(AppMessage::PreviousImage);
            }
            if self.icon_tool_button(ui, ">", "Next").clicked() {
                let _ = tx.send(AppMessage::NextImage);
            }

            ui.separator();

            // Zoom controls
            if self.icon_tool_button(ui, "/+", "Zoom In").clicked() {
                let _ = tx.send(AppMessage::ZoomIn);
            }
            if self.icon_tool_button(ui, "/-", "Zoom Out").clicked() {
                let _ = tx.send(AppMessage::ZoomOut);
            }
            if self.icon_tool_button(ui, "=1", "Fit to View").clicked() {
                let _ = tx.send(AppMessage::ZoomFit);
            }
            if self.icon_tool_button(ui, "1:1", "Actual Size").clicked() {
                let _ = tx.send(AppMessage::ZoomActual);
            }

            ui.separator();

            // View controls
            let gallery_btn = self.toggle_button(ui, "[=]", "Gallery", self.is_gallery_active);
            let editor_btn = self.toggle_button(ui, "[&]", "Editor", self.is_editor_active);

            if gallery_btn.clicked() {
                self.is_gallery_active = !self.is_gallery_active;
                let _ = tx.send(AppMessage::ToggleGallery);
            }
            if editor_btn.clicked() {
                self.is_editor_active = !self.is_editor_active;
                let _ = tx.send(AppMessage::ToggleEditing);
            }

            // Slideshow
            if self.icon_tool_button(ui, ">>", "Slideshow (F5)").clicked() {
                let _ = tx.send(AppMessage::ToggleSlideshow);
            }

            ui.separator();

            if self.icon_tool_button(ui, "[ ]", "Fullscreen (F11)").clicked() {
                let _ = tx.send(AppMessage::ToggleFullscreen);
            }

            // Right side: theme toggle (NOW WORKS)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let theme_icon = match theme_cycle {
                    "Dark" => "(*)",
                    "Light" => "(O)",
                    "Amoled" => "(-)",
                    _ => "(*)",
                };
                if self.icon_tool_button(ui, theme_icon, "Toggle Theme (Ctrl+T)").clicked() {
                    let _ = tx.send(AppMessage::ToggleTheme);
                }
            });
        });
    }

    /// Create an icon-based toolbar button with hover tooltip
    fn icon_tool_button(&self, ui: &mut egui::Ui, icon: &str, tooltip: &str) -> egui::Response {
        ui.add(
            Button::new(RichText::new(icon).size(13.0))
                .min_size(Vec2::new(32.0, 28.0)),
        )
        .on_hover_text(tooltip)
    }

    /// Create a toggle button with icon
    fn toggle_button(&self, ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> egui::Response {
        let text = format!("{} {}", icon, if active { "X" } else { label });
        if active {
            ui.add(
                Button::new(RichText::new(&text).size(12.0).color(Color32::WHITE))
                    .min_size(Vec2::new(48.0, 28.0))
                    .fill(Color32::from_rgb(70, 130, 255)),
            )
            .on_hover_text(label)
        } else {
            ui.add(
                Button::new(RichText::new(&text).size(12.0))
                    .min_size(Vec2::new(48.0, 28.0)),
            )
            .on_hover_text(label)
        }
    }
}
