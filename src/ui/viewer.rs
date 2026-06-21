// Glint - Image viewer panel
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::image::ImageCache;
use crate::thumbnail::ThumbnailCache;
use egui::{Color32, Context, Pos2, Rect, Sense, TextureHandle, TextureId, Ui, Vec2};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The main image viewer panel that renders the current image
pub struct ViewerPanel {
    current_image: Option<PathBuf>,
    zoom: f32,
    pan_offset: Vec2,
    fit_to_view: bool,
    image_size: Vec2,
    filename: String,
    has_image: bool,
    // Gallery landing page state
    gallery_items: Vec<PathBuf>,
    selected_gallery_index: Option<usize>,
    pending_gallery_open: Option<PathBuf>,
    pending_folder_open: Option<PathBuf>,
    scroll_offset: f32,
    // Thumbnail textures for gallery view
    thumbnail_textures: HashMap<PathBuf, TextureHandle>,
    /// Maximum number of cached thumbnail textures
    max_textures: usize,
    /// Multi-selection: set of selected indices
    selected_indices: HashSet<usize>,
    /// Last clicked index (for Shift+click range selection)
    last_clicked_index: Option<usize>,
    /// Drag reorder state: index being dragged
    drag_source_index: Option<usize>,
    /// Drag reorder: target index where item would be dropped
    drag_target_index: Option<usize>,
    /// Pending file operations (context menu)
    pending_delete: Option<Vec<PathBuf>>,
    /// Pending move-to operations (context menu)
    pending_move_to: Option<Vec<PathBuf>>,
    /// Context menu state: whether to show context menu popup
    context_menu_open: bool,
    /// Context menu position
    context_menu_pos: egui::Pos2,
    /// Thumbnail zoom slider value (0.5–2.0)
    thumb_zoom: f32,
    /// Pending navigation actions (set by viewer, consumed by app)
    pending_nav_prev: bool,
    pending_nav_next: bool,
    /// Cached egui texture handle for the currently displayed photo
    current_image_texture: Option<TextureHandle>,
    /// Filmstrip horizontal scroll offset
    filmstrip_offset: f32,
    /// Whether the filmstrip is hidden (toggled by user)
    filmstrip_hidden: bool,
}

impl ViewerPanel {
    pub fn new() -> Self {
        Self {
            current_image: None,
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            fit_to_view: true,
            image_size: Vec2::ZERO,
            filename: String::new(),
            has_image: false,
            gallery_items: Vec::new(),
            selected_gallery_index: None,
            pending_gallery_open: None,
            pending_folder_open: None,
            scroll_offset: 0.0,
            thumbnail_textures: HashMap::new(),
            max_textures: 500,
            selected_indices: HashSet::new(),
            last_clicked_index: None,
            drag_source_index: None,
            drag_target_index: None,
            pending_delete: None,
            pending_move_to: None,
            context_menu_open: false,
            context_menu_pos: egui::Pos2::ZERO,
            thumb_zoom: 1.0,
            pending_nav_prev: false,
            pending_nav_next: false,
            current_image_texture: None,
            filmstrip_offset: 0.0,
            filmstrip_hidden: false,
        }
    }

    pub fn set_image(&mut self, path: PathBuf) {
        self.filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        self.current_image = Some(path);
        self.has_image = true;
        self.fit_to_view = true;
        self.zoom = 1.0;
        self.pan_offset = Vec2::ZERO;
        // Force texture re-creation on next render
        self.current_image_texture = None;
        // Reset image_size — will be set when image loads from cache
        self.image_size = Vec2::ZERO;
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn image_width(&self) -> u32 {
        self.image_size.x as u32
    }

    pub fn image_height(&self) -> u32 {
        self.image_size.y as u32
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.01, 100.0);
    }

    pub fn set_fit_to_view(&mut self, fit: bool) {
        self.fit_to_view = fit;
    }

    pub fn set_pan_offset(&mut self, offset: Vec2) {
        self.pan_offset = offset;
    }

    pub fn fit_to_view(&self) -> bool {
        self.fit_to_view
    }

    pub fn current_filename(&self) -> &str {
        if self.has_image {
            &self.filename
        } else {
            "No image loaded"
        }
    }

    pub fn has_image(&self) -> bool {
        self.has_image
    }

    /// Clear cached thumbnail textures (call when gallery items change)
    pub fn clear_thumbnail_cache(&mut self) {
        self.thumbnail_textures.clear();
    }

    /// Set the gallery items shown in the landing page
    pub fn set_gallery_items(&mut self, items: Vec<PathBuf>) {
        self.gallery_items = items;
        self.clear_thumbnail_cache();
        self.clear_selection();
        if self.selected_gallery_index.is_none() && !self.gallery_items.is_empty() {
            self.selected_gallery_index = Some(0);
        }
    }

    /// Clear all selections
    fn clear_selection(&mut self) {
        self.selected_indices.clear();
        self.last_clicked_index = None;
        self.drag_source_index = None;
        self.drag_target_index = None;
        self.context_menu_open = false;
    }

    /// Take pending delete operations
    pub fn take_pending_delete(&mut self) -> Option<Vec<PathBuf>> {
        self.pending_delete.take()
    }

    /// Take pending move-to operations
    pub fn take_pending_move_to(&mut self) -> Option<Vec<PathBuf>> {
        self.pending_move_to.take()
    }

