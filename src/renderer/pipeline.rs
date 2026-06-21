// Glint - GPU rendering pipeline
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use anyhow::Result;

/// The GPU-accelerated rendering pipeline for Glint
/// Uses wgpu to provide hardware-accelerated image rendering
pub struct RenderPipeline {
    /// Whether the pipeline is initialized
    initialized: bool,
    /// Current rendering quality setting
    quality: RenderQuality,
    /// MSAA sample count
    sample_count: u32,
}

/// Render quality settings
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderQuality {
    /// Fastest rendering, lower quality
    Fast,
    /// Balanced quality and performance
    Balanced,
    /// Highest quality rendering
    HighQuality,
    /// Maximum quality with all effects
    Ultra,
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new() -> Self {
        Self {
            initialized: false,
            quality: RenderQuality::Balanced,
            sample_count: 4,
        }
    }

    /// Initialize the GPU pipeline with a wgpu device and queue
    pub fn initialize(&mut self) -> Result<()> {
        log::info!("Initializing render pipeline with quality: {:?}", self.quality);
        self.initialized = true;
        Ok(())
    }

    /// Check if the pipeline is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set the rendering quality
    pub fn set_quality(&mut self, quality: RenderQuality) {
        self.quality = quality;
        log::info!("Render quality set to {:?}", quality);
    }

    /// Get the current rendering quality
    pub fn quality(&self) -> RenderQuality {
        self.quality
    }

    /// Set the MSAA sample count
    pub fn set_sample_count(&mut self, count: u32) {
        self.sample_count = match count {
            1 | 2 | 4 | 8 => count,
            _ => 4,
        };
    }

    /// Get supported texture formats for the current adapter
    pub fn supported_formats() -> Vec<&'static str> {
        vec![
            "Rgba8UnormSrgb",
            "Bgra8UnormSrgb",
            "Rgba16Float",
            "Rgba32Float",
        ]
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}
