// Glint - Main application state
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use crate::browser::FileBrowser;
use crate::editor::Editor;
use crate::image::ImageCache;
use crate::image::ImageProcessor;
use crate::metadata::MetadataCache;
use crate::thumbnail::ThumbnailCache;
use crate::image::GlintImage;
use crate::ui::{AppTheme, GalleryPanel, IconCache, ToolbarPanel, ViewerPanel};
use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Context, FontId};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Messages sent between components of Glint
pub enum AppMessage {
    OpenFile(PathBuf),
    ImageLoaded(GlintImage),
    OpenDirectory(PathBuf),
    NextImage,
    PreviousImage,
    FirstImage,
    LastImage,
    ReloadImage,
    ToggleFullscreen,
    ToggleGallery,
    ToggleEditing,
    ToggleSlideshow,
    ToggleTheme,
    ZoomIn,
    ZoomOut,
    ZoomFit,
    ZoomActual,
    RotateLeft,
    RotateRight,
    CopyToClipboard,
    CopyFilePath,
    MoveToRecycleBin,
    ShowImageInfo,
    SetAsDefault,
    SetAsDefaultResult(bool),
    OpenDefaultApps,
    EditorAction(String),
    ScanDirectory(PathBuf),
    Exit,
}

/// The root application state for Glint
pub struct GlintApp {
    tx: Sender<AppMessage>,
    rx: Receiver<AppMessage>,
    pub theme: AppTheme,
    toolbar: ToolbarPanel,
    viewer: ViewerPanel,
    gallery: GalleryPanel,
    file_browser: FileBrowser,
    image_cache: ImageCache,
    metadata_cache: MetadataCache,
    thumbnail_cache: ThumbnailCache,
    editor: Editor,
    fullscreen: bool,
    gallery_visible: bool,
    editing_mode: bool,
    slideshow_active: bool,
    slideshow_timer: Instant,
    slideshow_interval: Duration,
    show_image_info: bool,
    window_title: String,
    startup_done: bool,
    is_default_viewer: bool,
    show_set_default_banner: bool,
    setting_default: bool,
    gallery_scanned: bool,
    icons: Option<IconCache>,
}

