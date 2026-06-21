// Glint - Gallery/thumbnail grid panel
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::browser::FileBrowser;
use egui::{Color32, Sense, Vec2};
use std::path::PathBuf;

/// A single thumbnail item in the gallery
struct ThumbnailItem {
    path: PathBuf,
    filename: String,
}

/// Gallery panel showing a grid of image thumbnails
pub struct GalleryPanel {
    items: Vec<ThumbnailItem>,
    selected_index: usize,
    thumbnail_size: f32,
    has_items: bool,
    pending_open: Option<PathBuf>,
}

impl GalleryPanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            thumbnail_size: 120.0,
            has_items: false,
            pending_open: None,
        }
    }

    pub fn set_items(&mut self, paths: Vec<PathBuf>) {
        self.items = paths
            .into_iter()
            .map(|path| {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                ThumbnailItem { path, filename }
            })
            .collect();
        self.has_items = !self.items.is_empty();
        self.pending_open = None;
    }

    /// Take the pending file to open, if any
    pub fn take_pending_open(&mut self) -> Option<PathBuf> {
        self.pending_open.take()
    }

    /// Render the gallery panel
    pub fn render(&mut self, ui: &mut egui::Ui, _file_browser: &FileBrowser) {
        if !self.has_items || self.items.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(
                    egui::RichText::new("Open a folder to view thumbnails")
                        .size(14.0)
                        .color(Color32::from_rgb(150, 150, 150)),
                );
            });
            return;
        }

        // Gallery header
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Gallery ({} images)", self.items.len()))
                    .size(13.0)
                    .color(Color32::from_rgb(200, 200, 200)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("-").clicked() {
                    self.thumbnail_size = (self.thumbnail_size - 20.0).max(60.0);
                }
                if ui.button("+").clicked() {
                    self.thumbnail_size = (self.thumbnail_size + 20.0).min(300.0);
                }
            });
        });

        ui.separator();

        // Scrollable thumbnail grid using a simple grid layout
        egui::ScrollArea::horizontal()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let thumb_total = self.thumbnail_size + 8.0;

                    for (index, item) in self.items.iter().enumerate() {
                        let is_selected = index == self.selected_index;

                        let bg_color = if is_selected {
                            Color32::from_rgb(70, 130, 255)
                        } else {
                            Color32::from_rgb(28, 28, 28)
                        };

                        let (_id, rect) = ui.allocate_space(Vec2::new(thumb_total, thumb_total));

                        let painter = ui.painter();
                        painter.rect_filled(rect, 4.0, bg_color);

                        // Thumbnail inner area
                        let thumb_rect = egui::Rect::from_min_size(
                            rect.min + Vec2::new(4.0, 4.0),
                            Vec2::new(self.thumbnail_size, self.thumbnail_size - 28.0),
                        );
                        painter.rect_filled(thumb_rect, 4.0, Color32::from_rgb(50, 50, 50));

                        // Filename label
                        let truncated = if item.filename.len() > 12 {
                            format!("{}...", &item.filename[..12.min(item.filename.len())])
                        } else {
                            item.filename.clone()
                        };
                        painter.text(
                            egui::pos2(rect.center().x, rect.max.y - 10.0),
                            egui::Align2::CENTER_CENTER,
                            &truncated,
                            egui::FontId::proportional(10.0),
                            Color32::from_rgb(180, 180, 180),
                        );

                        // Click detection (stable ID per item to avoid egui ID conflicts)
                        let response = ui.interact(rect, egui::Id::new(("gallery_item", index)), Sense::click());
                        if response.clicked() {
                            self.selected_index = index;
                            self.pending_open = Some(item.path.clone());
                        }
                    }
                });
            });
    }
}
