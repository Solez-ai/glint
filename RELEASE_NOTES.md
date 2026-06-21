# v0.1.0 - Initial Release

This is the first public release of Glint -- a native Windows photo viewer built from the ground up in Rust. The goal is simple: be the fastest, cleanest way to look at photos on Windows, without all the bloat that modern photo apps have.

## What's inside

- Instant startup and image loading (under 150ms)
- GPU-accelerated rendering via DirectX 12 / Vulkan
- Smooth zoom, pan, and navigation
- Fullscreen mode with double-click toggle
- Slideshow mode with configurable timing
- Folder browsing with automatic thumbnail generation
- SQLite-backed thumbnail cache for fast directory navigation
- File system watching for live gallery updates

## Supported formats

PNG, JPEG, WebP, GIF, BMP, TIFF, SVG, ICO, AVIF, HEIC, HEIF, plus RAW formats from Canon, Nikon, Sony, Fujifilm, Olympus, and Panasonic. Also handles QOI, EXR, HDR, DDS, and TGA.

## Basic editing

Crop with aspect ratio presets, rotate, flip, resize, adjust brightness/contrast/saturation, blur, sharpen, convert to grayscale, and export as PNG, JPEG, WebP, BMP, or TIFF with quality control.

## Windows integration

Glint registers itself as the default viewer for all image formats. File associations are written to the registry. Context menu entries are added for both files and folders. It can auto-start with Windows and syncs with the system dark/light mode preference.

## Hidden features

Slideshow (F5), image info overlay (Ctrl+I), theme cycling (Ctrl+T), drag-and-drop to open, middle-mouse pan, scroll to zoom, and more. Check the README for the full keyboard shortcut list.

## What's missing (coming soon)

- RAW camera format processing improvements
- Batch operations (resize, rename, convert)
- Plugin system for extensibility
- Video preview support
- Accessibility improvements
- Linux and macOS ports

## Installation

Download `Glint-Setup-x64.exe` from the release assets and run it. The installer sets up file associations and adds Glint to your Start menu. You can also build from source if you prefer.

## Credits

Built by Samin Yeasar (github.com/solez-ai). MIT licensed. No tracking, no telemetry, no accounts needed. Just a fast image viewer.
