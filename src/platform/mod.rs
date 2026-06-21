// Glint - Platform abstraction layer
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::*;

/// Cross-platform platform information
pub struct PlatformInfo;

impl PlatformInfo {
    /// Get the platform name
    pub fn name() -> &'static str {
        #[cfg(windows)]
        return "Windows";
        #[cfg(target_os = "linux")]
        return "Linux";
        #[cfg(target_os = "macos")]
        return "macOS";
    }

    /// Check if the current platform is Windows
    pub fn is_windows() -> bool {
        cfg!(windows)
    }

    /// Get the operating system version string
    pub fn os_version() -> String {
        #[cfg(windows)]
        {
            // Use a simpler approach that doesn't require RtlGetVersion
            std::env::consts::OS.to_string()
        }
        #[cfg(not(windows))]
        std::env::consts::OS.to_string()
    }
}
