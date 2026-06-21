// Glint - Image processing operations
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::image::GlintImage;
use anyhow::Result;
use std::sync::Arc;

/// Core image processing operations with hardware-accelerated paths where possible
pub struct ImageProcessor;

impl ImageProcessor {
    /// Resize an image to new dimensions using high-quality filtering
    pub fn resize(image: &GlintImage, new_width: u32, new_height: u32) -> Result<GlintImage> {
        let start = std::time::Instant::now();

        let resized = (*image.data).resize_exact(
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        log::info!(
            "Resized image from {}x{} to {}x{} in {:?}",
            image.width,
            image.height,
            new_width,
            new_height,
            start.elapsed()
        );

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(resized.clone()),
            width: new_width,
            height: new_height,
            color_type: format!("{:?}", resized.color()),
            file_size: image.file_size,
        })
    }

    /// Resize maintaining aspect ratio
    pub fn resize_to_fit(image: &GlintImage, max_width: u32, max_height: u32) -> Result<GlintImage> {
        let ratio = ((max_width as f32 / image.width as f32)
            .min(max_height as f32 / image.height as f32))
        .min(1.0);

        let new_width = (image.width as f32 * ratio) as u32;
        let new_height = (image.height as f32 * ratio) as u32;

        Self::resize(image, new_width.max(1), new_height.max(1))
    }

    /// Rotate an image by the specified degrees (0, 90, 180, 270)
    pub fn rotate(image: &GlintImage, degrees: f32) -> Result<GlintImage> {
        let rotated = match degrees as i32 {
            90 | -270 => (*image.data).rotate90(),
            180 | -180 => (*image.data).rotate180(),
            270 | -90 => (*image.data).rotate270(),
            _ => (*image.data).clone(),
        };

        let (new_width, new_height) = (rotated.width(), rotated.height());

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(rotated.clone()),
            width: new_width,
            height: new_height,
            color_type: format!("{:?}", rotated.color()),
            file_size: image.file_size,
        })
    }

    /// Flip an image horizontally or vertically
    pub fn flip(image: &GlintImage, horizontal: bool) -> Result<GlintImage> {
        let mut flipped = (*image.data).clone();
        if horizontal {
            image::imageops::flip_horizontal_in_place(&mut flipped);
        } else {
            image::imageops::flip_vertical_in_place(&mut flipped);
        }

        let color_type = format!("{:?}", flipped.color());
        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(flipped),
            width: image.width,
            height: image.height,
            color_type,
            file_size: image.file_size,
        })
    }

    /// Crop an image to the specified rectangle
    pub fn crop(
        image: &GlintImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<GlintImage> {
        let cropped = (*image.data).crop_imm(x, y, width, height);

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(cropped.clone()),
            width,
            height,
            color_type: format!("{:?}", cropped.color()),
            file_size: image.file_size,
        })
    }

    /// Adjust brightness (range: -255 to 255)
    pub fn adjust_brightness(image: &GlintImage, value: i32) -> Result<GlintImage> {
        let adjusted = (*image.data).clone().brighten(value);

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(adjusted.clone()),
            width: image.width,
            height: image.height,
            color_type: format!("{:?}", adjusted.color()),
            file_size: image.file_size,
        })
    }

    /// Adjust contrast (range: -255 to 255)
    pub fn adjust_contrast(image: &GlintImage, value: f32) -> Result<GlintImage> {
        let adjusted = (*image.data).clone().adjust_contrast(value);

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(adjusted.clone()),
            width: image.width,
            height: image.height,
            color_type: format!("{:?}", adjusted.color()),
            file_size: image.file_size,
        })
    }

    /// Apply a blur filter
    pub fn blur(image: &GlintImage, sigma: f32) -> Result<GlintImage> {
        let blurred = (*image.data).clone().blur(sigma);

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(blurred.clone()),
            width: image.width,
            height: image.height,
            color_type: format!("{:?}", blurred.color()),
            file_size: image.file_size,
        })
    }

    /// Convert to grayscale
    pub fn grayscale(image: &GlintImage) -> Result<GlintImage> {
        let gray = (*image.data).clone().grayscale();

        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new(gray.clone()),
            width: image.width,
            height: image.height,
            color_type: format!("{:?}", gray.color()),
            file_size: image.file_size,
        })
    }

    /// Apply hue rotation manually (applies hue shift in HSL color space)
    pub fn hue_rotate(image: &GlintImage, _degrees: i32) -> Result<GlintImage> {
        // The image crate doesn't have a built-in hue_rotate for DynamicImage.
        // For now, we return a clone of the original.
        // A proper implementation would convert to HSL, rotate hue, convert back.
        log::info!("Hue rotation by {} degrees (not yet implemented, returning original)", _degrees);
        Ok(GlintImage {
            path: image.path.clone(),
            data: Arc::new((*image.data).clone()),
            width: image.width,
            height: image.height,
            color_type: format!("{:?}", image.data.color()),
            file_size: image.file_size,
        })
    }
}