impl GlintApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = unbounded();

        // Check if Glint is already the default image viewer
        let is_default = crate::platform::WindowsIntegration::is_default_for_test_format();

        // Enable auto-start by default for first run
        #[cfg(windows)]
        {
            if !crate::platform::WindowsIntegration::is_auto_start_enabled() {
                let _ = crate::platform::WindowsIntegration::set_auto_start(true);
            }
        }

        Self {
            tx,
            rx,
            theme: AppTheme::Dark,
            toolbar: ToolbarPanel::new(),
            viewer: ViewerPanel::new(),
            gallery: GalleryPanel::new(),
            file_browser: FileBrowser::new(),
            image_cache: ImageCache::new(50),
            metadata_cache: MetadataCache::new(),
            thumbnail_cache: ThumbnailCache::new(),
            editor: Editor::new(),
            fullscreen: false,
            gallery_visible: false,
            editing_mode: false,
            slideshow_active: false,
            slideshow_timer: Instant::now(),
            slideshow_interval: Duration::from_secs(3),
            show_image_info: false,
            window_title: String::from("Glint"),
            startup_done: false,
            is_default_viewer: is_default,
            show_set_default_banner: !is_default,
            setting_default: false,
            gallery_scanned: false,
            icons: None,
        }
    }

    fn process_messages(&mut self, ctx: &Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::OpenFile(path) => {
                    log::info!("Opening file: {:?}", path);

                    // Scan the parent directory so prev/next arrows have files to navigate
                    if self.file_browser.is_empty()
                        || !self.file_browser.files().iter().any(|f| f == &path)
                    {
                        if let Some(parent) = path.parent() {
                            if parent.is_dir() {
                                self.file_browser.open_directory(parent);
                                self.gallery.set_items(self.file_browser.files().to_vec());
                                self.viewer.set_gallery_items(
                                    self.file_browser.files().to_vec(),
                                );
                            }
                        }
                    }

                    // Sync file_browser index to the opened file
                    if let Some(pos) = self.file_browser.files().iter().position(|f| f == &path) {
                        self.file_browser.set_current_index(pos);
                    }

                    // Load metadata on main thread (fast — reads EXIF headers only)
                    if let Err(e) = self.metadata_cache.load(&path) {
                        log::warn!("Failed to load metadata: {}", e);
                    }

                    // Show the filename and set title immediately (don't wait for decode)
                    self.viewer.set_image(path.clone());
                    self.window_title = format!(
                        "Glint - {}",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                        self.window_title.clone(),
                    ));
                    ctx.request_repaint();

                    // Decode the image on a background thread so the UI stays responsive.
                    let tx = self.tx.clone();
                    let load_path = path.clone();
                    std::thread::spawn(move || {
                        match crate::image::ImageLoader::load(&load_path) {
                            Ok(image) => {
                                let _ = tx.send(AppMessage::ImageLoaded(image));
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to decode image {:?}: {}",
                                    load_path,
                                    e
                                );
                            }
                        }
                    });
                }
                AppMessage::OpenDirectory(path) => {
                    log::info!("Opening directory: {:?}", path);
                    self.file_browser.open_directory(&path);
                    self.gallery.set_items(self.file_browser.files().to_vec());
                    self.viewer.set_gallery_items(self.file_browser.files().to_vec());
                    self.window_title = format!(
                        "Glint - {} ({})",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Folder"),
                        self.file_browser.len()
                    );
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                        self.window_title.clone(),
                    ));
                    ctx.request_repaint();
                }
                AppMessage::NextImage => {
                    if let Some(next) = self.file_browser.next() {
                        self.dispatch(AppMessage::OpenFile(next));
                    }
                }
                AppMessage::PreviousImage => {
                    if let Some(prev) = self.file_browser.previous() {
                        self.dispatch(AppMessage::OpenFile(prev));
                    }
                }
                AppMessage::FirstImage => {
                    if let Some(first) = self.file_browser.first() {
                        self.dispatch(AppMessage::OpenFile(first));
                    }
                }
                AppMessage::LastImage => {
                    if let Some(last) = self.file_browser.last() {
                        self.dispatch(AppMessage::OpenFile(last));
                    }
                }
                AppMessage::ReloadImage => {
                    if let Some(current) = self.file_browser.current() {
                        self.image_cache.invalidate(&current);
                        if let Err(e) = self.image_cache.load(&current) {
                            log::error!("Failed to reload image: {}", e);
                        }
                        ctx.request_repaint();
                    }
                }
                AppMessage::ToggleFullscreen => {
                    self.fullscreen = !self.fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                        self.fullscreen,
                    ));
                }
                AppMessage::ToggleGallery => {
                    self.gallery_visible = !self.gallery_visible;
                }
                AppMessage::ToggleEditing => {
                    self.editing_mode = !self.editing_mode;
                }
                AppMessage::ToggleSlideshow => {
                    self.slideshow_active = !self.slideshow_active;
                    self.slideshow_timer = Instant::now();
                }
                AppMessage::ToggleTheme => {
                    self.theme = match self.theme {
                        AppTheme::Dark => AppTheme::Light,
                        AppTheme::Light => AppTheme::Amoled,
                        AppTheme::Amoled => AppTheme::Dark,
                    };
                    self.theme.apply_to_context(ctx);
                }
                AppMessage::ZoomIn => {
                    let current_zoom = self.viewer.zoom();
                    self.viewer.set_zoom((current_zoom * 1.25).min(100.0));
                    self.viewer.set_fit_to_view(false);
                }
                AppMessage::ZoomOut => {
                    let current_zoom = self.viewer.zoom();
                    self.viewer.set_zoom((current_zoom / 1.25).max(0.01));
                    self.viewer.set_fit_to_view(false);
                }
                AppMessage::ZoomFit => {
                    self.viewer.set_fit_to_view(true);
                    self.viewer.set_pan_offset(egui::Vec2::ZERO);
                }
                AppMessage::ZoomActual => {
                    self.viewer.set_zoom(1.0);
                    self.viewer.set_fit_to_view(false);
                    self.viewer.set_pan_offset(egui::Vec2::ZERO);
                }
                AppMessage::RotateLeft => {
                    // Rotate -90 degrees (counter-clockwise) and insert directly into cache
                    if let Some(img) = self.editor.set_from_cache(&self.image_cache, &self.file_browser) {
                        if let Ok(rotated) = ImageProcessor::rotate(&img, -90.0) {
                            if let Some(ref path) = self.file_browser.current() {
                                self.editor.set_current_image(rotated.clone());
                                self.image_cache.insert(path.clone(), rotated);
                                self.viewer.set_image(path.clone());
                            }
                        }
                    }
                }
                AppMessage::RotateRight => {
                    // Rotate +90 degrees (clockwise) and insert directly into cache
                    if let Some(img) = self.editor.set_from_cache(&self.image_cache, &self.file_browser) {
                        if let Ok(rotated) = ImageProcessor::rotate(&img, 90.0) {
                            if let Some(ref path) = self.file_browser.current() {
                                self.editor.set_current_image(rotated.clone());
                                self.image_cache.insert(path.clone(), rotated);
                                self.viewer.set_image(path.clone());
                            }
                        }
                    }
                }
                AppMessage::ImageLoaded(image) => {
                    // Image finished decoding on background thread — insert into cache
                    let path = image.path.clone();
                    self.image_cache.insert(path.clone(), image);
                    self.viewer.set_loading(false);
                    ctx.request_repaint();
                }
                AppMessage::CopyToClipboard => {
                    // Copy first selected path to clipboard via egui
                    let paths = self.viewer.selected_paths();
                    if let Some(path) = paths.first().or(self.file_browser.current().as_ref()) {
                        if let Some(path_str) = path.to_str() {
                            ctx.copy_text(path_str.to_string());
                        }
                    }
                }
                AppMessage::CopyFilePath => {
                    if let Some(ref current) = self.file_browser.current() {
                        if let Some(path_str) = current.to_str() {
                            ctx.copy_text(path_str.to_string());
                        }
                    }
                }
                AppMessage::MoveToRecycleBin => {
                    let delete_path = self.file_browser.current();
                    if let Some(ref current) = delete_path {
                        #[cfg(windows)]
                        {
                            let path_str = current.to_string_lossy();
                            // Use PowerShell to move file to recycle bin via Shell.Application
                            // Run silently - no console window flashes
                            let _ = std::process::Command::new("powershell")
                                .args(&[
                                    "-NoProfile",
                                    "-WindowStyle", "Hidden",
                                    "-Command",
                                    &format!(
                                        "(New-Object -ComObject Shell.Application).Namespace(0).ParseName('{}').InvokeVerb('delete')",
                                        path_str.replace("'", "''")
                                    ),
                                ])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                                .spawn();
                        }
                        #[cfg(not(windows))]
                        {
                            let _ = std::fs::remove_file(current);
                        }
                        // Remove the file from file list and navigate to next image
                        self.file_browser.remove_paths(&[current.clone()]);
                        self.viewer.remove_paths(&[current.clone()]);
                        self.dispatch(AppMessage::NextImage);
                    }
                }
                AppMessage::ShowImageInfo => {
                    self.show_image_info = !self.show_image_info;
                }
                AppMessage::SetAsDefault => {
                    if self.setting_default {
                        return;
                    }
                    self.setting_default = true;

                    // Run registry + COM operations on a background thread
                    // so the UI doesn't freeze and command prompts don't appear
                    let tx = self.tx.clone();
                    std::thread::spawn(move || {
                        log::info!("Setting Glint as default viewer (background thread)...");

                        #[cfg(windows)]
                        {
                            // First register as available
                            match crate::platform::WindowsIntegration::register_as_available() {
                                Ok(_) => {
                                    // Then try the COM API to set as actual default
                                    match crate::platform::WindowsIntegration::set_as_default_only() {
                                        Ok(_) => {
                                            log::info!("Glint set as default image viewer!");
                                            let _ = tx.send(AppMessage::SetAsDefaultResult(true));
                                        }
                                        Err(e) => {
                                            log::warn!("COM API failed (non-admin likely): {}", e);
                                            // Fall back: just open settings page
                                            let _ = tx.send(AppMessage::SetAsDefaultResult(false));
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Registration failed: {}", e);
                                    let _ = tx.send(AppMessage::SetAsDefaultResult(false));
                                }
                            }
                        }

                        #[cfg(not(windows))]
                        {
                            let _ = tx.send(AppMessage::SetAsDefaultResult(false));
                        }
                    });
                }
                AppMessage::SetAsDefaultResult(success) => {
                    self.setting_default = false;
                    if success {
                        self.is_default_viewer = true;
                        self.show_set_default_banner = false;
                        log::info!("Glint successfully set as default image viewer!");
                    } else {
                        log::info!("Opening Default Apps settings page...");
                        let _ = crate::platform::WindowsIntegration::open_default_apps_settings();
                    }
                }
                AppMessage::OpenDefaultApps => {
                    #[cfg(windows)]
                    {
                        let _ = crate::platform::WindowsIntegration::open_default_apps_settings();
                    }
                }
                AppMessage::EditorAction(action) => {
                    log::info!("Editor action: {}", action);
                }
                AppMessage::ScanDirectory(path) => {
                    self.file_browser.open_directory(&path);
                    let files = self.file_browser.files().to_vec();
                    let file_count = files.len();
                    self.gallery.set_items(files.clone());
                    self.viewer.set_gallery_items(files);
                    if file_count > 0 {
                        self.window_title = format!(
                            "Glint - {} ({})",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Folder"),
                            self.file_browser.len()
                        );
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            self.window_title.clone(),
                        ));
                    }
                }
                AppMessage::Exit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn handle_slideshow(&mut self, _ctx: &Context) {
        if !self.slideshow_active {
            return;
        }
        if self.slideshow_timer.elapsed() >= self.slideshow_interval {
            self.dispatch(AppMessage::NextImage);
            self.slideshow_timer = Instant::now();
        }
    }

    fn handle_global_shortcuts(&mut self, ctx: &Context) {
        let (f11, escape, f5, space, del, home, end) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::F5),
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::Delete),
                i.key_pressed(egui::Key::Home),
                i.key_pressed(egui::Key::End),
            )
        });

        if f11 {
            self.dispatch(AppMessage::ToggleFullscreen);
        }
        if escape & self.fullscreen {
            self.dispatch(AppMessage::ToggleFullscreen);
        }
        if escape & !self.fullscreen {
            self.dispatch(AppMessage::ZoomFit);
        }
        if f5 {
            self.dispatch(AppMessage::ToggleSlideshow);
        }
        if space {
            self.dispatch(AppMessage::NextImage);
        }
        // Only handle global Delete if gallery has no selection
        if del && self.viewer.selected_indices().is_empty() {
            self.dispatch(AppMessage::MoveToRecycleBin);
        }
        if home {
            self.dispatch(AppMessage::FirstImage);
        }
        if end {
            self.dispatch(AppMessage::LastImage);
        }

        // Ctrl shortcuts
        let (ctrl_f, ctrl_e, ctrl_g, ctrl_o, ctrl_d, ctrl_plus, ctrl_minus, ctrl_0, ctrl_1, ctrl_q, ctrl_i, ctrl_t) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::F) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::E) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::G) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::O) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::D) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Equals) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Q) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::I) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::T) && i.modifiers.ctrl,
            )
        });

        if ctrl_f { self.dispatch(AppMessage::ToggleFullscreen); }
        if ctrl_e { self.dispatch(AppMessage::ToggleEditing); }
        if ctrl_g { self.dispatch(AppMessage::ToggleGallery); }
        if ctrl_o { self.handle_ctrl_o(); }
        if ctrl_d { self.handle_ctrl_d(); }
        if ctrl_plus { self.dispatch(AppMessage::ZoomIn); }
        if ctrl_minus { self.dispatch(AppMessage::ZoomOut); }
        if ctrl_0 { self.dispatch(AppMessage::ZoomFit); }
        if ctrl_1 { self.dispatch(AppMessage::ZoomActual); }
        if ctrl_q { self.dispatch(AppMessage::Exit); }
        if ctrl_i { self.dispatch(AppMessage::ShowImageInfo); }
        if ctrl_t { self.dispatch(AppMessage::ToggleTheme); }
    }

    pub fn dispatch(&self, msg: AppMessage) {
        if let Err(e) = self.tx.send(msg) {
            log::error!("Failed to dispatch message: {}", e);
        }
    }

    /// Check for file system changes (files added/removed in the open directory)
    fn check_fs_updates(&mut self) {
        self.file_browser.check_for_updates();
    }
}

