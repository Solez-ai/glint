// Glint - Color adjustment editor
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;

/// Color and lighting adjustment controls
pub struct AdjustEditor {
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub vibrance: i32,
    pub exposure: i32,
    pub gamma: f32,
    pub highlights: i32,
    pub shadows: i32,
    pub temperature: i32,
    pub tint: i32,
    pub hue: i32,
    pub auto_adjust: bool,
    pub apply_pending: bool,
}

impl AdjustEditor {
    pub fn new() -> Self {
        Self {
            brightness: 0,
            contrast: 0,
            saturation: 0,
            vibrance: 0,
            exposure: 0,
            gamma: 1.0,
            highlights: 0,
            shadows: 0,
            temperature: 0,
            tint: 0,
            hue: 0,
            auto_adjust: false,
            apply_pending: false,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Color & Lighting")
                .size(14.0)
                .color(Color32::from_rgb(200, 200, 200)),
        );

        ui.add_space(4.0);

        ui.checkbox(&mut self.auto_adjust, "Auto Adjust");

        ui.add_space(8.0);

        // Direct slider usage to avoid borrow checker issues
        ui.add(egui::Slider::new(&mut self.brightness, -100..=100).text("Brightness"));
        ui.add(egui::Slider::new(&mut self.contrast, -100..=100).text("Contrast"));
        ui.add(egui::Slider::new(&mut self.saturation, -100..=100).text("Saturation"));
        ui.add(egui::Slider::new(&mut self.vibrance, -100..=100).text("Vibrance"));

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Tone").size(13.0).color(Color32::from_rgb(180, 180, 180)),
        );
        ui.add(egui::Slider::new(&mut self.exposure, -100..=100).text("Exposure"));
        ui.add(egui::Slider::new(&mut self.gamma, 0.1..=5.0).text("Gamma"));
        ui.add(egui::Slider::new(&mut self.highlights, -100..=100).text("Highlights"));
        ui.add(egui::Slider::new(&mut self.shadows, -100..=100).text("Shadows"));

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("Color Balance")
                .size(13.0)
                .color(Color32::from_rgb(180, 180, 180)),
        );
        ui.add(egui::Slider::new(&mut self.temperature, -100..=100).text("Temperature"));
        ui.add(egui::Slider::new(&mut self.tint, -100..=100).text("Tint"));
        ui.add(egui::Slider::new(&mut self.hue, -180..=180).text("Hue"));

        ui.add_space(12.0);

        if ui.button(egui::RichText::new("Reset All").size(13.0)).clicked() {
            self.brightness = 0;
            self.contrast = 0;
            self.saturation = 0;
            self.vibrance = 0;
            self.exposure = 0;
            self.gamma = 1.0;
            self.highlights = 0;
            self.shadows = 0;
            self.temperature = 0;
            self.tint = 0;
            self.hue = 0;
            self.auto_adjust = false;
        }

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Apply Adjustments")
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

    pub fn take_apply(&mut self) -> bool {
        std::mem::take(&mut self.apply_pending)
    }
}

impl Default for AdjustEditor {
    fn default() -> Self {
        Self::new()
    }
}
