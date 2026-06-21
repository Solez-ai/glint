// Glint - Thumbnail module
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

mod cache;

pub use cache::ThumbnailCache;

/// Thumbnail configuration constants
pub mod config {
    /// Default thumbnail size in pixels
    pub const DEFAULT_THUMBNAIL_SIZE: u32 = 256;
    /// Maximum thumbnail size
    pub const MAX_THUMBNAIL_SIZE: u32 = 1024;
    /// Minimum thumbnail size
    pub const MIN_THUMBNAIL_SIZE: u32 = 64;
    /// JPEG quality for thumbnail storage
    pub const THUMBNAIL_QUALITY: u8 = 85;
    /// Maximum cache entries
    pub const MAX_CACHE_ENTRIES: usize = 5000;
}
