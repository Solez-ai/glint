// Glint - Windows platform integration
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use windows::Win32::UI::Shell::IApplicationAssociationRegistration;
#[cfg(windows)]
use windows::Win32::UI::Shell::SHCNE_ASSOCCHANGED;
#[cfg(windows)]
use windows::Win32::UI::Shell::SHCNF_IDLIST;
#[cfg(windows)]
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, CoInitializeEx, COINIT_APARTMENTTHREADED};
#[cfg(windows)]
use windows::core::BSTR;

/// The CLSID for ApplicationAssociationRegistration COM object
/// {591209c7-767b-42b1-b8f1-7d8b0135482d}
#[cfg(windows)]
const CLSID_APP_ASSOC_REG: windows::core::GUID = windows::core::GUID {
    data1: 0x591209c7,
    data2: 0x767b,
    data3: 0x42b1,
    data4: [0xb8, 0xf1, 0x7d, 0x8b, 0x01, 0x35, 0x48, 0x2d],
};

/// Windows-specific shell integration for Glint
/// Handles file associations, auto-start, context menus, and native Windows integration
pub struct WindowsIntegration;

impl WindowsIntegration {
    /// Register Glint as an available handler for all supported image formats.
    /// This adds Glint to the "Open with" list and creates proper ProgID entries.
    /// Use `set_as_default_all()` to also make it the actual default.
    pub fn register_as_available() -> Result<()> {
        log::info!("Registering Glint as available image viewer...");

        Self::create_progid()?;
        for ext in Self::get_registered_formats() {
            Self::register_format(&ext)?;
        }
        Self::register_capabilities()?;

        Ok(())
    }