impl eframe::App for GlintApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme on first frame
        if !self.startup_done {
            self.theme.apply_to_context(ctx);

            // Sync with Windows dark/light mode
            #[cfg(windows)]
            {
                if crate::platform::WindowsIntegration::is_windows_dark_mode() {
                    self.theme = AppTheme::Dark;
                } else {
                    self.theme = AppTheme::Light;
                }
                self.theme.apply_to_context(ctx);
            }

            #[cfg(not(windows))]
            {
                self.theme.apply_to_context(ctx);
            }

            self.startup_done = true;
        }

        // On first frame, scan common directories for gallery view
        if !self.gallery_scanned {
            self.gallery_scanned = true;

            // Try to scan the Pictures folder first
            let known_dirs = vec![
                dirs::picture_dir(),
                dirs::desktop_dir(),
                dirs::download_dir(),
            ];

            let found_dir = known_dirs.iter().find_map(|d| d.as_ref());

            if let Some(pics) = found_dir {
                log::info!("Auto-scanning {:?} for gallery view...", pics);
                self.dispatch(AppMessage::ScanDirectory(pics.clone()));
            }
        }

        // Process message queue
        self.process_messages(ctx);

        // Handle slideshow
        self.handle_slideshow(ctx);

        // Handle global keyboard shortcuts
        self.handle_global_shortcuts(ctx);

        // Handle pending gallery opens
        if let Some(path) = self.gallery.take_pending_open() {
            self.dispatch(AppMessage::OpenFile(path));
        }

        // Handle viewer gallery click (when clicking a thumbnail in the landing page)
        if let Some(path) = self.viewer.take_pending_gallery_open() {
            self.dispatch(AppMessage::OpenFile(path));
        }

        // Handle viewer navigation (left/right arrows in photo viewer)
        if self.viewer.take_pending_nav_prev() {
            self.dispatch(AppMessage::PreviousImage);
        }
        if self.viewer.take_pending_nav_next() {
            self.dispatch(AppMessage::NextImage);
        }

        // Handle folder button click from gallery landing page
        if let Some(path) = self.viewer.take_pending_folder_open() {
            self.dispatch(AppMessage::OpenDirectory(path));
        }

        // Handle pending delete from gallery
        if let Some(paths) = self.viewer.take_pending_delete() {
            for path in &paths {
                log::info!("Deleting: {:?}", path);
                #[cfg(windows)]
                {
                    let path_str = path.to_string_lossy();
                    let _ = std::process::Command::new("powershell")
                        .args(&[
                            "-NoProfile",
                            "-WindowStyle", "Hidden",
                            "-Command",
                            &format!(
                                "(New-Object -ComObject Shell.Application).Namespace(0).ParseName('{}').InvokeVerb('delete')",
                                path_str.replace("'", "''")
                            ),
                        ])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .creation_flags(0x08000000)
                        .spawn();
                }
                #[cfg(not(windows))]
                {
                    let _ = std::fs::remove_file(path);
                }
            }
            self.viewer.remove_paths(&paths);
            // Also update file_browser to remove the deleted files
            self.file_browser.remove_paths(&paths);
            ctx.request_repaint();
        }

        // Check for file system changes (files added/removed in open directory)
        self.check_fs_updates();

        // Handle pending move-to from gallery context menu
        if let Some(paths) = self.viewer.take_pending_move_to() {
            if let Some(dest_dir) = rfd::FileDialog::new()
                .set_title("Move selected files to...")
                .pick_folder()
            {
                for path in &paths {
                    if let Some(filename) = path.file_name() {
                        let dest = dest_dir.join(filename);
                        log::info!("Moving {:?} to {:?}", path, dest);
                        if let Err(e) = std::fs::rename(path, &dest) {
                            log::error!("Failed to move {:?}: {}", path, e);
                        }
                    }
                }
                self.viewer.remove_paths(&paths);
                ctx.request_repaint();
            }
        }

        // Initialize icon cache on first frame (needs egui context)
        if self.icons.is_none() {
            let size = 18.0;
            self.icons = Some(IconCache::new(ctx, size));
        }

        // Phase 1: Render toolbar
        let tx = &self.tx;
        let theme_name = match self.theme {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::Amoled => "Amoled",
        };
        let text_color = self.theme.text_color();
        egui::TopBottomPanel::top("toolbar")
            .min_height(40.0)
            .frame(egui::Frame::NONE
                .fill(self.theme.surface_color())
                .stroke(egui::Stroke::new(1.0, self.theme.text_secondary_color().gamma_multiply(0.15)))
            )
            .show(ctx, |ui| {
                if let Some(ref icons) = self.icons {
                    self.toolbar.render(ui, tx, icons, theme_name, text_color);
                }
            });

        // Phase 2: Editor panel (right side)
        if self.editing_mode {
            egui::SidePanel::right("editor_panel")
                .default_width(280.0)
                .resizable(true)
                            .frame(egui::Frame::NONE.fill(self.theme.surface_color()))
                .show(ctx, |ui| {
                    self.editor.render(ui);
                });

            // Check if editor produced a processed result and update the viewer
            if let Some(processed) = self.editor.take_current_image() {
                let path = processed.path.clone();
                self.image_cache.insert(path.clone(), processed);
                self.viewer.set_image(path);
                ctx.request_repaint();
            }

            // Check if editor exported to a new file (Save As or suffix) and sync file_browser
            if let Some(export_path) = self.editor.take_export_path() {
                // Only add to file list if it's in the current directory (suffix saves)
                let in_current_dir = self.file_browser.current_directory()
                    .map(|d| export_path.parent() == Some(d))
                    .unwrap_or(false);
                if in_current_dir {
                    self.file_browser.add_path(export_path.clone());
                    self.gallery.set_items(self.file_browser.files().to_vec());
                    self.viewer.set_gallery_items(self.file_browser.files().to_vec());
                }
                // Sync current_index if the exported file is in the list
                if let Some(pos) = self.file_browser.files().iter().position(|f| f == &export_path) {
                    self.file_browser.set_current_index(pos);
                }
                ctx.request_repaint();
            }
        }

        // Phase 3: Gallery panel (bottom) - only visible when requested
        if self.gallery_visible {
            egui::TopBottomPanel::bottom("gallery")
                .default_height(200.0)
                .resizable(true)
                .min_height(100.0)
                            .frame(egui::Frame::NONE.fill(self.theme.surface_color()))
                .show(ctx, |ui| {
                    self.gallery.render(ui, &self.file_browser);
                });
        }

        // Phase 4: "Set as default" banner (shown on first launch if not default)
        if self.show_set_default_banner {
            egui::TopBottomPanel::top("default_banner")
                .min_height(36.0)
                .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(40, 80, 160)))
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("Glint is not your default photo viewer. ")
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        );
                        if self.setting_default {
                            ui.label(
                                egui::RichText::new("Setting as default...")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(200, 200, 200)),
                            );
                        } else if ui.button(
                            egui::RichText::new("Set as default")
                                .size(12.0)
                                .color(egui::Color32::WHITE),
                        ).clicked() {
                            self.dispatch(AppMessage::SetAsDefault);
                        }
                        ui.add_space(4.0);
                        if ui.button("X").clicked() {
                            self.show_set_default_banner = false;
                        }
                        ui.add_space(4.0);
                    });
                });
        }

        // Phase 5: Central viewing area with sidebar
        egui::SidePanel::left("gallery_sidebar")
            .default_width(220.0)
            .resizable(true)
            .min_width(160.0)
            .max_width(350.0)
            .frame(egui::Frame::NONE.fill(self.theme.surface_color()))
            .show_animated(ctx, !self.viewer.has_image() && self.viewer.gallery_count() > 0, |ui| {
                self.viewer.render_sidebar(ui, &self.file_browser);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(self.theme.background_color()))
            .show(ctx, |ui| {
                self.viewer.render(ctx, ui, Some(&self.thumbnail_cache), Some(&self.image_cache));
                self.handle_drag_and_drop(ctx, ui);

                // Render slideshow indicator
                if self.slideshow_active {
                    let painter = ui.painter();
                    let text = format!(
                        "Slideshow - {}s",
                        self.slideshow_interval.as_secs()
                    );
                    painter.text(
                        egui::pos2(ui.max_rect().right() - 150.0, ui.max_rect().top() + 20.0),
                        egui::Align2::RIGHT_TOP,
                        text,
                        FontId::proportional(13.0),
                        self.theme.accent_color(),
                    );
                }
            });

        // Phase 6: Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(26.0)
                        .frame(egui::Frame::NONE.fill(self.theme.surface_color()))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(26.0);
                    ui.add_space(8.0);

                    // Left side: file info (only if an image is loaded)
                    if self.viewer.has_image() {
                        let file_text = format!(
                            "{}  |  {}x{}",
                            self.viewer.current_filename(),
                            self.viewer.image_width(),
                            self.viewer.image_height(),
                        );
                        ui.label(
                            egui::RichText::new(&file_text)
                                .size(11.0)
                                .color(self.theme.text_secondary_color()),
                        );
                    } else {
                        // Show gallery info
                        let count = self.viewer.gallery_count();
                        if count > 0 {
                            ui.label(
                                egui::RichText::new(format!("{} images", count))
                                    .size(11.0)
                                    .color(self.theme.text_secondary_color()),
                            );
                        }
                    }

                    // Center: zoom level (only if image loaded)
                    if self.viewer.has_image() {
                        ui.with_layout(
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.add_space(16.0);
                                let zoom_text = if self.viewer.fit_to_view() {
                                    "Fit".to_string()
                                } else {
                                    format!("{:.0}%", self.viewer.zoom() * 100.0)
                                };
                                ui.label(
                                    egui::RichText::new(zoom_text)
                                        .size(11.0)
                                        .color(self.theme.text_color()),
                                );
                            },
                        );
                    }

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            // Show default viewer status
                            #[cfg(windows)]
                            if self.is_default_viewer {
                                ui.label(
                                    egui::RichText::new("Default viewer")
                                        .size(10.0)
                                        .color(self.theme.text_secondary_color()),
                                );
                                ui.add_space(8.0);
                            }

                            // Right side: navigation counter + slideshow status
                            if self.slideshow_active {
                                ui.label(
                                    egui::RichText::new("SLIDESHOW")
                                        .size(10.0)
                                        .color(self.theme.accent_color()),
                                );
                                ui.add_space(8.0);
                            }
                            let nav_text = if self.file_browser.len() > 0 {
                                format!(
                                    "{} / {}",
                                    self.file_browser.current_index() + 1,
                                    self.file_browser.len()
                                )
                            } else {
                                String::new()
                            };
                            ui.label(
                                egui::RichText::new(&nav_text)
                                    .size(11.0)
                                    .color(self.theme.text_secondary_color()),
                            );
                            ui.add_space(8.0);
                        },
                    );
                });
            });

        ctx.request_repaint();
    }
}

