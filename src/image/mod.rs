// Glint - Image module
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

mod cache;
mod loader;
mod processing;

pub use cache::ImageCache;
pub use loader::ImageLoader;
pub use processing::ImageProcessor;

use image::DynamicImage;
use std::path::PathBuf;
use std::sync::Arc;

/// Represents a loaded image in the application
#[derive(Clone)]
pub struct GlintImage {
    /// Path to the image file
    pub path: PathBuf,
    /// The decoded image data
    pub data: Arc<DynamicImage>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Color type description
    pub color_type: String,
    /// File size in bytes
    pub file_size: u64,
}

impl GlintImage {
    /// Calculate the aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Get the megapixel count
    pub fn megapixels(&self) -> f32 {
        (self.width as f64 * self.height as f64 / 1_000_000.0) as f32
    }

    /// Check if the image has an alpha channel
    pub fn has_alpha(&self) -> bool {
        let color = self.data.color();
        color.has_alpha()
    }
}

/// Supported image file formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
    Svg,
    Ico,
    Avif,
    Heic,
    Heif,
    Raw,
    Qoi,
    Exr,
    Hdr,
    Dds,
    Tga,
    PngPreview,
}

impl ImageFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Webp => "webp",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Svg => "svg",
            ImageFormat::Ico => "ico",
            ImageFormat::Avif => "avif",
            ImageFormat::Heic => "heic",
            ImageFormat::Heif => "heif",
            ImageFormat::Raw => "raw",
            ImageFormat::Qoi => "qoi",
            ImageFormat::Exr => "exr",
            ImageFormat::Hdr => "hdr",
            ImageFormat::Dds => "dds",
            ImageFormat::Tga => "tga",
            ImageFormat::PngPreview => "psd",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            "webp" => Some(ImageFormat::Webp),
            "gif" => Some(ImageFormat::Gif),
            "bmp" => Some(ImageFormat::Bmp),
            "tiff" | "tif" => Some(ImageFormat::Tiff),
            "svg" => Some(ImageFormat::Svg),
            "ico" => Some(ImageFormat::Ico),
            "avif" => Some(ImageFormat::Avif),
            "heic" => Some(ImageFormat::Heic),
            "heif" => Some(ImageFormat::Heif),
            "cr2" | "cr3" | "nef" | "arw" | "dng" | "raf" | "orf" | "rw2" | "pef" | "raw" => {
                Some(ImageFormat::Raw)
            }
            "qoi" => Some(ImageFormat::Qoi),
            "exr" => Some(ImageFormat::Exr),
            "hdr" => Some(ImageFormat::Hdr),
            "dds" => Some(ImageFormat::Dds),
            "tga" => Some(ImageFormat::Tga),
            "psd" => Some(ImageFormat::PngPreview),
            _ => None,
        }
    }
}
