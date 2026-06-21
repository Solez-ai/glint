// Glint - Resize editor tool
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;

/// Image resize tool
pub struct ResizeEditor {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    maintain_aspect: bool,
    preset: PresetSize,
    resolution: u32,
    filter: ResampleFilter,
    pub apply_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PresetSize {
    #[allow(dead_code)]
    Custom,
    SocialMedia,
    Desktop4k,
    Desktop1080p,
    Tablet,
    Phone,
    PrintA4,
    PrintA3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResampleFilter {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos3,
}

impl ResampleFilter {
    fn label(&self) -> &'static str {
        match self {
            ResampleFilter::Nearest => "Nearest (Fast)",
            ResampleFilter::Bilinear => "Bilinear",
            ResampleFilter::Bicubic => "Bicubic",
            ResampleFilter::Lanczos3 => "Lanczos (Best)",
        }
    }
}

impl ResizeEditor {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            original_width: 0,
            original_height: 0,
            maintain_aspect: true,
            preset: PresetSize::Desktop1080p,
            resolution: 72,
            filter: ResampleFilter::Lanczos3,
            apply_pending: false,
        }
    }

    pub fn take_apply(&mut self) -> bool {
        std::mem::take(&mut self.apply_pending)
    }

    pub fn set_original_size(&mut self, width: u32, height: u32) {
        self.original_width = width;
        self.original_height = height;
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Resize Image")
                .size(14.0)
                .color(Color32::from_rgb(200, 200, 200)),
        );

        ui.add_space(4.0);

        if self.original_width > 0 {
            ui.label(
                egui::RichText::new(format!(
                    "Original: {} x {} px",
                    self.original_width, self.original_height
                ))
                .size(11.0)
                .color(Color32::from_rgb(150, 150, 150)),
            );
        }

        ui.add_space(8.0);

        ui.label("Preset");
        ui.horizontal_wrapped(|ui| {
            let presets = [
                (PresetSize::SocialMedia, "Social"),
                (PresetSize::Desktop4k, "4K"),
                (PresetSize::Desktop1080p, "1080p"),
                (PresetSize::Tablet, "Tablet"),
                (PresetSize::Phone, "Phone"),
                (PresetSize::PrintA4, "A4"),
                (PresetSize::PrintA3, "A3"),
            ];

            for (preset, label) in &presets {
                let selected = self.preset == *preset;
                if ui.selectable_label(selected, *label).clicked() {
                    self.preset = *preset;
                    match preset {
                        PresetSize::Custom => {}
                        PresetSize::SocialMedia => { self.width = 1200; self.height = 630; }
                        PresetSize::Desktop4k => { self.width = 3840; self.height = 2160; }
                        PresetSize::Desktop1080p => { self.width = 1920; self.height = 1080; }
                        PresetSize::Tablet => { self.width = 2048; self.height = 1536; }
                        PresetSize::Phone => { self.width = 1080; self.height = 1920; }
                        PresetSize::PrintA4 => { self.width = 2480; self.height = 3508; }
                        PresetSize::PrintA3 => { self.width = 3508; self.height = 4961; }
                    }
                }
            }
        });

        ui.add_space(8.0);

        ui.label("Dimensions");
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.width, 1..=10000).text("Width"));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.height, 1..=10000).text("Height"));
        });

        ui.checkbox(&mut self.maintain_aspect, "Maintain Aspect Ratio");

        ui.add_space(8.0);

        ui.label("Resolution (DPI)");
        ui.add(egui::Slider::new(&mut self.resolution, 72..=1200).text("DPI"));

        ui.add_space(8.0);

        ui.label("Resampling Filter");
        egui::ComboBox::new("resample_filter", self.filter.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.filter, ResampleFilter::Nearest, ResampleFilter::Nearest.label());
                ui.selectable_value(&mut self.filter, ResampleFilter::Bilinear, ResampleFilter::Bilinear.label());
                ui.selectable_value(&mut self.filter, ResampleFilter::Bicubic, ResampleFilter::Bicubic.label());
                ui.selectable_value(&mut self.filter, ResampleFilter::Lanczos3, ResampleFilter::Lanczos3.label());
            });

        ui.add_space(12.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!("Resize to {} x {}", self.width, self.height))
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
    }
}

impl Default for ResizeEditor {
    fn default() -> Self {
        Self::new()
    }
}
