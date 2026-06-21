// Glint - Rotate editor tool
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use egui::Color32;

/// Rotation and flip editor controls
pub struct RotateEditor {
    pub angle: f32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub straighten: f32,
    pub apply_pending: bool,
}

impl RotateEditor {
    pub fn new() -> Self {
        Self {
            angle: 0.0,
            flip_h: false,
            flip_v: false,
            straighten: 0.0,
            apply_pending: false,
        }
    }

    pub fn take_apply(&mut self) -> bool {
        std::mem::take(&mut self.apply_pending)
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Rotate & Flip")
                .size(14.0)
                .color(Color32::from_rgb(200, 200, 200)),
        );

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let rotate_left = ui.add(
                egui::Button::new(egui::RichText::new("Rotate Left (90)").size(12.0))
                    .min_size(egui::Vec2::new(ui.available_width() / 2.0 - 4.0, 36.0)),
            );
            if rotate_left.clicked() {
                self.angle = (self.angle - 90.0).rem_euclid(360.0);
            }

            let rotate_right = ui.add(
                egui::Button::new(egui::RichText::new("Rotate Right (90)").size(12.0))
                    .min_size(egui::Vec2::new(ui.available_width(), 36.0)),
            );
            if rotate_right.clicked() {
                self.angle = (self.angle + 90.0).rem_euclid(360.0);
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let flip_h_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new(if self.flip_h { "Flip H (On)" } else { "Flip H (Off)" })
                        .size(12.0),
                )
                .fill(if self.flip_h { Color32::from_rgb(70, 130, 255) } else { Color32::from_rgb(40, 40, 40) })
                .min_size(egui::Vec2::new(ui.available_width() / 2.0 - 4.0, 36.0)),
            );
            if flip_h_btn.clicked() {
                self.flip_h = !self.flip_h;
            }

            let flip_v_btn = ui.add(
                egui::Button::new(
                    egui::RichText::new(if self.flip_v { "Flip V (On)" } else { "Flip V (Off)" })
                        .size(12.0),
                )
                .fill(if self.flip_v { Color32::from_rgb(70, 130, 255) } else { Color32::from_rgb(40, 40, 40) })
                .min_size(egui::Vec2::new(ui.available_width(), 36.0)),
            );
            if flip_v_btn.clicked() {
                self.flip_v = !self.flip_v;
            }
        });

        ui.add_space(12.0);

        ui.label("Custom Angle");
        ui.add(egui::Slider::new(&mut self.angle, 0.0..=360.0).text("Degrees"));

        ui.add_space(8.0);

        ui.label("Straighten");
        ui.add(egui::Slider::new(&mut self.straighten, -45.0..=45.0).text("Degrees"));

        ui.add_space(8.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Apply Rotation")
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

impl Default for RotateEditor {
    fn default() -> Self {
        Self::new()
    }
}
