// Glint - A next-generation native Windows photo viewer and lightweight editor
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use anyhow::Result;
use glint::GlintApp;
use std::env;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Parse CLI arguments for installer and advanced usage
    let args: Vec<String> = env::args().collect();
    let cli_flags: Vec<&str> = args.iter().map(|s| s.as_str()).skip(1).collect();

    // Handle installer flags before starting the GUI
    #[cfg(windows)]
    {
        if cli_flags.contains(&"--register-associations") {
            glint::platform::WindowsIntegration::register_as_available()?;
            return Ok(());
        }
        if cli_flags.contains(&"--set-default") {
            glint::platform::WindowsIntegration::register_as_available()?;
            glint::platform::WindowsIntegration::set_as_default_all()?;
            return Ok(());
        }
        if cli_flags.contains(&"--open-defaults") {
            glint::platform::WindowsIntegration::open_default_apps_settings()?;
            return Ok(());
        }
        if cli_flags.contains(&"--check-default") {
            let is_default = glint::platform::WindowsIntegration::is_default_for_test_format();
            if is_default {
                println!("Glint IS the default image viewer");
            } else {
                println!("Glint is NOT the default image viewer");
            }
            return Ok(());
        }
        if cli_flags.contains(&"--unregister-associations") {
            glint::platform::WindowsIntegration::unregister_file_associations()?;
            return Ok(());
        }
        if cli_flags.contains(&"--enable-autostart") {
            glint::platform::WindowsIntegration::set_auto_start(true)?;
            return Ok(());
        }
        if cli_flags.contains(&"--disable-autostart") {
            glint::platform::WindowsIntegration::set_auto_start(false)?;
            return Ok(());
        }
    }

    log::info!("Starting Glint v{}", env!("CARGO_PKG_VERSION"));

    // Check for file path argument to open directly
    let initial_path = cli_flags
        .iter()
        .find(|&&f| !f.starts_with("--"))
        .map(|&s| s.to_string());

    // Parse mode flags
    let start_fullscreen = cli_flags.contains(&"--fullscreen");
    let start_slideshow = cli_flags.contains(&"--slideshow");
    let start_minimized = cli_flags.contains(&"--background");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Glint")
            .with_min_inner_size([800.0, 600.0])
            .with_max_inner_size([3840.0, 2160.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
                    .unwrap_or_default(),
            )
            .with_visible(!start_minimized),
        multisampling: 4,
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Glint",
        native_options,
        Box::new(move |cc| {
            let app = GlintApp::new(cc);

            // Handle initial file or directory if provided
            if let Some(ref path_str) = initial_path {
                let path = std::path::Path::new(path_str);
                if path.is_file() {
                    app.dispatch(glint::AppMessage::OpenFile(path.to_path_buf()));
                } else if path.is_dir() {
                    app.dispatch(glint::AppMessage::OpenDirectory(path.to_path_buf()));
                }
            }

            // Set initial modes
            if start_fullscreen {
                app.dispatch(glint::AppMessage::ToggleFullscreen);
            }
            if start_slideshow {
                app.dispatch(glint::AppMessage::ToggleSlideshow);
            }

            Ok(Box::new(app))
        }),
    );

    log::info!("Glint shutdown complete");

    Ok(())
}
