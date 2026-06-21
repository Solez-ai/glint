// Glint - Export/save tool
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;
use std::path::PathBuf;

/// Export settings and controls
pub struct ExportEditor {
    pub format: ExportFormat,
    pub quality: u8,
    pub strip_metadata: bool,
    pub convert_profile: bool,
    pub suffix: String,
    pub overwrite: bool,
    /// Set to true when "Export Image" is clicked (uses original path + suffix logic)
    pub apply_pending: bool,
    /// Set when "Save As..." picks a destination path
    pub save_as_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Original,
    Png,
    Jpeg,
    Webp,
    Bmp,
    Tiff,
}

impl ExportFormat {
    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Original => "Same as Source",
            ExportFormat::Png => "PNG",
            ExportFormat::Jpeg => "JPEG",
            ExportFormat::Webp => "WebP",
            ExportFormat::Bmp => "BMP",
            ExportFormat::Tiff => "TIFF",
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Original => "",
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Webp => "webp",
            ExportFormat::Bmp => "bmp",
            ExportFormat::Tiff => "tiff",
        }
    }
}

impl ExportEditor {
    pub fn new() -> Self {
        Self {
            format: ExportFormat::Original,
            quality: 92,
            strip_metadata: false,
            convert_profile: false,
            suffix: "_edited".to_string(),
            overwrite: false,
            apply_pending: false,
            save_as_path: None,
        }
    }

    pub fn take_apply(&mut self) -> bool {
        std::mem::take(&mut self.apply_pending)
    }

    pub fn take_save_as_path(&mut self) -> Option<PathBuf> {
        self.save_as_path.take()
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Export Settings")
                .size(13.0)
                .color(Color32::from_rgb(200, 200, 200)),
        );

        ui.add_space(8.0);

        ui.label("Format");
        egui::ComboBox::new("export_format", self.format.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.format, ExportFormat::Original, ExportFormat::Original.label());
                ui.selectable_value(&mut self.format, ExportFormat::Png, ExportFormat::Png.label());
                ui.selectable_value(&mut self.format, ExportFormat::Jpeg, ExportFormat::Jpeg.label());
                ui.selectable_value(&mut self.format, ExportFormat::Webp, ExportFormat::Webp.label());
                ui.selectable_value(&mut self.format, ExportFormat::Bmp, ExportFormat::Bmp.label());
                ui.selectable_value(&mut self.format, ExportFormat::Tiff, ExportFormat::Tiff.label());
            });

        ui.add_space(4.0);

        if matches!(self.format, ExportFormat::Jpeg | ExportFormat::Webp) {
            ui.add(egui::Slider::new(&mut self.quality, 1..=100).text("Quality"));
            ui.label(
                egui::RichText::new(match self.quality {
                    1..=30 => "Low quality, small file",
                    31..=60 => "Medium quality",
                    61..=85 => "High quality",
                    86..=95 => "Very high quality",
                    _ => "Maximum quality, large file",
                })
                .size(10.0)
                .color(Color32::from_rgb(150, 150, 150)),
            );
            ui.add_space(4.0);
        }

        ui.checkbox(&mut self.strip_metadata, "Strip Metadata (EXIF)");
        ui.checkbox(&mut self.convert_profile, "Convert Color Profile to sRGB");
        ui.checkbox(&mut self.overwrite, "Overwrite Original File");

        if !self.overwrite {
            ui.horizontal(|ui| {
                ui.label("Suffix:");
                ui.add(egui::TextEdit::singleline(&mut self.suffix).desired_width(100.0));
            });
        }

        ui.add_space(12.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Export Image")
                        .size(14.0)
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(70, 130, 255))
                .min_size(egui::Vec2::new(ui.available_width(), 32.0)),
            )
            .clicked()
        {
            self.apply_pending = true;
        }

        ui.add_space(4.0);

        if ui
            .add(
                egui::Button::new(egui::RichText::new("Save As...").size(13.0))
                    .min_size(egui::Vec2::new(ui.available_width(), 28.0)),
            )
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp", "tiff"])
                .save_file()
            {
                self.save_as_path = Some(path);
            }
        }
    }
}

impl Default for ExportEditor {
    fn default() -> Self {
        Self::new()
    }
}
