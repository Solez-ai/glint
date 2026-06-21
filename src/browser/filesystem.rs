// Glint - File system browser
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::browser::sorting::{is_supported_image, ImageFilter, SortDirection, SortField};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

/// File browser state for navigating through images in a directory
pub struct FileBrowser {
    current_directory: Option<PathBuf>,
    files: Vec<PathBuf>,
    current_index: usize,
    sort_field: SortField,
    sort_direction: SortDirection,
    filter: ImageFilter,
    recent_directories: Vec<PathBuf>,
    bookmarks: Vec<PathBuf>,
    watcher: Option<RecommendedWatcher>,
    watcher_rx: Option<Receiver<Result<Event, notify::Error>>>,
    last_index_duration: std::time::Duration,
}

impl FileBrowser {
    pub fn new() -> Self {
        Self {
            current_directory: None,
            files: Vec::new(),
            current_index: 0,
            sort_field: SortField::Name,
            sort_direction: SortDirection::Ascending,
            filter: ImageFilter::new(),
            recent_directories: Vec::new(),
            bookmarks: Vec::new(),
            watcher: None,
            watcher_rx: None,
            last_index_duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn open_directory(&mut self, path: &Path) {
        let start = Instant::now();

        if !path.is_dir() {
            log::warn!("Not a directory: {:?}", path);
            return;
        }

        if let Some(current) = &self.current_directory {
            if current != path {
                self.recent_directories.push(current.clone());
                if self.recent_directories.len() > 20 {
                    self.recent_directories.remove(0);
                }
            }
        }

        self.current_directory = Some(path.to_path_buf());
        self.index_directory(path);

        // Setup file watcher after indexing
        self.watcher = None;
        self.watcher_rx = None;
        self.setup_watcher(path);

        self.last_index_duration = start.elapsed();
        log::info!(
            "Indexed directory {:?} ({} images) in {:?}",
            path,
            self.files.len(),
            self.last_index_duration
        );
    }

    fn index_directory(&mut self, path: &Path) {
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() && is_supported_image(&entry_path) {
                    files.push(entry_path);
                }
            }
        }

        files.sort_by(|a, b| {
            let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
            a_name.to_lowercase().cmp(&b_name.to_lowercase())
        });

        self.files = files;
        self.current_index = 0;
    }

    fn setup_watcher(&mut self, path: &Path) {
        let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

        match RecommendedWatcher::new(tx, Config::default()) {
            Ok(mut watcher) => {
                if watcher.watch(path, RecursiveMode::NonRecursive).is_ok() {
                    self.watcher = Some(watcher);
                    self.watcher_rx = Some(rx);
                    log::debug!("File watcher set up for {:?}", path);
                }
            }
            Err(e) => {
                log::warn!("Failed to set up file watcher: {}", e);
            }
        }
    }

    pub fn check_for_updates(&mut self) {
        // Use a separate scope for the immutable borrow of watcher_rx
        let mut should_reindex = false;
        let mut dir_to_index = None;
        
        if let Some(ref rx) = self.watcher_rx {
            while let Ok(Ok(event)) = rx.try_recv() {
                match event.kind {
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                        should_reindex = true;
                        dir_to_index = self.current_directory.clone();
                    }
                    _ => {}
                }
            }
        }
        
        if should_reindex {
            log::info!("File system change detected, re-indexing...");
            if let Some(dir) = dir_to_index {
                self.index_directory(&dir);
            }
        }
    }

    pub fn next(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.files.len();
        Some(self.files[self.current_index].clone())
    }

    pub fn previous(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = if self.current_index == 0 {
            self.files.len() - 1
        } else {
            self.current_index - 1
        };
        Some(self.files[self.current_index].clone())
    }

    pub fn first(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = 0;
        Some(self.files[0].clone())
    }

    pub fn last(&mut self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = self.files.len() - 1;
        Some(self.files[self.files.len() - 1].clone())
    }

    pub fn go_to(&mut self, index: usize) -> Option<PathBuf> {
        if index >= self.files.len() {
            return None;
        }
        self.current_index = index;
        Some(self.files[index].clone())
    }

    pub fn set_current_index(&mut self, index: usize) {
        if index < self.files.len() {
            self.current_index = index;
        }
    }

    /// Add a new path to the file list in sorted position
    pub fn add_path(&mut self, path: PathBuf) {
        // Only add image files
        if !crate::browser::sorting::is_supported_image(&path) {
            return;
        }
        // Don't add duplicates
        if self.files.contains(&path) {
            return;
        }
        // Insert in alphabetical order
        let new_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        let pos = self.files.partition_point(|f| {
            let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            name <= new_name
        });
        self.files.insert(pos, path);
        // If the inserted position is at or before current_index, bump it
        if pos <= self.current_index {
            self.current_index = self.current_index.saturating_add(1).min(self.files.len().saturating_sub(1));
        }
    }

    pub fn remove_paths(&mut self, paths: &[PathBuf]) {
        self.files.retain(|p| !paths.contains(p));
        if !self.files.is_empty() {
            self.current_index = self.current_index.min(self.files.len() - 1);
        } else {
            self.current_index = 0;
        }
    }

    pub fn current(&self) -> Option<PathBuf> {
        if self.files.is_empty() {
            None
        } else {
            Some(self.files[self.current_index].clone())
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn current_directory(&self) -> Option<&Path> {
        self.current_directory.as_deref()
    }

    pub fn set_sort(&mut self, field: SortField, direction: SortDirection) {
        self.sort_field = field;
        self.sort_direction = direction;
        if let Some(ref dir) = self.current_directory.clone() {
            self.index_directory(dir);
        }
    }

    pub fn set_filter(&mut self, filter: ImageFilter) {
        self.filter = filter;
        if let Some(ref dir) = self.current_directory.clone() {
            self.index_directory(dir);
        }
    }

    pub fn bookmark_current(&mut self) {
        if let Some(dir) = &self.current_directory {
            if !self.bookmarks.contains(dir) {
                self.bookmarks.push(dir.clone());
                log::info!("Bookmarked directory: {:?}", dir);
            }
        }
    }

    pub fn bookmarks(&self) -> &[PathBuf] {
        &self.bookmarks
    }

    pub fn recent_directories(&self) -> &[PathBuf] {
        &self.recent_directories
    }

    pub fn last_index_duration(&self) -> std::time::Duration {
        self.last_index_duration
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}
