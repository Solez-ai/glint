// Glint - Image cache
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::image::GlintImage;
use anyhow::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Thread-safe LRU cache for decoded images with smart memory management
pub struct ImageCache {
    /// Internal LRU cache (uses Mutex for interior mutability)
    cache: Mutex<LruCache<PathBuf, GlintImage>>,
    /// Maximum memory usage in bytes (approximate)
    max_memory_bytes: Mutex<u64>,
    /// Current estimated memory usage
    current_memory: Mutex<u64>,
}

impl ImageCache {
    /// Create a new image cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        let non_zero = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(50).unwrap());
        Self {
            cache: Mutex::new(LruCache::new(non_zero)),
            max_memory_bytes: Mutex::new(512 * 1024 * 1024), // 512 MB default
            current_memory: Mutex::new(0),
        }
    }

    /// Load an image into the cache
    pub fn load(&self, path: &PathBuf) -> Result<()> {
        let image = super::loader::ImageLoader::load(path)?;
        let image_size = (image.width as u64) * (image.height as u64) * 4;

        let mut cache = self.cache.lock().unwrap();
        let mut memory = self.current_memory.lock().unwrap();
        let max_mem = *self.max_memory_bytes.lock().unwrap();

        // If the path already exists in the cache, subtract the old entry's size first
        if let Some(old) = cache.peek(path) {
            let old_size = (old.width as u64) * (old.height as u64) * 4;
            *memory = memory.saturating_sub(old_size);
        }

        // Evict entries if we're over the memory limit
        while *memory + image_size > max_mem && !cache.is_empty() {
            if let Some((evicted_path, evicted)) = cache.pop_lru() {
                let evicted_size = (evicted.width as u64) * (evicted.height as u64) * 4;
                *memory = memory.saturating_sub(evicted_size);
                log::debug!("Evicted cached image: {:?}", evicted_path);
            }
        }

        *memory += image_size;
        cache.put(path.clone(), image);
        log::debug!(
            "Cached image: {:?} (cache size: {}, memory: {} MB)",
            path,
            cache.len(),
            *memory / 1024 / 1024
        );
        Ok(())
    }

    /// Get an image from the cache
    pub fn get(&self, path: &PathBuf) -> Option<GlintImage> {
        let mut cache = self.cache.lock().unwrap();
        cache.get(path).cloned()
    }

    /// Check if an image is cached
    pub fn contains(&self, path: &PathBuf) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.contains(path)
    }

    /// Invalidate a specific image from the cache
    pub fn invalidate(&self, path: &PathBuf) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(removed) = cache.pop(path) {
            let mut memory = self.current_memory.lock().unwrap();
            let removed_size = (removed.width as u64) * (removed.height as u64) * 4;
            *memory = memory.saturating_sub(removed_size);
            log::debug!("Invalidated cache entry: {:?}", path);
        }
    }

    /// Clear the entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        let mut memory = self.current_memory.lock().unwrap();
        *memory = 0;
        log::info!("Image cache cleared");
    }

    /// Get the current number of cached images
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Insert a GlintImage directly into the cache (for processed editor results)
    pub fn insert(&self, path: PathBuf, image: GlintImage) {
        let image_size = (image.width as u64) * (image.height as u64) * 4;
        let mut cache = self.cache.lock().unwrap();
        let mut memory = self.current_memory.lock().unwrap();

        // If the path already exists, subtract old entry's size
        if let Some(old) = cache.peek(&path) {
            let old_size = (old.width as u64) * (old.height as u64) * 4;
            *memory = memory.saturating_sub(old_size);
        }

        *memory += image_size;
        cache.put(path, image);
    }

    /// Set the maximum memory usage (in MB)
    pub fn set_max_memory(&self, max_mb: u64) {
        let mut max_mem = self.max_memory_bytes.lock().unwrap();
        *max_mem = max_mb * 1024 * 1024;
    }

    /// Get the current estimated memory usage in MB
    pub fn current_memory_mb(&self) -> u64 {
        let memory = self.current_memory.lock().unwrap();
        *memory / 1024 / 1024
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }
}