    /// Get a copy of selected paths
    pub fn selected_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self.selected_indices
            .iter()
            .filter_map(|i| self.gallery_items.get(*i))
            .cloned()
            .collect();
        paths.sort_by_key(|p| {
            self.gallery_items.iter().position(|x| x == p)
        });
        paths
    }

    /// Handle gallery item click with modifier keys
    fn handle_gallery_click(&mut self, index: usize, ctrl: bool, shift: bool) {
        if ctrl {
            // Ctrl+click: toggle this item in selection
            if self.selected_indices.contains(&index) {
                self.selected_indices.remove(&index);
                // If nothing selected, fall back to single selection
                if self.selected_indices.is_empty() {
                    self.selected_gallery_index = None;
                }
            } else {
                self.selected_indices.insert(index);
                self.selected_gallery_index = Some(index);
            }
            self.last_clicked_index = Some(index);
        } else if shift {
            // Shift+click: select range from last_clicked_index to index
            let range_start = self.last_clicked_index.unwrap_or(index);
            let (lo, hi) = if range_start <= index {
                (range_start, index)
            } else {
                (index, range_start)
            };
            self.selected_indices.clear();
            for i in lo..=hi {
                self.selected_indices.insert(i);
            }
            self.selected_gallery_index = Some(index);
            self.last_clicked_index = Some(index);
        } else {
            // Simple click: select only this item
            if self.selected_indices.len() > 1 {
                // Clear multi-select, select just this one
                self.selected_indices.clear();
            }
            self.selected_indices.insert(index);
            self.selected_gallery_index = Some(index);
            self.last_clicked_index = Some(index);
            // Open image on single click
            if let Some(item) = self.gallery_items.get(index) {
                self.pending_gallery_open = Some(item.clone());
            }
        }
    }

    /// Remove deleted paths from gallery_items and clear selection
    pub fn remove_paths(&mut self, paths: &[PathBuf]) {
        self.gallery_items.retain(|p| !paths.contains(p));
        self.clear_selection();
        self.clear_thumbnail_cache();
    }

    /// Render the folder sidebar (directory tree, bookmarks, recent)
    pub fn render_sidebar(&mut self, ui: &mut egui::Ui, file_browser: &crate::browser::FileBrowser) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(160.0);

            // Title
            ui.label(
                egui::RichText::new("Browse").size(14.0).color(egui::Color32::from_rgb(200, 200, 200)),
            );
            ui.separator();

            // Quick access section
            ui.label(
                egui::RichText::new("Quick Access").size(11.0).color(egui::Color32::from_rgb(120, 120, 120)),
            );
            ui.add_space(2.0);

            let quick_folders: [(&str, fn() -> Option<PathBuf>); 3] = [
                ("[*]  Pictures", dirs::picture_dir as fn() -> Option<PathBuf>),
                ("[-]  Desktop", dirs::desktop_dir as fn() -> Option<PathBuf>),
                ("[~]  Downloads", dirs::download_dir as fn() -> Option<PathBuf>),
            ];

            for (label, dir_fn) in &quick_folders {
                if let Some(path) = dir_fn() {
                    let is_active = file_browser.current_directory()
                        .map(|d| d == path.as_path())
                        .unwrap_or(false);
                    let fill = if is_active {
                        Some(Color32::from_rgb(50, 90, 180))
                    } else {
                        None
                    };
                    let response = egui::Frame::NONE
                        .fill(fill.unwrap_or(Color32::from_rgba_premultiplied(0, 0, 0, 0)))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(*label).size(12.0).color(Color32::from_rgb(180, 180, 180))
                            )
                        });
                    if response.response.clicked() {
                        self.pending_folder_open = Some(path);
                    }
                }
            }

            ui.add_space(12.0);

            // Bookmarks section
            let bookmarks = file_browser.bookmarks();
            if !bookmarks.is_empty() {
                ui.label(
                    egui::RichText::new("Bookmarks").size(11.0).color(Color32::from_rgb(120, 120, 120)),
                );
                ui.add_space(2.0);

                for bookmark in bookmarks {
                    let name = bookmark.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Folder");
                    let label = format!("[*]  {}", name);
                    if ui.add(
                        egui::Button::new(egui::RichText::new(&label).size(12.0).color(Color32::from_rgb(180, 180, 180)))
                            .frame(false)
                            .min_size(egui::Vec2::new(ui.available_width(), 24.0)),
                    ).clicked() {
                        self.pending_folder_open = Some(bookmark.clone());
                    }
                }
                ui.add_space(12.0);
            }

            // Recent directories section
            let recent = file_browser.recent_directories();
            if !recent.is_empty() {
                ui.label(
                    egui::RichText::new("Recent").size(11.0).color(Color32::from_rgb(120, 120, 120)),
                );
                ui.add_space(2.0);

                let show_recent = recent.len().min(10);
                for dir in recent.iter().rev().take(show_recent) {
                    let name = dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Folder");
                    let label = format!("[~]  {}", name);
                    if ui.add(
                        egui::Button::new(egui::RichText::new(&label).size(11.0).color(Color32::from_rgb(160, 160, 160)))
                            .frame(false)
                            .min_size(egui::Vec2::new(ui.available_width(), 20.0)),
                    ).clicked() {
                        self.pending_folder_open = Some(dir.clone());
                    }
                }
                ui.add_space(12.0);
            }

            // Bookmark current folder button
            if file_browser.current_directory().is_some() {
                ui.separator();
                if ui.add(
                    egui::Button::new(egui::RichText::new("[+]  Bookmark current folder").size(11.0))
                        .frame(false)
                        .min_size(egui::Vec2::new(ui.available_width(), 24.0)),
                ).clicked() {
                    // TODO: send bookmark message
                    log::info!("Bookmark clicked");
                }
            }
        });
    }

    /// Take a pending gallery thumbnail click to open an image file
    pub fn take_pending_gallery_open(&mut self) -> Option<PathBuf> {
        self.pending_gallery_open.take()
    }

    /// Take a pending folder click to open a directory
    pub fn take_pending_folder_open(&mut self) -> Option<PathBuf> {
        self.pending_folder_open.take()
    }

    /// Number of gallery items
    pub fn gallery_count(&self) -> usize {
        self.gallery_items.len()
    }

    /// Get a reference to the selected indices set
    pub fn selected_indices(&self) -> &HashSet<usize> {
        &self.selected_indices
    }

    /// Take pending navigation (consumed by app.rs to dispatch messages)
    pub fn take_pending_nav_prev(&mut self) -> bool {
        std::mem::take(&mut self.pending_nav_prev)
    }

    pub fn take_pending_nav_next(&mut self) -> bool {
        std::mem::take(&mut self.pending_nav_next)
    }

    /// Render the viewer panel with optional thumbnail cache for gallery and image cache for photos
    pub fn render(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        thumbnail_cache: Option<&ThumbnailCache>,
        image_cache: Option<&ImageCache>,
    ) {
        if !self.has_image {
            self.render_gallery_landing(ctx, ui, thumbnail_cache);
            return;
        }

        let available = ui.available_size();

        // Handle mouse events for zoom and pan (uses ui.interact, not painting)
        self.handle_input(ctx, ui);

        let viewport_rect = ui.max_rect();

        // === PHASE 1: INTERACTION — allocate all interactive rects ===

        // Left navigation zone (35% width, click to go prev)
        let left_zone = Rect::from_min_size(
            viewport_rect.min,
            Vec2::new(viewport_rect.width() * 0.35, viewport_rect.height()),
        );
        let left_resp = ui.allocate_rect(left_zone, Sense::click());
        let left_hovered = left_resp.hovered();
        if left_resp.clicked() {
            self.pending_nav_prev = true;
        }

        // Right navigation zone (35% width from right edge, click to go next)
        let right_zone = Rect::from_min_size(
            egui::pos2(
                viewport_rect.max.x - viewport_rect.width() * 0.35,
                viewport_rect.min.y,
            ),
            Vec2::new(viewport_rect.width() * 0.35, viewport_rect.height()),
        );
        let right_resp = ui.allocate_rect(right_zone, Sense::click());
        let right_hovered = right_resp.hovered();
        if right_resp.clicked() {
            self.pending_nav_next = true;
        }

        // "Gallery" button at top-left
        let gallery_btn_rect = Rect::from_min_size(
            egui::pos2(viewport_rect.min.x + 16.0, viewport_rect.min.y + 16.0),
            Vec2::new(100.0, 30.0),
        );
        let gallery_btn_resp = ui.allocate_rect(gallery_btn_rect, Sense::click());
        let btn_hovered = gallery_btn_resp.hovered();
        if gallery_btn_resp.clicked() {
            self.has_image = false;
            self.current_image = None;
        }

        // === PHASE 2: PAINT — draw all visuals ===

        // Before computing display_rect, try to load the image from cache so
        // image_size is populated with real dimensions (no first-frame snap)
        if self.image_size == Vec2::ZERO {
            if let (Some(ref cur_path), Some(cache)) = (&self.current_image, image_cache) {
                if let Some(glint_image) = cache.get(cur_path) {
                    self.image_size = Vec2::new(
                        glint_image.width as f32,
                        glint_image.height as f32,
                    );
                }
            }
        }

        // Calculate the image display rectangle (now uses correct image_size)
        let display_rect = self.calculate_display_rect(available);

        // Draw the checkerboard background
        self.render_checkerboard(ui, display_rect);

        let painter = ui.painter();

        // Draw the actual image from the ImageCache (if available)
        if let Some(ref cur_path) = self.current_image {
            if let Some(cache) = image_cache {
                // Check if we already have a cached texture
                let needs_texture = self.current_image_texture.is_none();

                if needs_texture {
                    if let Some(glint_image) = cache.get(cur_path) {
                        // Convert DynamicImage to egui ColorImage
                        let rgba = glint_image.data.to_rgba8();
                        let width = rgba.width() as usize;
                        let height = rgba.height() as usize;
                        let pixels = rgba.into_vec();

                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [width, height],
                            &pixels,
                        );

                        let texture = ctx.load_texture(
                            &format!("viewer_{}", cur_path.to_string_lossy()),
                            color_image,
                            egui::TextureOptions::default(),
                        );
                        self.current_image_texture = Some(texture);
                    }
                }

                if let Some(ref tex) = self.current_image_texture {
                    // Draw the image directly into display_rect
                    // display_rect already has correct aspect from calculate_display_rect()
                    // including fit-to-view, zoom, and pan
                    painter.image(
                        tex.id(),
                        display_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    // Image not in cache yet — show fallback
                    painter.rect_filled(display_rect, 0.0, Color32::from_rgb(30, 30, 30));
                    painter.text(
                        display_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &self.filename,
                        egui::FontId::proportional(14.0),
                        Color32::from_rgb(180, 180, 180),
                    );
                }
            } else {
                // No image cache provided — fall back to placeholder
                painter.rect_filled(display_rect, 0.0, Color32::from_rgb(30, 30, 30));
                painter.text(
                    display_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &self.filename,
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(180, 180, 180),
                );
            }
        }

        // Draw navigation arrows (fade in on hover)
        if left_hovered {
            let fade_rect = Rect::from_min_size(
                viewport_rect.min,
                Vec2::new(70.0, viewport_rect.height()),
            );
            painter.rect_filled(fade_rect, 0.0, Color32::from_black_alpha(90));
            painter.text(
                egui::pos2(viewport_rect.min.x + 40.0, viewport_rect.center().y),
                egui::Align2::CENTER_CENTER,
                "\u{25C0}",
                egui::FontId::proportional(36.0),
                Color32::from_white_alpha(200),
            );
        }
        if right_hovered {
            let fade_rect = Rect::from_min_size(
                egui::pos2(viewport_rect.max.x - 70.0, viewport_rect.min.y),
                Vec2::new(70.0, viewport_rect.height()),
            );
            painter.rect_filled(fade_rect, 0.0, Color32::from_black_alpha(90));
            painter.text(
                egui::pos2(viewport_rect.max.x - 40.0, viewport_rect.center().y),
                egui::Align2::CENTER_CENTER,
                "\u{25B6}",
                egui::FontId::proportional(36.0),
                Color32::from_white_alpha(200),
            );
        }

        // Draw "Gallery" button
        let btn_fill = if btn_hovered {
            Color32::from_rgba_premultiplied(50, 50, 50, 200)
        } else {
            Color32::from_black_alpha(120)
        };
        painter.rect_filled(gallery_btn_rect, 4.0, btn_fill);
        painter.text(
            gallery_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{25C4}  Gallery",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(200, 200, 200),
        );

        // === Bottom info bar ===
        // Shows filename (left), zoom (center), position counter (right)
        let info_bar_rect = Rect::from_min_size(
            egui::pos2(viewport_rect.min.x + 16.0, viewport_rect.max.y - 40.0),
            Vec2::new(viewport_rect.width() - 32.0, 32.0),
        );
        painter.rect_filled(info_bar_rect, 6.0, Color32::from_black_alpha(160));

        // Left: filename
        painter.text(
            egui::pos2(info_bar_rect.min.x + 12.0, info_bar_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &self.filename,
            egui::FontId::proportional(13.0),
            Color32::from_rgb(220, 220, 220),
        );

        // Center: zoom level
        let zoom_text = if self.fit_to_view {
            "Fit to view".to_string()
        } else {
            format!("{:.0}%", self.zoom * 100.0)
        };
        painter.text(
            egui::pos2(info_bar_rect.center().x, info_bar_rect.center().y),
            egui::Align2::CENTER_CENTER,
            &zoom_text,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(160, 160, 160),
        );

        // Right: position counter (e.g., "3 / 15")
        // Find the current image position in the gallery items
        let pos_text = if let Some(ref cur) = self.current_image {
            if let Some(pos) = self.gallery_items.iter().position(|p| p == cur) {
                format!("{} / {}", pos + 1, self.gallery_items.len())
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        if !pos_text.is_empty() {
            painter.text(
                egui::pos2(info_bar_rect.max.x - 12.0, info_bar_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                &pos_text,
                egui::FontId::proportional(13.0),
                Color32::from_rgb(200, 200, 220),
            );
        }

        // === Top-right: keyboard shortcut hint ===
        painter.text(
            egui::pos2(viewport_rect.max.x - 16.0, viewport_rect.min.y + 28.0),
            egui::Align2::RIGHT_TOP,
            "Scroll: zoom  |  Middle drag: pan",
            egui::FontId::proportional(10.0),
            Color32::from_rgba_premultiplied(180, 180, 180, 160),
        );

        // === Filmstrip bar at the bottom ===
        self.render_filmstrip(ctx, ui, viewport_rect, thumbnail_cache);
    }

    /// Render the gallery landing page (shown when no image is loaded)
    fn render_gallery_landing(&mut self, ctx: &Context, ui: &mut Ui, thumbnail_cache: Option<&ThumbnailCache>) {
        let available = ui.available_size();
        let rect = Rect::from_min_size(ui.next_widget_position(), available);
        let bg = Color32::from_rgb(18, 18, 18);

        if self.gallery_items.is_empty() {
            self.render_empty_gallery(ctx, ui, rect, bg, available);
            return;
        }

        self.render_thumbnail_grid(ctx, ui, rect, bg, available, thumbnail_cache);
    }

    fn render_empty_gallery(&mut self, ctx: &Context, ui: &mut Ui, rect: Rect, bg: Color32, _available: Vec2) {
        // First pass: allocate all interactive rects and check clicks
        let mut folder_clicked: Option<PathBuf> = None;

        let folder_rects: Vec<(Rect, PathBuf, String)> = {
            let mut rects = Vec::new();
            let mut folder_y = rect.center().y + 90.0;

            for (label, dir_fn) in &[
                ("Pictures", dirs::picture_dir as fn() -> Option<PathBuf>),
                ("Desktop", dirs::desktop_dir as fn() -> Option<PathBuf>),
                ("Downloads", dirs::download_dir as fn() -> Option<PathBuf>),
            ] {
                if let Some(dir_path) = dir_fn() {
                    let btn_rect = Rect::from_min_size(
                        egui::pos2(rect.center().x - 100.0, folder_y),
                        Vec2::new(200.0, 30.0),
                    );

                    let response = ui.allocate_rect(btn_rect, Sense::click());
                    if response.clicked() {
                        folder_clicked = Some(dir_path.clone());
                    }

                    rects.push((btn_rect, dir_path.clone(), format!("  {label}")));
                    folder_y += 38.0;
                }
            }
            rects
        };

        // Handle folder click after UI interaction
        if let Some(ref path) = folder_clicked {
            self.pending_folder_open = Some(path.clone());
        }

        // Second pass: paint everything
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, bg);

        // Title
        painter.text(
            rect.center() - Vec2::new(0.0, 80.0),
            egui::Align2::CENTER_CENTER,
            "Glint",
            egui::FontId::proportional(48.0),
            Color32::from_rgb(70, 130, 255),
        );

        painter.text(
            rect.center() - Vec2::new(0.0, 30.0),
            egui::Align2::CENTER_CENTER,
            "Your Photos, Your Way",
            egui::FontId::proportional(18.0),
            Color32::from_rgb(180, 180, 180),
        );

        painter.text(
            rect.center() + Vec2::new(0.0, 20.0),
            egui::Align2::CENTER_CENTER,
            "Drop images anywhere to start  |  Ctrl+O to open a file  |  Ctrl+D to open a folder",
            egui::FontId::proportional(13.0),
            Color32::from_rgb(120, 120, 120),
        );

        painter.text(
            rect.center() + Vec2::new(0.0, 60.0),
            egui::Align2::CENTER_CENTER,
            "Or browse your folders below:",
            egui::FontId::proportional(12.0),
            Color32::from_rgb(100, 100, 100),
        );

        // Draw folder buttons
        let interact_pos = ctx.input(|i| i.pointer.interact_pos());
        for (btn_rect, _path, label) in &folder_rects {
            let hovered = btn_rect.contains(interact_pos.unwrap_or(Pos2::ZERO));
            let fill = if hovered {
                Color32::from_rgb(70, 130, 255)
            } else {
                Color32::from_rgb(40, 40, 40)
            };
            painter.rect_filled(*btn_rect, 6.0, fill);
            painter.text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );
        }
    }

    fn render_thumbnail_grid(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        rect: Rect,
        bg: Color32,
        available: Vec2,
        thumbnail_cache: Option<&ThumbnailCache>,
    ) {
        let item_count = self.gallery_items.len();

        // Calculate grid layout (scaled by thumb_zoom)
        let thumb_gen_size = (256.0 * self.thumb_zoom) as u32;
        let thumb_display_size = 140.0 * self.thumb_zoom;
        let padding = 8.0;
        let available_width = available.x;
        let available_height = available.y;
        let cols = ((available_width - 32.0) / (thumb_display_size + padding))
            .floor().max(1.0) as usize;
        let cell_size = thumb_display_size + padding;

        let thumb_image_area = thumb_display_size - 8.0;
        let thumb_image_height = thumb_display_size - 28.0;

        let start_x = rect.min.x + 16.0;
        let start_y = rect.min.y + 50.0 - self.scroll_offset;
        let total_rows = if cols > 0 { (item_count + cols - 1) / cols } else { 0 };
        let total_height = total_rows as f32 * cell_size + 50.0;

        // Handle scroll wheel
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        if scroll_delta.y != 0.0 && total_height > available_height {
            self.scroll_offset = (self.scroll_offset - scroll_delta.y * 2.0)
                .clamp(0.0, (total_height - available_height + 50.0).max(0.0));
        }

        // === VIRTUAL SCROLLING: Calculate visible index range ===
        // Only iterate over items that are actually visible on screen.
        // This makes rendering O(visible_items) instead of O(total_items).
        let viewport_top = self.scroll_offset.max(50.0) - 50.0;
        let viewport_bottom = viewport_top + available_height;

        // Add a buffer of one cell above and below to prevent pop-in during scrolling
        let buffer = cell_size;
        let first_row = ((viewport_top - buffer) / cell_size).floor().max(0.0) as usize;
        let last_row_raw = ((viewport_bottom + buffer) / cell_size).ceil() as usize;
        let last_row = last_row_raw.min(total_rows.saturating_sub(1)).max(first_row);

        let first_index = first_row * cols;
        let last_index = ((last_row + 1) * cols).min(item_count);

        // Read modifier keys for multi-selection and keyboard shortcuts
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let key_a = ctx.input(|i| i.key_pressed(egui::Key::A) && i.modifiers.ctrl);
        let delete_key = ctx.input(|i| i.key_pressed(egui::Key::Delete));

        // Keyboard shortcuts for selection
        if escape && !self.selected_indices.is_empty() {
            self.clear_selection();
        }
        if key_a {
            // Ctrl+A: select all visible items
            self.selected_indices.clear();
            for i in 0..item_count {
                self.selected_indices.insert(i);
            }
            if !self.gallery_items.is_empty() {
                self.selected_gallery_index = Some(0);
                self.last_clicked_index = Some(item_count - 1);
            }
        }
        if delete_key && !self.selected_indices.is_empty() {
            self.pending_delete = Some(self.selected_paths());
        }

        // Thumbnail zoom slider (below the header text)
        let slider_rect = Rect::from_min_size(
            egui::pos2(rect.max.x - 200.0, rect.min.y + 44.0),
            Vec2::new(180.0, 28.0),
        );
        let zoom_response = ui.put(
            slider_rect,
            egui::Slider::new(&mut self.thumb_zoom, 0.5..=2.0)
                .text("Size")
                .step_by(0.1)
                .show_value(false),
        );
        if zoom_response.changed() {
            // Clear thumbnail textures to regenerate at new size
            self.thumbnail_textures.clear();
            // Reset scroll to top when zoom changes
            self.scroll_offset = 0.0;
        }

        // First pass: allocate rects for visible items and detect interactions
        let mut clicked_index: Option<usize> = None;
        let mut right_clicked_index: Option<usize> = None;

        // Drag reorder: reset if mouse is released
        if self.drag_source_index.is_some() && !ui.input(|i| i.pointer.any_down()) {
            // Drop: reorder items
            // Need to fix indices: after removing src, items after src shift down by 1
            if let (Some(src), Some(mut dst)) = (self.drag_source_index, self.drag_target_index) {
                if src != dst && dst < self.gallery_items.len() {
                    let item = self.gallery_items.remove(src);
                    // If the target was after the source, account for the shift
                    if dst > src {
                        dst = dst.saturating_sub(1);
                    }
                    if dst <= self.gallery_items.len() {
                        self.gallery_items.insert(dst, item);
                    } else {
                        self.gallery_items.push(item);
                    }
                    self.clear_thumbnail_cache();
                }
            }
            self.drag_source_index = None;
            self.drag_target_index = None;
        }

        for i in first_index..last_index {
            let row = i / cols;
            let col = i % cols;
            let x = start_x + col as f32 * cell_size;
            let y = start_y + row as f32 * cell_size;

            let item_rect = Rect::from_min_size(
                egui::pos2(x, y),
                Vec2::new(thumb_display_size, thumb_display_size),
            );

            // Use click_and_drag() for items to support both selection and drag-reorder
            let response = ui.allocate_rect(item_rect, Sense::click_and_drag());

            // Context menu on right-click
            if response.secondary_clicked() {
                right_clicked_index = Some(i);
            }

            if response.clicked() {
                clicked_index = Some(i);
            }
            if response.double_clicked() {
                // Double-click always opens, regardless of selection mode
                if let Some(item) = self.gallery_items.get(i) {
                    self.pending_gallery_open = Some(item.clone());
                }
            }

            // Detect drag start for reorder
            if response.dragged() && self.selected_indices.contains(&i) {
                if self.drag_source_index.is_none() {
                    self.drag_source_index = Some(i);
                }
                // Calculate drop target based on pointer position
                let pointer_pos = ui.input(|p| p.pointer.hover_pos());
                if let Some(pos) = pointer_pos {
                    let rel_x = pos.x - start_x;
                    let rel_y = pos.y - start_y;
                    let target_col = (rel_x / cell_size).floor() as usize;
                    let target_row = (rel_y / cell_size).floor() as usize;
                    let target = (target_row * cols + target_col).min(item_count.saturating_sub(1));
                    self.drag_target_index = Some(target);
                }
            }
        }

        // Handle single-click selection with modifiers
        if let Some(index) = clicked_index {
            self.handle_gallery_click(index, ctrl, shift);
        }

        // Handle right-click: select and show context menu
        if let Some(index) = right_clicked_index {
            if !self.selected_indices.contains(&index) {
                self.selected_indices.clear();
                self.selected_indices.insert(index);
                self.selected_gallery_index = Some(index);
                self.last_clicked_index = Some(index);
            }
            // Open context menu at pointer position
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                self.context_menu_open = true;
                self.context_menu_pos = pos;
            }
        }

        // Second pass: paint everything
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, bg);

        let sel_count = self.selected_indices.len();

        // Header
        let header_text = if sel_count > 1 {
            format!("Gallery  -  {sel_count} selected  ({item_count} total)")
        } else if first_index > 0 || last_index < item_count {
            format!("Gallery  -  {item_count} images  (showing {}-{})",
                first_index + 1, last_index.min(item_count))
        } else {
            format!("Gallery  -  {item_count} images")
        };
        painter.text(
            egui::pos2(rect.min.x + 16.0, rect.min.y + 16.0),
            egui::Align2::LEFT_TOP,
            &header_text,
            egui::FontId::proportional(18.0),
            Color32::from_rgb(220, 220, 220),
        );

        // Selection actions toolbar (shown when items are selected)
        if sel_count > 1 {
            let action_x = rect.min.x + 16.0;
            let action_y = rect.min.y + 38.0;
            painter.text(
                egui::pos2(action_x, action_y),
                egui::Align2::LEFT_TOP,
                "Escape: deselect  |  Ctrl+A: select all  |  Delete: remove",
                egui::FontId::proportional(11.0),
                Color32::from_rgb(120, 120, 120),
            );
        }

        // Paint only visible thumbnails
        for i in first_index..last_index.min(item_count) {
            if let Some(item) = self.gallery_items.get(i) {
                let row = i / cols;
                let col = i % cols;
                let x = start_x + col as f32 * cell_size;
                let y = start_y + row as f32 * cell_size;

                let item_rect = Rect::from_min_size(
                    egui::pos2(x, y),
                    Vec2::new(thumb_display_size, thumb_display_size),
                );

                let filename = item
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let is_selected = self.selected_indices.contains(&i);
                let is_drag_source = self.drag_source_index == Some(i);
                let is_drop_target = self.drag_target_index == Some(i)
                    && self.drag_source_index.is_some()
                    && self.drag_source_index != Some(i);

                // Background card
                let bg_color = if is_drag_source {
                    Color32::from_rgb(100, 100, 100) // dimmed while dragging
                } else if is_drop_target {
                    Color32::from_rgb(40, 110, 200) // highlight drop target
                } else if is_selected {
                    Color32::from_rgb(50, 90, 180)
                } else {
                    Color32::from_rgb(30, 30, 30)
                };

                // Selection indicator: colored top border for selected items
                if is_selected && !is_drag_source {
                    painter.rect_filled(
                        Rect::from_min_size(
                            egui::pos2(item_rect.min.x, item_rect.min.y),
                            Vec2::new(item_rect.width(), 3.0),
                        ),
                        3.0,
                        Color32::from_rgb(70, 150, 255),
                    );
                }

                let checker_color1 = Color32::from_rgb(40, 40, 40);
                let checker_color2 = Color32::from_rgb(50, 50, 50);

                painter.rect_filled(item_rect, 6.0, bg_color);

                // Selection overlay checkbox on the top-right corner
                if is_selected {
                    let check_rect = Rect::from_min_size(
                        egui::pos2(item_rect.max.x - 22.0, item_rect.min.y + 4.0),
                        Vec2::new(18.0, 18.0),
                    );
                    painter.rect_filled(check_rect, 3.0, Color32::from_rgb(70, 150, 255));
                    // Checkmark
                    painter.text(
                        check_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "\u{2713}",
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }

                // Selection number badge (if showing multiple)
                if sel_count > 1 && is_selected {
                    // Show selection order number (sorted for stable display)
                    let mut order: Vec<usize> = self.selected_indices.iter().cloned().collect();
                    order.sort_unstable();
                    let pos = order.iter().position(|x| *x == i);
                    if let Some(n) = pos {
                        let num_rect = Rect::from_min_size(
                            egui::pos2(item_rect.min.x + 4.0, item_rect.min.y + 4.0),
                            Vec2::new(18.0, 18.0),
                        );
                        painter.rect_filled(num_rect, 3.0, Color32::from_rgb(70, 150, 255));
                        painter.text(
                            num_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &format!("{}", n + 1),
                            egui::FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                    }
                }

                // Thumbnail image area
                let thumb_area = Rect::from_min_size(
                    egui::pos2(x + 4.0, y + 4.0),
                    Vec2::new(thumb_image_area, thumb_image_height),
                );

                // Draw checkerboard background for transparency
                let check_size = 4.0;
                for cx in 0..(thumb_image_area / check_size).ceil() as i32 {
                    for cy in 0..(thumb_image_height / check_size).ceil() as i32 {
                        let c = if (cx + cy) % 2 == 0 { checker_color1 } else { checker_color2 };
                        painter.rect_filled(
                            Rect::from_min_size(
                                egui::pos2(thumb_area.min.x + cx as f32 * check_size,
                                           thumb_area.min.y + cy as f32 * check_size),
                                Vec2::new(check_size, check_size),
                            ),
                            0.0,
                            c,
                        );
                    }
                }

                // Render actual thumbnail image
                let has_thumbnail = if let Some(cache) = thumbnail_cache {
                    let tex_id = Self::ensure_thumbnail(
                        &mut self.thumbnail_textures,
                        self.max_textures,
                        ctx,
                        cache,
                        item,
                        thumb_gen_size,
                    );
                    if let Some(tid) = tex_id {
                        let tex_size = self.thumbnail_textures.get(item)
                            .map(|t| t.size_vec2())
                            .unwrap_or(Vec2::new(1.0, 1.0));
                        let aspect = tex_size.x / tex_size.y;

                        let (img_w, img_h) = if aspect > (thumb_image_area / thumb_image_height) {
                            (thumb_image_area, thumb_image_area / aspect)
                        } else {
                            (thumb_image_height * aspect, thumb_image_height)
                        };

                        let img_rect = Rect::from_center_size(
                            thumb_area.center(),
                            Vec2::new(img_w, img_h),
                        );

                        painter.image(
                            tid,
                            img_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // Fallback: show file extension badge
                if !has_thumbnail {
                    let ext = item.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_uppercase();
                    painter.text(
                        thumb_area.center(),
                        egui::Align2::CENTER_CENTER,
                        &ext,
                        egui::FontId::proportional(16.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }

                // Filename label
                let truncated = if filename.len() > 14 {
                    format!("{}...", &filename[..12])
                } else {
                    filename.clone()
                };
                painter.text(
                    egui::pos2(item_rect.center().x, item_rect.max.y - 8.0),
                    egui::Align2::CENTER_CENTER,
                    &truncated,
                    egui::FontId::proportional(10.0),
                    Color32::from_rgb(180, 180, 180),
                );
            }
        }

        // Scrollbar indicator (only shown when content overflows)
        if total_height > available_height {
            let scrollable_height = total_height - 50.0;
            let visible_ratio = (available_height - 50.0) / scrollable_height;
            let scrollbar_height = visible_ratio * (available_height - 50.0);
            let scrollbar_y = rect.min.y + 50.0
                + (self.scroll_offset / scrollable_height) * (available_height - 50.0);
            painter.rect_filled(
                Rect::from_min_size(
                    egui::pos2(rect.max.x - 6.0, scrollbar_y),
                    Vec2::new(4.0, scrollbar_height.max(20.0)),
                ),
                2.0,
                Color32::from_rgb(80, 80, 80),
            );
        }

        // Info at bottom-right
        let info_text = if sel_count > 1 {
            format!("{sel_count} selected  /  {item_count} images")
        } else {
            format!("{item_count} images  |  scrolling {}-{}",
                first_index + 1, last_index.min(item_count))
        };
        painter.text(
            egui::pos2(rect.max.x - 12.0, rect.max.y - 8.0),
            egui::Align2::RIGHT_BOTTOM,
            &info_text,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(80, 80, 80),
        );

        // Context menu popup (shown on right-click)
        if self.context_menu_open {
            // Close context menu if user clicks elsewhere
            let any_mouse_down = ctx.input(|i| i.pointer.any_down());
            let click_outside = !ctx.input(|i| {
                let pos = i.pointer.hover_pos().unwrap_or(Pos2::ZERO);
                let menu_rect = Rect::from_min_size(
                    self.context_menu_pos,
                    Vec2::new(180.0, 150.0),
                );
                menu_rect.contains(pos)
            });

            if any_mouse_down && click_outside {
                self.context_menu_open = false;
            }

            egui::Area::new(egui::Id::new("gallery_context_menu"))
                .fixed_pos(self.context_menu_pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let frame = egui::Frame::popup(ui.style());
                    frame.show(ui, |ui| {
                        ui.set_min_width(160.0);

                        let sel_paths = self.selected_paths();
                        let sel_label = if sel_paths.len() > 1 {
                            format!("{} items selected", sel_paths.len())
                        } else if sel_paths.len() == 1 {
                            sel_paths[0].file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string()
                        } else {
                            String::new()
                        };

                        ui.label(
                            egui::RichText::new(&sel_label)
                                .size(11.0)
                                .color(Color32::from_rgb(140, 140, 140)),
                        );
                        ui.separator();

                        // Open
                        if ui.button("  \u{1F441}  Open").clicked() {
                            if let Some(path) = sel_paths.first() {
                                self.pending_gallery_open = Some(path.clone());
                            }
                            self.context_menu_open = false;
                        }

                        // Copy path
                        if ui.button("  \u{1F4CB}  Copy Path").clicked() {
                            // Copy first selected path to clipboard
                            if let Some(path) = sel_paths.first() {
                                if let Some(path_str) = path.to_str() {
                                    ui.ctx().copy_text(path_str.to_string());
                                }
                            }
                            self.context_menu_open = false;
                        }

                        ui.separator();

                        // Delete
                        if ui.button("  \u{1F5D1}  Delete").clicked() {
                            self.pending_delete = Some(self.selected_paths());
                            self.context_menu_open = false;
                        }

                        // Move to...
                        if ui.button("  \u{1F4C2}  Move to...").clicked() {
                            self.pending_move_to = Some(self.selected_paths());
                            self.context_menu_open = false;
                        }

                        ui.separator();

                        // Select All / Deselect
                        if ui.button("  \u{220E}  Select All").clicked() {
                            self.selected_indices.clear();
                            for i in 0..item_count {
                                self.selected_indices.insert(i);
                            }
                            self.context_menu_open = false;
                        }
                    });
                });
        }
    }

    /// Render the filmstrip bar at the bottom of the photo viewer
    fn render_filmstrip(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        viewport_rect: Rect,
        thumbnail_cache: Option<&ThumbnailCache>,
    ) {
        let item_count = self.gallery_items.len();
        if item_count == 0 {
            return;
        }

        let filmstrip_height = 80.0;
        let thumb_size = 60.0;
        let thumb_pad = 6.0;
        let cell_w = thumb_size + thumb_pad;

        // Filmstrip bar position — ABOVE the info bar (which is at max.y-40..max.y-8)
        // Place it flush above the info bar with a small 4px gap
        let bar_bottom_y = viewport_rect.max.y - 48.0; // 8px above info bar top
        let bar_y = bar_bottom_y - filmstrip_height;
        let bar_rect = Rect::from_min_size(
            egui::pos2(viewport_rect.min.x, bar_y),
            Vec2::new(viewport_rect.width(), filmstrip_height),
        );

        if self.filmstrip_hidden {
            // Collapsed state: show a small toggle button near the bottom edge (above the info bar)
            let collapsed_toggle_rect = Rect::from_min_size(
                egui::pos2(viewport_rect.center().x - 30.0, viewport_rect.max.y - 60.0),
                Vec2::new(60.0, 16.0),
            );
            let collapsed_toggle_resp = ui.allocate_rect(collapsed_toggle_rect, Sense::click());
            let collapsed_toggle_hovered = collapsed_toggle_resp.hovered();
            if collapsed_toggle_resp.clicked() {
                self.filmstrip_hidden = false;
            }

            let painter = ui.painter();
            let toggle_color = if collapsed_toggle_hovered {
                Color32::from_rgba_premultiplied(80, 80, 80, 200)
            } else {
                Color32::from_black_alpha(100)
            };
            painter.rect_filled(collapsed_toggle_rect, 4.0, toggle_color);
            painter.text(
                collapsed_toggle_rect.center(),
                egui::Align2::CENTER_CENTER,
                "\u{25B2}", // ▲ (click to expand)
                egui::FontId::proportional(10.0),
                Color32::from_gray(160),
            );
            return;
        }

        // Expanded state: toggle button at the top-center of the filmstrip
        let toggle_btn_rect = Rect::from_min_size(
            egui::pos2(bar_rect.center().x - 30.0, bar_y - 14.0),
            Vec2::new(60.0, 16.0),
        );
        let toggle_resp = ui.allocate_rect(toggle_btn_rect, Sense::click());
        let toggle_hovered = toggle_resp.hovered();
        if toggle_resp.clicked() {
            self.filmstrip_hidden = true;
        }

        // Expanded state — draw the filmstrip
        // Phase 1: Interaction (allocate rects for scroll + thumbnails)

        // Scroll wheel for horizontal scrolling
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        let total_width = item_count as f32 * cell_w;
        let visible_width = bar_rect.width() - 32.0;
        if scroll_delta.y != 0.0 && total_width > visible_width {
            self.filmstrip_offset = (self.filmstrip_offset - scroll_delta.y * 3.0)
                .clamp(0.0, (total_width - visible_width).max(0.0));
        }

        // Find the current image index
        let current_index = self.current_image.as_ref()
            .and_then(|cur| self.gallery_items.iter().position(|p| p == cur));

        let mut clicked_path: Option<PathBuf> = None;

        let start_x = bar_rect.min.x + 16.0 - self.filmstrip_offset;
        let start_y = bar_rect.min.y + 10.0;

        // Only allocate rects for visible thumbnails (virtual horizontal scrolling)
        let first_vis = (self.filmstrip_offset / cell_w).floor() as usize;
        let last_vis_raw = ((self.filmstrip_offset + visible_width) / cell_w).ceil() as usize + 1;
        let last_vis = last_vis_raw.min(item_count);

        for i in first_vis..last_vis {
            let x = start_x + i as f32 * cell_w;
            if x + thumb_size < bar_rect.min.x || x > bar_rect.max.x {
                continue;
            }

            let item_rect = Rect::from_min_size(
                egui::pos2(x, start_y),
                Vec2::new(thumb_size, thumb_size),
            );

            let response = ui.allocate_rect(item_rect, Sense::click());
            if response.clicked() {
                if let Some(item) = self.gallery_items.get(i) {
                    clicked_path = Some(item.clone());
                }
            }
        }

        // Handle click navigation
        if let Some(path) = clicked_path {
            self.pending_gallery_open = Some(path);
        }

        // Phase 2: Painting
        let painter = ui.painter();

        // Filmstrip background
        painter.rect_filled(bar_rect, 6.0, Color32::from_black_alpha(180));

        // Toggle button (expanded state)
        let toggle_color = if toggle_hovered {
            Color32::from_rgba_premultiplied(80, 80, 80, 180)
        } else {
            Color32::from_black_alpha(80)
        };
        painter.rect_filled(toggle_btn_rect, 4.0, toggle_color);
        painter.text(
            toggle_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{25BC}", // ▼ (click to collapse)
            egui::FontId::proportional(10.0),
            Color32::from_gray(160),
        );

        // Paint thumbnails
        for i in first_vis..last_vis.min(item_count) {
            let x = start_x + i as f32 * cell_w;
            if x + thumb_size < bar_rect.min.x || x > bar_rect.max.x {
                continue;
            }

            let item_rect = Rect::from_min_size(
                egui::pos2(x, start_y),
                Vec2::new(thumb_size, thumb_size),
            );

            let is_current = current_index == Some(i);

            // Background card for thumbnail
            let bg = if is_current {
                Color32::from_rgb(50, 90, 180)
            } else {
                Color32::from_rgb(35, 35, 35)
            };
            painter.rect_filled(item_rect, 4.0, bg);

            // Current image indicator: blue top border
            if is_current {
                painter.rect_filled(
                    Rect::from_min_size(
                        egui::pos2(item_rect.min.x, item_rect.min.y),
                        Vec2::new(item_rect.width(), 3.0),
                    ),
                    3.0,
                    Color32::from_rgb(70, 150, 255),
                );
            }

            // Try to render actual thumbnail
            let has_thumb = if let (Some(item), Some(cache)) = (
                self.gallery_items.get(i),
                thumbnail_cache,
            ) {
                let tex_id = Self::ensure_thumbnail(
                    &mut self.thumbnail_textures,
                    self.max_textures,
                    ctx,
                    cache,
                    item,
                    64, // small thumbnail for filmstrip
                );
                if let Some(tid) = tex_id {
                    let tex_size = self.thumbnail_textures.get(item)
                        .map(|t| t.size_vec2())
                        .unwrap_or(Vec2::new(1.0, 1.0));
                    let aspect = tex_size.x / tex_size.y;

                    let thumb_area = Rect::from_min_size(
                        egui::pos2(x + 4.0, start_y + 4.0),
                        Vec2::new(thumb_size - 8.0, thumb_size - 8.0),
                    );

                    let (img_w, img_h) = if aspect > 1.0 {
                        (thumb_area.width(), thumb_area.width() / aspect)
                    } else {
                        (thumb_area.height() * aspect, thumb_area.height())
                    };

                    let img_rect = Rect::from_center_size(
                        thumb_area.center(),
                        Vec2::new(img_w, img_h),
                    );

                    painter.image(
                        tid,
                        img_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Fallback: extension badge
            if !has_thumb {
                if let Some(item) = self.gallery_items.get(i) {
                    let ext = item.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("?")
                        .to_uppercase();
                    painter.text(
                        item_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &ext,
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(100, 100, 100),
                    );
                }
            }
        }

        // Scroll indicator (if content overflows)
        if total_width > visible_width {
            let scroll_ratio = visible_width / total_width;
            let scrollbar_w = scroll_ratio * visible_width;
            let scrollbar_x = bar_rect.min.x + 16.0
                + (self.filmstrip_offset / (total_width - visible_width)) * (visible_width - scrollbar_w);
            painter.rect_filled(
                Rect::from_min_size(
                    egui::pos2(scrollbar_x, bar_rect.max.y - 4.0),
                    Vec2::new(scrollbar_w.max(20.0), 2.0),
                ),
                1.0,
                Color32::from_rgb(120, 120, 120),
            );
        }
    }

    /// Ensure a thumbnail texture exists for the given path.
    /// Takes a field-level `&mut HashMap` to avoid borrow conflicts with the caller.
    /// Returns `TextureId` (Copy) instead of a reference to avoid lifetime issues.
    fn ensure_thumbnail(
        textures: &mut HashMap<PathBuf, TextureHandle>,
        max_textures: usize,
        ctx: &Context,
        cache: &ThumbnailCache,
        path: &Path,
        size: u32,
    ) -> Option<TextureId> {
        // Check if we already have the texture cached
        if let Some(tex) = textures.get(path) {
            return Some(tex.id());
        }

        // Evict oldest entries if cache is full
        if textures.len() >= max_textures {
            if let Some(key) = textures.keys().next().cloned() {
                textures.remove(&key);
            }
        }

        // Generate thumbnail using the cache
        match cache.get_or_generate(path, size) {
            Ok(Some(color_image)) => {
                let handle = ctx.load_texture(
                    &format!("thumb_{}", path.to_string_lossy()),
                    color_image,
                    Default::default(),
                );
                let id = handle.id();
                textures.insert(path.to_path_buf(), handle);
                Some(id)
            }
            Ok(None) => {
                log::warn!("Failed to generate thumbnail for {:?} (decoding failed)", path);
                None
            }
            Err(e) => {
                log::error!("Thumbnail error for {:?}: {}", path, e);
                None
            }
        }
    }

    fn calculate_display_rect(&self, available: Vec2) -> Rect {
        let image_aspect = if self.image_size.y > 0.0 {
            self.image_size.x / self.image_size.y
        } else {
            1.0
        };

        let display_size = if self.fit_to_view {
            let view_aspect = available.x / available.y;
            if view_aspect > image_aspect {
                Vec2::new(available.y * image_aspect, available.y)
            } else {
                Vec2::new(available.x, available.x / image_aspect)
            }
        } else {
            if self.image_size.x > 0.0 && self.image_size.y > 0.0 {
                self.image_size * self.zoom
            } else {
                Vec2::splat(100.0) * self.zoom
            }
        };

        let centered_pos = Pos2::new(
            (available.x - display_size.x) / 2.0 + self.pan_offset.x,
            (available.y - display_size.y) / 2.0 + self.pan_offset.y,
        );

        Rect::from_min_size(centered_pos, display_size)
    }

    fn render_checkerboard(&self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter();
        let square_size = 8.0f32;
        let color1 = Color32::from_rgb(40, 40, 40);
        let color2 = Color32::from_rgb(50, 50, 50);

        let cols = (rect.width() / square_size).ceil() as i32;
        let rows = (rect.height() / square_size).ceil() as i32;
        let max_squares = 500;

        let mut count = 0;
        for row in 0..rows {
            for col in 0..cols {
                if count >= max_squares {
                    break;
                }
                let color = if (row + col) % 2 == 0 { color1 } else { color2 };
                let x = rect.min.x + col as f32 * square_size;
                let y = rect.min.y + row as f32 * square_size;
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(x, y), Vec2::new(square_size, square_size)),
                    0.0,
                    color,
                );
                count += 1;
            }
            if count >= max_squares {
                break;
            }
        }
    }

    /// Handle mouse and keyboard input for zoom, pan, and navigation
    fn handle_input(&mut self, ctx: &Context, ui: &mut Ui) {
        // Mouse scroll for zoom — skip if pointer is over the filmstrip area
        // Filmstrip occupies max.y - (filmstrip_height + 48) to max.y - 48 when expanded
        // The collapsed toggle is at max.y - 60 to max.y - 44
        const FILMSTRIP_AREA_HEIGHT: f32 = 128.0; // 80px filmstrip + 48px offset (bar_bottom_y = max.y - 48, height = 80, so top = max.y - 128)
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
        let over_filmstrip = if let Some(pos) = pointer_pos {
            let viewport_bottom = ui.max_rect().max.y;
            pos.y > viewport_bottom - FILMSTRIP_AREA_HEIGHT
        } else {
            false
        };

        if over_filmstrip {
            // Don't zoom when pointer is over the filmstrip;
            // the filmstrip consumes scroll for horizontal scrolling
        } else {
            let scroll_delta = ctx.input(|i| i.raw_scroll_delta);
            if scroll_delta.y != 0.0 && self.has_image {
                self.fit_to_view = false;
                let zoom_factor = 1.0 + scroll_delta.y.abs() * 0.01;
                if scroll_delta.y > 0.0 {
                    self.zoom *= zoom_factor;
                } else {
                    self.zoom /= zoom_factor;
                }
                self.zoom = self.zoom.clamp(0.01, 100.0);
            }
        }

        // Interact area for panning
        let response = ui.interact(
            ui.max_rect(),
            egui::Id::new("viewer_pan"),
            egui::Sense::click_and_drag(),
        );

        // Handle drag pan with middle mouse
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            self.pan_offset += delta;
        }

        // Keyboard shortcuts
        let (escape_pressed, r_pressed) = ctx.input(|i| {
            (i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::R))
        });

        if escape_pressed {
            self.fit_to_view = true;
            self.zoom = 1.0;
            self.pan_offset = Vec2::ZERO;
        }
        if r_pressed {
            self.fit_to_view = !self.fit_to_view;
            if self.fit_to_view {
                self.pan_offset = Vec2::ZERO;
            }
        }
    }
}

impl Default for ViewerPanel {
    fn default() -> Self {
        Self::new()
    }
}
