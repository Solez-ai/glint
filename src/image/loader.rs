// Glint - Image loader
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::image::GlintImage;
use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Handles decoding image files from disk with format-specific optimizations
pub struct ImageLoader;

impl ImageLoader {
    /// Load an image from a file path
    pub fn load(path: &Path) -> Result<GlintImage> {
        let start = Instant::now();

        let file_size = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {:?}", path))?
            .len();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let data = Self::decode_image(path, &ext)?;

        let width = data.width();
        let height = data.height();
        let color_type = format!("{:?}", data.color());

        let elapsed = start.elapsed();
        log::info!(
            "Loaded image: {} ({}x{}, {} MB) in {:?}",
            path.display(),
            width,
            height,
            file_size / 1_000_000,
            elapsed
        );

        Ok(GlintImage {
            path: path.to_path_buf(),
            data: Arc::new(data),
            width,
            height,
            color_type,
            file_size,
        })
    }

    /// Decode an image from the given path
    fn decode_image(path: &Path, ext: &str) -> Result<DynamicImage> {
        match ext {
            "heic" | "heif" => Self::decode_heic(path),
            "svg" => Self::decode_svg(path),
            _ => Self::decode_standard(path),
        }
    }

    /// Decode standard formats using the image crate
    fn decode_standard(path: &Path) -> Result<DynamicImage> {
        let img = image::ImageReader::open(path)
            .with_context(|| format!("Failed to open image file {:?}", path))?
            .with_guessed_format()
            .with_context(|| format!("Failed to guess format for {:?}", path))?
            .decode()
            .with_context(|| format!("Failed to decode image {:?}", path))?;

        Ok(img)
    }

    /// Decode HEIC/HEIF images
    fn decode_heic(path: &Path) -> Result<DynamicImage> {
        log::info!("Attempting HEIC decode for {:?}", path);
        Self::decode_standard(path).or_else(|_| {
            Err(anyhow::anyhow!(
                "HEIC decoding not yet implemented - install libheif or convert to JPEG"
            ))
        })
    }

    /// Decode SVG images
    fn decode_svg(path: &Path) -> Result<DynamicImage> {
        let img = image::ImageReader::open(path)
            .with_context(|| format!("Failed to open SVG {:?}", path))?
            .with_guessed_format()
            .map_err(|_| anyhow::anyhow!("SVG format not supported by default image decoder"))?
            .decode()
            .map_err(|_| anyhow::anyhow!("SVG decoding requires usvg crate, falling back"))?;

        Ok(img)
    }
}
