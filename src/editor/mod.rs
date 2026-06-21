// Glint - Editor module
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

mod adjust;
mod crop;
mod export;
mod resize;
mod rotate;

pub use adjust::AdjustEditor;
pub use crop::CropEditor;
pub use export::ExportEditor;
pub use resize::ResizeEditor;
pub use rotate::RotateEditor;

use crate::editor::export::ExportFormat;
use crate::image::GlintImage;
use crate::image::ImageCache;
use crate::browser::FileBrowser;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// The main editor state, managing all editing tools
pub struct Editor {
    /// Crop tool
    crop: CropEditor,
    /// Rotate tool
    rotate: RotateEditor,
    /// Resize tool
    resize: ResizeEditor,
    /// Color/lighting adjustments
    adjust: AdjustEditor,
    /// Export settings
    export: ExportEditor,
    /// Currently active tool
    active_tool: EditorTool,
    /// Original image before edits (for undo)
    original: Option<GlintImage>,
    /// Currently edited image
    current: Option<GlintImage>,
    /// Whether a processed result is ready for app.rs to consume
    pending_result: bool,
    /// Path of a newly-exported file (different from original) — signals app.rs to sync file_browser
    pending_export_path: Option<PathBuf>,
}

/// The currently active editing tool
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorTool {
    None,
    Crop,
    Rotate,
    Resize,
    Adjust,
    Annotate,
    Filter,
}

impl Editor {
    /// Create a new editor
    pub fn new() -> Self {
        Self {
            crop: CropEditor::new(),
            rotate: RotateEditor::new(),
            resize: ResizeEditor::new(),
            adjust: AdjustEditor::new(),
            export: ExportEditor::new(),
            active_tool: EditorTool::None,
            original: None,
            current: None,
            pending_result: false,
            pending_export_path: None,
        }
    }

    /// Render the editor panel
    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(
                egui::RichText::new("Edit").size(16.0),
            );
            ui.separator();

            // Tool selection buttons - NOW ACTUALLY WORK (no closures to avoid borrow issues)
            ui.horizontal_wrapped(|ui| {
                if self.tool_button(ui, "[*] Crop", EditorTool::Crop).clicked() {
                    self.active_tool = EditorTool::Crop;
                }
                if self.tool_button(ui, "[~] Rotate", EditorTool::Rotate).clicked() {
                    self.active_tool = EditorTool::Rotate;
                }
                if self.tool_button(ui, "[+] Resize", EditorTool::Resize).clicked() {
                    self.active_tool = EditorTool::Resize;
                }
                if self.tool_button(ui, "[&] Adjust", EditorTool::Adjust).clicked() {
                    self.active_tool = EditorTool::Adjust;
                }
            });

            ui.separator();

