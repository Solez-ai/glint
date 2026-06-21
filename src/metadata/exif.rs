// Glint - EXIF metadata reader
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use anyhow::{Context, Result};
use exif::{Exif, In, Reader, Tag};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Structured EXIF/metadata for an image
#[derive(Debug, Clone)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<f64>,
    pub aperture: Option<f64>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub date_taken: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub orientation: Option<u32>,
    pub color_space: Option<String>,
    pub flash: Option<bool>,
    pub software: Option<String>,
    pub icc_profile: Option<String>,
    pub raw: HashMap<String, String>,
}

impl ExifData {
    /// Parse EXIF data from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {} for EXIF", path.display()))?;
        let mut bufreader = std::io::BufReader::new(&file);
        let exif = Reader::new().read_from_container(&mut bufreader)
            .context("Failed to read EXIF data")?;

        let camera_make = exif.get_field(Tag::Make, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into());
        let camera_model = exif.get_field(Tag::Model, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into());
        let lens_model = exif.get_field(Tag::LensModel, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().into());
        let focal_length = exif.get_field(Tag::FocalLength, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<f64>().ok());
        let aperture = exif.get_field(Tag::FNumber, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<f64>().ok());
        let shutter_speed = exif.get_field(Tag::ExposureTime, In::PRIMARY)
            .map(|f| f.display_value().to_string());
        let iso = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<u32>().ok());
        let date_taken = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
            .map(|f| f.display_value().to_string());
        let gps_latitude = parse_gps_coordinate(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef, "N");
        let gps_longitude = parse_gps_coordinate(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef, "E");
        let gps_altitude = exif.get_field(Tag::GPSAltitude, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<f64>().ok());
        let image_width = exif.get_field(Tag::ImageWidth, In::PRIMARY)
            .or_else(|| exif.get_field(Tag::PixelXDimension, In::PRIMARY))
            .and_then(|f| f.display_value().to_string().parse::<u32>().ok());
        let image_height = exif.get_field(Tag::ImageLength, In::PRIMARY)
            .or_else(|| exif.get_field(Tag::PixelYDimension, In::PRIMARY))
            .and_then(|f| f.display_value().to_string().parse::<u32>().ok());
        let orientation = exif.get_field(Tag::Orientation, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<u32>().ok());
        let color_space = exif.get_field(Tag::ColorSpace, In::PRIMARY)
            .map(|f| f.display_value().to_string());
        let flash = exif.get_field(Tag::Flash, In::PRIMARY)
            .and_then(|f| f.display_value().to_string().parse::<u32>().ok())
            .map(|v| v & 0x01 != 0);
        let software = exif.get_field(Tag::Software, In::PRIMARY)
            .map(|f| f.display_value().to_string());

        log::debug!("EXIF parsed for {:?}", path);

        Ok(Self {
            camera_make,
            camera_model,
            lens_model,
            focal_length,
            aperture,
            shutter_speed,
            iso,
            date_taken,
            gps_latitude,
            gps_longitude,
            gps_altitude,
            image_width,
            image_height,
            orientation,
            color_space,
            flash,
            software,
            icc_profile: None,
            raw: HashMap::new(),
        })
    }

    pub fn has_gps(&self) -> bool {
        self.gps_latitude.is_some() && self.gps_longitude.is_some()
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref make) = self.camera_make {
            if let Some(ref model) = self.camera_model {
                parts.push(format!("{} {}", make, model));
            } else {
                parts.push(make.clone());
            }
        }

        if let Some(iso) = self.iso {
            parts.push(format!("ISO {}", iso));
        }

        if let Some(fl) = self.focal_length {
            parts.push(format!("{}mm", fl));
        }

        if let Some(ap) = self.aperture {
            parts.push(format!("f/{}", ap));
        }

        if parts.is_empty() {
            "No EXIF data".to_string()
        } else {
            parts.join(" | ")
        }
    }
}

/// Parse a GPS coordinate from EXIF rational + ref fields
fn parse_gps_coordinate(
    exif: &Exif,
    value_tag: Tag,
    ref_tag: Tag,
    positive_ref: &str,
) -> Option<f64> {
    let value = exif.get_field(value_tag, In::PRIMARY)?;
    // GPS coordinates are stored as three rational values [degrees, minutes, seconds]
    let parts: Vec<f64> = value.display_value().to_string()
        .split(&[',', ' '][..])
        .filter_map(|s| s.trim().parse::<f64>().ok())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let mut coord = parts[0] + parts[1] / 60.0 + parts[2] / 3600.0;
    // Apply sign based on reference (S/W are negative)
    if let Some(ref_val) = exif.get_field(ref_tag, In::PRIMARY) {
        let ref_str = ref_val.display_value().to_string();
        if !ref_str.trim().eq_ignore_ascii_case(positive_ref) {
            coord = -coord;
        }
    }
    Some(coord)
}

impl Default for ExifData {
    fn default() -> Self {
        Self {
            camera_make: None,
            camera_model: None,
            lens_model: None,
            focal_length: None,
            aperture: None,
            shutter_speed: None,
            iso: None,
            date_taken: None,
            gps_latitude: None,
            gps_longitude: None,
            gps_altitude: None,
            image_width: None,
            image_height: None,
            orientation: None,
            color_space: None,
            flash: None,
            software: None,
            icc_profile: None,
            raw: HashMap::new(),
        }
    }
}

/// Thread-safe cache for image metadata
pub struct MetadataCache {
    cache: Mutex<HashMap<PathBuf, ExifData>>,
}

impl MetadataCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn load(&self, path: &Path) -> Result<ExifData> {
        let data = ExifData::from_path(path)?;
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(path.to_path_buf(), data.clone());
        }
        Ok(data)
    }

    pub fn get(&self, path: &Path) -> Option<ExifData> {
        self.cache.lock().ok()?.get(path).cloned()
    }

    pub fn invalidate(&self, path: &Path) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(path);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}
