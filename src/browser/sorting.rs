// Glint - File sorting and filtering
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use std::path::Path;

/// Sort field for image browsing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    Size,
    Date,
    Resolution,
    ColorProfile,
    Random,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Image filter criteria
#[derive(Debug, Clone)]
pub struct ImageFilter {
    /// Filter by file type (extension)
    pub file_types: Vec<String>,
    /// Filter by minimum width
    pub min_width: Option<u32>,
    /// Filter by maximum width
    pub max_width: Option<u32>,
    /// Filter by minimum height
    pub min_height: Option<u32>,
    /// Filter by maximum height
    pub max_height: Option<u32>,
    /// Filter by orientation
    pub orientation: Option<Orientation>,
    /// Filter by date range (start timestamp)
    pub date_from: Option<i64>,
    /// Filter by date range (end timestamp)
    pub date_to: Option<i64>,
    /// Search query for filename
    pub search_query: Option<String>,
}

/// Image orientation filter
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Landscape,
    Portrait,
    Square,
    Panoramic,
}

impl ImageFilter {
    /// Create a default (no filter) filter
    pub fn new() -> Self {
        Self {
            file_types: Vec::new(),
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            orientation: None,
            date_from: None,
            date_to: None,
            search_query: None,
        }
    }

    /// Apply the filter to a list of file paths
    pub fn apply(&self, paths: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        paths
            .into_iter()
            .filter(|path| self.matches(path))
            .collect()
    }

    /// Check if a file path matches all filter criteria
    fn matches(&self, path: &Path) -> bool {
        // File type filter
        if !self.file_types.is_empty() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if !self.file_types.contains(&ext) {
                return false;
            }
        }

        // Search query filter
        if let Some(query) = &self.search_query {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !filename.to_lowercase().contains(&query.to_lowercase()) {
                return false;
            }
        }

        true
    }
}

impl Default for ImageFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Supported image file extensions for browsing
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif",
    "webp", "ico", "avif", "heic", "heif", "svg",
    "cr2", "cr3", "nef", "arw", "dng", "raf", "orf", "rw2", "pef", "raw",
    "qoi", "exr", "hdr", "dds", "tga",
];

/// Check if a file extension is a supported image format
pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}