impl GlintApp {
    fn handle_drag_and_drop(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        use egui::*;

        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            for file in &dropped_files {
                if let Some(path) = &file.path {
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let is_image = matches!(
                        ext.as_str(),
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "tif"
                            | "webp" | "ico" | "avif" | "heic" | "heif"
                            | "svg" | "raw" | "cr2" | "cr3" | "nef" | "arw"
                            | "dng" | "raf" | "orf" | "rw2" | "qoi" | "exr"
                            | "hdr" | "dds" | "tga"
                    );
                    if is_image {
                        self.dispatch(AppMessage::OpenFile(path.clone()));
                    } else if path.is_dir() {
                        self.dispatch(AppMessage::OpenDirectory(path.clone()));
                    }
                }
            }
        }

        let drop_allowed = ctx.input(|i| {
            i.raw.hovered_files.iter().any(|f| {
                f.path.as_ref().map_or(false, |p| {
                    p.is_dir()
                        || p.extension().and_then(|e| e.to_str()).is_some_and(
                            |e| {
                                matches!(
                                    e.to_lowercase().as_str(),
                                    "png" | "jpg" | "jpeg" | "gif" | "bmp"
                                        | "tiff" | "tif" | "webp" | "ico"
                                        | "avif" | "heic" | "heif" | "svg"
                                )
                            },
                        )
                })
            })
        });

        if drop_allowed {
            let painter = ui.painter();
            let rect = ui.max_rect();
            painter.rect_filled(rect, 0.0, Color32::from_black_alpha(128));
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Drop images here to open them instantly",
                FontId::proportional(20.0),
                Color32::WHITE,
            );
        }
    }
}

/// Handle the Ctrl+O shortcut to open a file dialog
impl GlintApp {
    pub fn handle_ctrl_o(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "All Images",
                &[
                    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "webp",
                    "ico", "avif", "heic", "heif", "svg",
                ],
            )
            .add_filter("PNG", &["png"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .add_filter("WebP", &["webp"])
            .add_filter("BMP", &["bmp"])
            .add_filter("GIF", &["gif"])
            .add_filter("TIFF", &["tiff", "tif"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.dispatch(AppMessage::OpenFile(path));
        }
    }

    pub fn handle_ctrl_d(&self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.dispatch(AppMessage::OpenDirectory(path));
        }
    }
}
