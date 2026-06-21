// Glint - SQLite-backed thumbnail cache
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use anyhow::{Context, Result};
use egui::ColorImage;
use image::ImageEncoder;
use rusqlite::Connection;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// SQLite-backed persistent thumbnail cache for instant folder browsing
pub struct ThumbnailCache {
    db: Mutex<Connection>,
    hits: Mutex<u64>,
    misses: Mutex<u64>,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("glint")
            .join("thumbnails");

        std::fs::create_dir_all(&cache_dir).ok();

        let db_path = cache_dir.join("thumbnails.db");
        let db = Connection::open(&db_path).unwrap_or_else(|_| {
            Connection::open_in_memory().unwrap()
        });

        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS thumbnails (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL UNIQUE,
                thumbnail_data BLOB NOT NULL,
                thumbnail_size INTEGER NOT NULL,
                file_modified INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                image_width INTEGER NOT NULL,
                image_height INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_thumbnails_path ON thumbnails(file_path);
            CREATE INDEX IF NOT EXISTS idx_thumbnails_accessed ON thumbnails(accessed_at);",
        ).ok();

        Self {
            db: Mutex::new(db),
            hits: Mutex::new(0),
            misses: Mutex::new(0),
        }
    }

    pub fn get_or_generate(&self, path: &Path, size: u32) -> Result<Option<ColorImage>> {
        if let Some(cached) = self.get_from_cache(path, size)? {
            let mut hits = self.hits.lock().unwrap();
            *hits += 1;
            log::debug!("Thumbnail cache HIT for {:?}", path);
            return Ok(Some(cached));
        }

        let mut misses = self.misses.lock().unwrap();
        *misses += 1;

        let thumbnail = self.generate_thumbnail(path, size)?;
        if let Some(ref thumb) = thumbnail {
            self.store_in_cache(path, thumb, size)?;
        }

        Ok(thumbnail)
    }

    fn get_from_cache(&self, path: &Path, _size: u32) -> Result<Option<ColorImage>> {
        let db = self.db.lock().unwrap();
        let path_str = path.to_string_lossy().to_string();

        let file_modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let file_size = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);

        let mut stmt = match db.prepare(
            "SELECT thumbnail_data, thumbnail_size, image_width, image_height
             FROM thumbnails
             WHERE file_path = ?1 AND file_modified = ?2 AND file_size = ?3
             LIMIT 1",
        ) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Failed to prepare thumbnail query: {}", e);
                return Ok(None);
            }
        };

        let result = stmt.query_row(
            rusqlite::params![path_str, file_modified, file_size],
            |row| {
                let data: Vec<u8> = row.get(0)?;
                let thumb_size: i32 = row.get(1)?;
                let width: i32 = row.get(2)?;
                let height: i32 = row.get(3)?;
                Ok((data, thumb_size as u32, width as u32, height as u32))
            },
        );

        match result {
            Ok((data, _thumb_size, width, height)) => {
                let img = image::load_from_memory(&data)
                    .context("Failed to decode cached thumbnail")?;
                let rgba = img.to_rgba8();
                let pixels = rgba.into_raw();

                Ok(Some(ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &pixels,
                )))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => {
                log::warn!("Thumbnail cache query error: {}", e);
                Ok(None)
            }
        }
    }

    fn generate_thumbnail(&self, path: &Path, size: u32) -> Result<Option<ColorImage>> {
        let img = match image::ImageReader::open(path) {
            Ok(reader) => match reader.with_guessed_format() {
                Ok(decoder) => match decoder.decode() {
                    Ok(img) => img,
                    Err(e) => {
                        log::warn!("Failed to decode thumbnail for {:?}: {}", path, e);
                        return Ok(None);
                    }
                },
                Err(e) => {
                    log::warn!("Failed to guess format for thumbnail {:?}: {}", path, e);
                    return Ok(None);
                }
            },
            Err(e) => {
                log::warn!("Failed to open image for thumbnail {:?}: {}", path, e);
                return Ok(None);
            }
        };

        let thumbnail = img.thumbnail(size, size);
        let rgba = thumbnail.to_rgba8();
        let pixels = rgba.into_raw();
        let (width, height) = (thumbnail.width(), thumbnail.height());

        Ok(Some(ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            &pixels,
        )))
    }

    fn store_in_cache(&self, path: &Path, thumbnail: &ColorImage, _size: u32) -> Result<()> {
        let db = self.db.lock().unwrap();
        let path_str = path.to_string_lossy().to_string();

        let file_modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let file_size = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);

        // Encode thumbnail as PNG bytes for storage
        let encoded = {
            let mut bytes: Vec<u8> = Vec::new();
            {
                let writer = BufWriter::new(&mut bytes);
                let encoder = image::codecs::png::PngEncoder::new(writer);
                if let Err(e) = encoder.write_image(
                    bytemuck::cast_slice(thumbnail.pixels.as_ref()),
                    thumbnail.width() as u32,
                    thumbnail.height() as u32,
                    image::ExtendedColorType::Rgba8,
                ) {
                    log::warn!("Failed to encode thumbnail for caching: {}", e);
                    return Ok(());
                }
            }
            bytes
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if let Err(e) = db.execute(
            "INSERT OR REPLACE INTO thumbnails
             (file_path, thumbnail_data, thumbnail_size, file_modified, file_size,
              image_width, image_height, created_at, accessed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                path_str,
                encoded,
                thumbnail.width() as i32,
                file_modified,
                file_size,
                thumbnail.width() as i32,
                thumbnail.height() as i32,
                now,
                now,
            ],
        ) {
            log::warn!("Failed to store thumbnail in cache: {}", e);
        }

        log::debug!("Stored thumbnail for {:?} in cache", path);
        Ok(())
    }

    pub fn stats(&self) -> (u64, u64) {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        (hits, misses)
    }

    pub fn hit_ratio(&self) -> f64 {
        let (hits, misses) = self.stats();
        let total = hits + misses;
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }

    pub fn clear(&self) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM thumbnails", [])?;
        db.execute("VACUUM", [])?;
        log::info!("Thumbnail cache cleared");
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT COUNT(*) FROM thumbnails", [], |row| row.get(0))
            .map_err(|e| e.into())
    }

    pub fn prune(&self, max_entries: usize) -> Result<i64> {
        let db = self.db.lock().unwrap();
        let count = db.execute(
            "DELETE FROM thumbnails WHERE id NOT IN (
                SELECT id FROM thumbnails ORDER BY accessed_at DESC LIMIT ?1
            )",
            rusqlite::params![max_entries as i64],
        )?;
        db.execute("VACUUM", [])?;
        log::info!("Pruned {} old thumbnail entries", count);
        Ok(count as i64)
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new()
    }
}