    /// Set Glint as the default viewer for ALL registered image formats using
    /// the official Windows IApplicationAssociationRegistration COM API.
    /// This writes the proper UserChoice hash that modern Windows respects.
    ///
    /// NOTE: This requires the process to be running with administrator privileges.
    /// If not elevated, it will return an error and you should fall back to
    /// prompting the user or opening the Default Apps settings page.
    #[cfg(windows)]
    pub fn set_as_default_all() -> Result<()> {
        log::info!("Setting Glint as default image viewer for all formats...");

        // First ensure we're registered as a capable application
        Self::register_as_available()?;

        // Initialize COM for this thread (safe to call multiple times)
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        // Create the IApplicationAssociationRegistration COM object
        unsafe {
            let registration: Result<IApplicationAssociationRegistration, _> =
                CoCreateInstance(&CLSID_APP_ASSOC_REG, None, CLSCTX_INPROC_SERVER);

            match registration {
                Ok(reg) => {
                    let app_id = BSTR::from("Glint");
                    // SetAppAsDefaultAll sets the app as default for all registered
                    // extensions. It may require elevation.
                    reg.SetAppAsDefaultAll(&app_id)
                        .context("SetAppAsDefaultAll COM call failed - try running as administrator")?;

                    log::info!("Glint successfully set as default image viewer via COM API");
                }
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create ApplicationAssociationRegistration COM object. \
                         This usually means the process is not running as administrator. \
                         Error: {}",
                        e
                    );
                }
            }
        }

        // Notify Windows of the association change
        Self::notify_shell();

        log::info!("Glint set as default image viewer completed");
        Ok(())
    }

    /// Non-Windows fallback
    #[cfg(not(windows))]
    pub fn set_as_default_all() -> Result<()> {
        log::warn!("Setting as default is only supported on Windows");
        Ok(())
    }

    /// Set Glint as the default viewer using ONLY the COM API call.
    /// DOES NOT call register_as_available() first — the caller should
    /// have already registered associations.
    /// Used by the background thread so the UI doesn't freeze.
    #[cfg(windows)]
    pub fn set_as_default_only() -> Result<()> {
        log::info!("Setting Glint as default via COM API (associations already registered)...");

        // Initialize COM for this thread
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        unsafe {
            let registration: Result<IApplicationAssociationRegistration, _> =
                CoCreateInstance(&CLSID_APP_ASSOC_REG, None, CLSCTX_INPROC_SERVER);

            match registration {
                Ok(reg) => {
                    let app_id = BSTR::from("Glint");
                    reg.SetAppAsDefaultAll(&app_id)
                        .context("SetAppAsDefaultAll COM call failed - try running as administrator")?;
                    log::info!("Glint successfully set as default image viewer via COM API");
                }
                Err(e) => {
                    anyhow::bail!(
                        "Failed to create ApplicationAssociationRegistration COM object. \
                         This usually means the process is not running as administrator. \
                         Error: {}",
                        e
                    );
                }
            }
        }

        Self::notify_shell();
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_as_default_only() -> Result<()> {
        log::warn!("Setting as default is only supported on Windows");
        Ok(())
    }

    /// Open the Windows Default Apps settings page so the user can manually
    /// set Glint as the default for image types.
    pub fn open_default_apps_settings() -> Result<()> {
        log::info!("Opening Default Apps settings page...");
        let _ = Command::new("cmd")
            .args(&["/c", "start", "ms-settings:defaultapps"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .context("Failed to open Default Apps settings")?;
        Ok(())
    }

    /// Check if Glint is currently set as the default for a test extension (.png)
    /// Uses the COM API to query the current default.
    #[cfg(windows)]
    pub fn is_default_for_test_format() -> bool {
        // Check the UserChoice registry key which stores the actual default
        // for a file extension (set via "Always use this app")
        // NOTE: Do NOT use Stdio::null() on stdout - we need to read the output!
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\FileExts\\.png\\UserChoice",
                "/v", "Progid",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("Glint.ImageViewer")
            }
            _ => false,
        }
    }

    #[cfg(not(windows))]
    pub fn is_default_for_test_format() -> bool {
        false
    }

    // =========================================================================
    // Registry helpers
    // =========================================================================

    /// Create the Glint.ImageViewer ProgID in registry
    fn create_progid() -> Result<()> {
        let app_path = Self::get_app_path();

        // Main ProgID entry
        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer",
              "/ve", "/t", "REG_SZ", "/d", "Glint Image Viewer",
              "/f"])?;

        // DefaultIcon
        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer\\DefaultIcon",
              "/ve", "/t", "REG_SZ",
              "/d", &format!("{},1", app_path),
              "/f"])?;

        // Open command
        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer\\shell\\open\\command",
              "/ve", "/t", "REG_SZ",
              "/d", &format!("\"{}\" \"%1\"", app_path),
              "/f"])?;

        // Edit command
        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer\\shell\\edit\\command",
              "/ve", "/t", "REG_SZ",
              "/d", &format!("\"{}\" \"%1\"", app_path),
              "/f"])?;

        // Slideshow verb
        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer\\shell\\slideshow",
              "/ve", "/t", "REG_SZ", "/d", "Start &slideshow",
              "/f"])?;

        step_reg("add",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer\\shell\\slideshow\\command",
              "/ve", "/t", "REG_SZ",
              "/d", &format!("\"{}\" --slideshow \"%1\"", app_path),
              "/f"])?;

        // "Open with &Glint" context menu entry for directories
        step_reg("add",
            &["HKCU\\Software\\Classes\\Directory\\shell\\Glint",
              "/ve", "/t", "REG_SZ", "/d", "Open with &Glint",
              "/f"])?;

        step_reg("add",
            &["HKCU\\Software\\Classes\\Directory\\shell\\Glint\\command",
              "/ve", "/t", "REG_SZ",
              "/d", &format!("\"{}\" \"%1\"", app_path),
              "/f"])?;

        Ok(())
    }

    /// Register a single file extension to be opened with Glint
    fn register_format(ext: &str) -> Result<()> {
        // Set the ProgID for this extension
        step_reg("add",
            &[&format!("HKCU\\Software\\Classes\\{}", ext),
              "/ve", "/t", "REG_SZ", "/d", "Glint.ImageViewer",
              "/f"])?;

        // Add PerceivedType
        step_reg("add",
            &[&format!("HKCU\\Software\\Classes\\{}", ext),
              "/v", "PerceivedType", "/t", "REG_SZ", "/d", "image",
              "/f"])?;

        // Add Content Type
        let content_type = match ext {
            ".png" => "image/png",
            ".jpg" | ".jpeg" => "image/jpeg",
            ".gif" => "image/gif",
            ".bmp" => "image/bmp",
            ".tiff" | ".tif" => "image/tiff",
            ".webp" => "image/webp",
            ".ico" => "image/x-icon",
            ".avif" => "image/avif",
            ".heic" | ".heif" => "image/heic",
            ".svg" => "image/svg+xml",
            _ => "image/unknown",
        };
        let _ = step_reg("add",
            &[&format!("HKCU\\Software\\Classes\\{}", ext),
              "/v", "Content Type", "/t", "REG_SZ", "/d", content_type,
              "/f"]);

        // Add OpenWithList so Glint appears in the "Open with..." menu
        let owl_key = format!("HKCU\\Software\\Classes\\{}\\OpenWithList\\glint.exe", ext);
        let _ = step_reg("add",
            &[&owl_key,
              "/ve", "/t", "REG_SZ", "/f"]);

        log::debug!("Registered extension {}", ext);
        Ok(())
    }

    /// Register Glint in the Windows Default Programs system (RegisteredApplications)
    fn register_capabilities() -> Result<()> {
        let cap_path = "HKCU\\Software\\Glint\\Capabilities";

        step_reg("add",
            &[cap_path,
              "/v", "ApplicationName", "/t", "REG_SZ", "/d", "Glint",
              "/f"])?;

        step_reg("add",
            &[cap_path,
              "/v", "ApplicationDescription", "/t", "REG_SZ",
              "/d", "A fast, native image viewer and lightweight editor for Windows",
              "/f"])?;

        // Register in RegisteredApplications (makes Glint visible in
        // the Default Apps settings page under "Set defaults by app")
        step_reg("add",
            &["HKCU\\Software\\RegisteredApplications",
              "/v", "Glint", "/t", "REG_SZ",
              "/d", "Software\\Glint\\Capabilities",
              "/f"])?;

        Ok(())
    }

    // =========================================================================
    // Auto-start
    // =========================================================================

    /// Register Glint to auto-start when the user logs in
    pub fn register_auto_start(enable: bool) -> Result<()> {
        let app_path = Self::get_app_path();

        if enable {
            step_reg("add",
                &["HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                  "/v", "Glint", "/t", "REG_SZ",
                  "/d", &format!("\"{}\" --background", app_path),
                  "/f"])?;
            log::info!("Glint auto-start enabled");
        } else {
            step_reg("delete",
                &["HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                  "/v", "Glint",
                  "/f"])?;
            log::info!("Glint auto-start disabled");
        }

        Ok(())
    }

    // =========================================================================
    // Unregistration
    // =========================================================================

    /// Remove all Glint file associations
    pub fn unregister_file_associations() -> Result<()> {
        log::info!("Removing Glint file associations...");

        let _ = step_reg("delete",
            &["HKCU\\Software\\Classes\\Glint.ImageViewer", "/f"]);

        for ext in Self::get_registered_formats() {
            let _ = step_reg("delete",
                &[&format!("HKCU\\Software\\Classes\\{}", ext), "/ve", "/f"]);
            let owl_key = format!("HKCU\\Software\\Classes\\{}\\OpenWithList\\glint.exe", ext);
            let _ = step_reg("delete",
                &[&owl_key, "/f"]);
        }

        let _ = step_reg("delete",
            &["HKCU\\Software\\RegisteredApplications", "/v", "Glint", "/f"]);
        let _ = step_reg("delete",
            &["HKCU\\Software\\Glint", "/f"]);
        let _ = step_reg("delete",
            &["HKCU\\Software\\Classes\\Directory\\shell\\Glint", "/f"]);

        Self::notify_shell();
        log::info!("Glint file associations removed");
        Ok(())
    }

    // =========================================================================
    // Shell notification
    // =========================================================================

    /// Notify Windows shell of file association changes using SHChangeNotify
    #[cfg(windows)]
    fn notify_shell() {
        unsafe {
            windows::Win32::UI::Shell::SHChangeNotify(
                SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None
            );
        }
    }

    #[cfg(not(windows))]
    fn notify_shell() {}

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Get the path to the Glint executable
    fn get_app_path() -> String {
        std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "glint.exe".to_string())
    }

    /// Get all registered image file extensions
    pub fn get_registered_formats() -> Vec<String> {
        vec![
            ".png".to_string(), ".jpg".to_string(), ".jpeg".to_string(),
            ".gif".to_string(), ".bmp".to_string(),
            ".tiff".to_string(), ".tif".to_string(),
            ".webp".to_string(), ".ico".to_string(), ".avif".to_string(),
            ".heic".to_string(), ".heif".to_string(),
            ".svg".to_string(),
            ".cr2".to_string(), ".cr3".to_string(),
            ".nef".to_string(), ".arw".to_string(),
            ".dng".to_string(), ".raf".to_string(),
            ".orf".to_string(), ".rw2".to_string(), ".pef".to_string(),
            ".qoi".to_string(), ".exr".to_string(),
            ".hdr".to_string(), ".dds".to_string(), ".tga".to_string(),
        ]
    }

    // =========================================================================
    // Utility
    // =========================================================================

    /// Open the file location in Windows Explorer
    pub fn open_in_explorer(path: &Path) -> Result<()> {
        let target = path.parent().unwrap_or(path);
        Command::new("explorer")
            .arg(&target)
            .spawn()
            .context("Failed to open Explorer")?;
        Ok(())
    }

    /// Open the file's properties dialog
    pub fn open_properties(path: &Path) -> Result<()> {
        let safe_path = path.to_string_lossy().replace("'", "''");
        let _ = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-WindowStyle", "Hidden",
                "-Command",
                &format!(
                    "(New-Object -ComObject Shell.Application).Namespace(0).ParseName('{}').InvokeVerb('Properties')",
                    safe_path
                ),
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn();
        Ok(())
    }

    /// Check if Windows is in dark mode
    pub fn is_windows_dark_mode() -> bool {
        // NOTE: Do NOT use Stdio::null() on stdout - we need to read the output!
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v", "AppsUseLightTheme",
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("0x0")
            }
            Err(_) => false,
        }
    }

    /// Enable or disable auto-start (convenience wrapper)
    pub fn set_auto_start(enabled: bool) -> Result<()> {
        Self::register_auto_start(enabled)
    }

    /// Check if auto-start is currently enabled
    pub fn is_auto_start_enabled() -> bool {
        let output = Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v", "Glint",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
}

/// Helper to run a reg.exe command silently (no console window flashes).
fn step_reg(operation: &str, args: &[&str]) -> Result<()> {
    let mut cmd_args = vec![operation];
    cmd_args.extend_from_slice(args);
    let output = Command::new("reg")
        .args(&cmd_args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .with_context(|| format!("reg {} failed", operation))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let msg = if stderr.is_empty() {
            format!("reg {} failed with exit code {:?}", operation, output.status.code())
        } else {
            format!("reg {} failed: {}", operation, stderr)
        };
        anyhow::bail!("{}", msg);
    }
    Ok(())
}

/// Windows utility functions
pub mod win_utils {
    /// Get the Windows build number (for feature detection)
    pub fn windows_build() -> u32 {
        let output = std::process::Command::new("cmd")
            .args(&["/c", "ver"])
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(version_str) = stdout.split_whitespace()
                    .find(|s| s.contains('.')) {
                    if let Some(build) = version_str.rsplit('.').next() {
                        return build.trim_end_matches(']').parse().unwrap_or(0);
                    }
                }
                0
            }
            Err(_) => 0,
        }
    }

    /// Check if running on Windows 11 (build 22000+)
    pub fn is_windows_11() -> bool {
        windows_build() >= 22000
    }
}