            // Render the active tool's panel
            match self.active_tool {
                EditorTool::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("Select a tool to begin editing")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(150, 150, 150)),
                        );
                    });
                }
                EditorTool::Crop => self.crop.render(ui),
                EditorTool::Rotate => self.rotate.render(ui),
                EditorTool::Resize => self.resize.render(ui),
                EditorTool::Adjust => self.adjust.render(ui),
                EditorTool::Annotate => {}
                EditorTool::Filter => {}
            }

            ui.separator();

            // Export section
            ui.collapsing("Export", |ui| {
                self.export.render(ui);
            });
        });

        // Process any pending Apply button operations from sub-editors
        self.apply_pending();

        // Process any pending export operations
        self.process_export();
    }

    /// Set the current image from a rotated/processed result
    pub fn set_current_image(&mut self, image: GlintImage) {
        self.current = Some(image);
        self.pending_result = true;
    }

    /// Take the current processed image (consumed by app.rs to update viewer)
    pub fn take_current_image(&mut self) -> Option<GlintImage> {
        if self.pending_result {
            self.pending_result = false;
            self.current.clone()
        } else {
            None
        }
    }

    /// Take the path of a newly-exported file (signals app.rs to sync file_browser and gallery)
    pub fn take_export_path(&mut self) -> Option<PathBuf> {
        self.pending_export_path.take()
    }

    /// Check and apply any pending operations from sub-editors
    fn apply_pending(&mut self) {
        use crate::image::ImageProcessor;

        let img = match self.current.as_ref() {
            Some(img) => img.clone(),
            None => return,
        };

        let result = if self.adjust.take_apply() {
            let mut img = img;
            if self.adjust.brightness != 0 {
                if let Ok(adjusted) = ImageProcessor::adjust_brightness(&img, self.adjust.brightness) {
                    img = adjusted;
                }
            }
            if self.adjust.contrast != 0 {
                if let Ok(adjusted) = ImageProcessor::adjust_contrast(&img, self.adjust.contrast as f32) {
                    img = adjusted;
                }
            }
            Some(img)
        } else if self.crop.take_apply() {
            let x = (self.crop.crop_rect[0] / 100.0 * img.width as f32) as u32;
            let y = (self.crop.crop_rect[1] / 100.0 * img.height as f32) as u32;
            let w = ((self.crop.crop_rect[2] / 100.0 * img.width as f32) as u32).max(1);
            let h = ((self.crop.crop_rect[3] / 100.0 * img.height as f32) as u32).max(1);
            ImageProcessor::crop(&img, x, y, w, h).ok()
        } else if self.resize.take_apply() {
            let w = self.resize.width.max(1);
            let h = self.resize.height.max(1);
            ImageProcessor::resize(&img, w, h).ok()
        } else if self.rotate.take_apply() {
            let mut img = img;
            if self.rotate.angle != 0.0 {
                if let Ok(rotated) = ImageProcessor::rotate(&img, self.rotate.angle) {
                    img = rotated;
                }
            }
            if self.rotate.flip_h {
                if let Ok(flipped) = ImageProcessor::flip(&img, true) {
                    img = flipped;
                }
            }
            if self.rotate.flip_v {
                if let Ok(flipped) = ImageProcessor::flip(&img, false) {
                    img = flipped;
                }
            }
            Some(img)
        } else {
            None
        };

        if let Some(processed) = result {
            log::info!("Editor: applied {}", processed.path.display());
            self.current = Some(processed);
            self.pending_result = true;
        }
    }

    /// Process any pending export/save operations
    fn process_export(&mut self) {
        let apply = self.export.take_apply();
        let save_as = self.export.take_save_as_path();

        if !apply && save_as.is_none() {
            return;
        }

        let img = match self.current.as_ref().or(self.original.as_ref()) {
            Some(img) => img.clone(),
            None => {
                log::warn!("Export: no image to save");
                return;
            }
        };

        // Determine the save path
        let save_path: PathBuf = if let Some(path) = save_as {
            path
        } else if self.export.overwrite {
            img.path.clone()
        } else {
            // Append suffix before extension
            let stem = img.path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
            let ext = self.export.format.extension();
            if !ext.is_empty() {
                img.path.with_file_name(format!("{}{}.{}", stem, self.export.suffix, ext))
            } else if let Some(old_ext) = img.path.extension().and_then(|e| e.to_str()) {
                img.path.with_file_name(format!("{}{}.{}", stem, self.export.suffix, old_ext))
            } else {
                img.path.with_file_name(format!("{}{}", stem, self.export.suffix))
            }
        };

        let is_new_path = save_path != img.path;

        // Perform the actual save
        match Self::save_image_to_disk(&img, &save_path, self.export.format, self.export.quality) {
            Ok(_) => {
                log::info!("Exported image to {:?}", save_path);
                // Update the current image with the save path and signal app.rs
                // so it inserts the result into the ImageCache and refreshes the viewer
                let mut saved = img;
                saved.path = save_path.clone();
                self.current = Some(saved);
                self.pending_result = true;
                // If this is a new file (not overwrite), signal app.rs to sync file_browser
                if is_new_path {
                    self.pending_export_path = Some(save_path);
                }
            }
            Err(e) => {
                log::error!("Failed to export image: {}", e);
            }
        }
    }

    /// Save a GlintImage to disk using the image crate's encoders
    fn save_image_to_disk(img: &GlintImage, path: &Path, format: ExportFormat, quality: u8) -> Result<()> {
        use std::io::BufWriter;
        use image::ImageEncoder;

        let rgba = img.data.to_rgba8();
        let (width, height) = rgba.dimensions();
        let bytes = rgba.into_raw();

        match format {
            ExportFormat::Original => {
                // Let DynamicImage auto-detect format from extension
                img.data.save(path).map_err(|e| anyhow::anyhow!("Save failed: {}", e))
            }
            ExportFormat::Png => {
                let file = std::fs::File::create(path)?;
                let w = BufWriter::new(file);
                let encoder = image::codecs::png::PngEncoder::new(w);
                encoder.write_image(&bytes, width, height, image::ExtendedColorType::Rgba8)?;
                Ok(())
            }
            ExportFormat::Jpeg => {
                let file = std::fs::File::create(path)?;
                let w = BufWriter::new(file);
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(w, quality);
                encoder.encode(&bytes, width, height, image::ExtendedColorType::Rgba8)?;
                Ok(())
            }
            ExportFormat::Webp => {
                let file = std::fs::File::create(path)?;
                let w = BufWriter::new(file);
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(w);
                encoder.encode(&bytes, width, height, image::ExtendedColorType::Rgba8)?;
                Ok(())
            }
            ExportFormat::Bmp => {
                let file = std::fs::File::create(path)?;
                let mut w = BufWriter::new(file);
                let encoder = image::codecs::bmp::BmpEncoder::new(&mut w);
                encoder.write_image(&bytes, width, height, image::ExtendedColorType::Rgba8)?;
                Ok(())
            }
            ExportFormat::Tiff => {
                let file = std::fs::File::create(path)?;
                let w = BufWriter::new(file);
                let encoder = image::codecs::tiff::TiffEncoder::new(w);
                encoder.write_image(&bytes, width, height, image::ExtendedColorType::Rgba8)?;
                Ok(())
            }
        }
    }

    /// Set the current image from the ImageCache (returns Some if an image is loaded)
    pub fn set_from_cache(&mut self, cache: &ImageCache, browser: &FileBrowser) -> Option<GlintImage> {
        if let Some(ref path) = browser.current() {
            if let Some(img) = cache.get(path) {
                self.original = Some(img.clone());
                self.current = Some(img.clone());
                return Some(img);
            }
        }
        None
    }

    /// Get the currently edited image (for applying edits)
    pub fn current_image(&self) -> Option<&GlintImage> {
        self.current.as_ref()
    }

    /// Get a mutable reference to the current image (for applying edits)
    pub fn current_image_mut(&mut self) -> Option<&mut GlintImage> {
        self.current.as_mut()
    }

    /// Create a tool selection button
    fn tool_button(&self, ui: &mut egui::Ui, label: &str, tool: EditorTool) -> egui::Response {
        let active = self.active_tool == tool;
        if active {
            ui.add(
                egui::Button::new(
                    egui::RichText::new(label).size(13.0).color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(70, 130, 255))
                .min_size(egui::Vec2::new(60.0, 28.0)),
            )
        } else {
            ui.add(
                egui::Button::new(egui::RichText::new(label).size(13.0))
                    .min_size(egui::Vec2::new(60.0, 28.0)),
            )
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
