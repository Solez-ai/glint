// Glint - Crop editor tool
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;

/// Crop tool for selecting and cropping image regions
pub struct CropEditor {
    pub crop_rect: [f32; 4],
    constrain_aspect: bool,
    aspect_ratio: f32,
    aspect_preset: AspectPreset,
    apply_pending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AspectPreset {
    Free,
    Square,
    Standard4x3,
    Standard16x9,
    Photo3x2,
    Photo4x5,
    Photo9x16,
    Cinemascope,
}

impl AspectPreset {
    fn ratio(&self) -> Option<f32> {
        match self {
            AspectPreset::Free => None,
            AspectPreset::Square => Some(1.0),
            AspectPreset::Standard4x3 => Some(4.0 / 3.0),
            AspectPreset::Standard16x9 => Some(16.0 / 9.0),
            AspectPreset::Photo3x2 => Some(3.0 / 2.0),
            AspectPreset::Photo4x5 => Some(4.0 / 5.0),
            AspectPreset::Photo9x16 => Some(9.0 / 16.0),
            AspectPreset::Cinemascope => Some(2.35),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            AspectPreset::Free => "Free",
            AspectPreset::Square => "1:1",
            AspectPreset::Standard4x3 => "4:3",
            AspectPreset::Standard16x9 => "16:9",
            AspectPreset::Photo3x2 => "3:2",
            AspectPreset::Photo4x5 => "4:5",
            AspectPreset::Photo9x16 => "9:16",
            AspectPreset::Cinemascope => "2.35:1",
        }
    }
}

impl CropEditor {
    pub fn new() -> Self {
        Self {
            crop_rect: [0.0, 0.0, 100.0, 100.0],
            constrain_aspect: false,
            aspect_ratio: 1.0,
            aspect_preset: AspectPreset::Free,
            apply_pending: false,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Crop & Rotate")
                .size(14.0)
                .color(Color32::from_rgb(200, 200, 200)),
        );

        ui.add_space(8.0);

        ui.label("Aspect Ratio");
        ui.horizontal_wrapped(|ui| {
            let presets = [
                AspectPreset::Free,
                AspectPreset::Square,
                AspectPreset::Standard4x3,
                AspectPreset::Standard16x9,
                AspectPreset::Photo3x2,
                AspectPreset::Photo4x5,
                AspectPreset::Photo9x16,
                AspectPreset::Cinemascope,
            ];

            for preset in &presets {
                let selected = self.aspect_preset == *preset;
                if ui.selectable_label(selected, preset.label()).clicked() {
                    self.aspect_preset = *preset;
                    self.constrain_aspect = preset.ratio().is_some();
                    if let Some(ratio) = preset.ratio() {
                        self.aspect_ratio = ratio;
                    }
                }
            }
        });

        ui.add_space(8.0);

        ui.label("Crop Area");
        ui.add(egui::Slider::new(&mut self.crop_rect[0], 0.0..=100.0).text("Left (%)"));
        ui.add(egui::Slider::new(&mut self.crop_rect[1], 0.0..=100.0).text("Top (%)"));
        ui.add(egui::Slider::new(&mut self.crop_rect[2], 0.0..=100.0).text("Width (%)"));
        ui.add(egui::Slider::new(&mut self.crop_rect[3], 0.0..=100.0).text("Height (%)"));

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Apply Crop")
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

        if ui.button(egui::RichText::new("Reset").size(13.0)).clicked() {
            self.crop_rect = [0.0, 0.0, 100.0, 100.0];
            self.aspect_preset = AspectPreset::Free;
            self.constrain_aspect = false;
        }
    }
    pub fn take_apply(&mut self) -> bool {
        std::mem::take(&mut self.apply_pending)
    }
}

impl Default for CropEditor {
    fn default() -> Self {
        Self::new()
    }
}
