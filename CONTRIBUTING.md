# Contributing to Glint

Hey, thanks for wanting to help out. Glint is a small project with big ambitions - to make the best native image viewer for Windows. Whether you're fixing a bug, adding a feature, or improving the docs, your help is genuinely appreciated.

## Setting up

You'll need Rust (latest stable) and Windows 10 or 11. Most of the Windows integration stuff doesn't work on Linux or macOS, but the core image viewing features are cross-platform if you want to work on those.

```bash
git clone https://github.com/solez-ai/glint.git
cd glint
cargo build --release
```

If you want faster iteration, use `cargo check` instead - it skips the linker step and is way faster.

```bash
cargo check
```

## Project structure

The code is organized by what each module does:

- `src/ui/` - All the visible interface stuff (toolbar, viewer, gallery, themes)
- `src/image/` - Loading images from disk, caching them, processing them
- `src/editor/` - The editing tools (crop, rotate, resize, color adjustments, export)
- `src/renderer/` - GPU rendering pipeline (wgpu abstraction)
- `src/metadata/` - EXIF and other image metadata parsing
- `src/thumbnail/` - The SQLite-backed thumbnail cache
- `src/browser/` - File system navigation, sorting, filtering
- `src/platform/` - Windows-specific stuff (registry, file associations, auto-start)

## What to work on

Check the Issues tab for things that need doing. If you want to add something that's not there yet, open an issue first so we can talk about it - saves everyone time.

Some things that would be really helpful:
- Adding support for more image formats
- Improving the thumbnail cache performance
- Better keyboard shortcut customization
- Accessibility improvements
- Making the installer smoother
- Improving error handling for edge cases

## Coding

Nothing too fancy. Just follow what's already in the codebase. Use `rustfmt` and `clippy` before submitting. Write tests for new functionality if you can.

```bash
cargo fmt
cargo clippy
cargo test
```

## Submitting changes

1. Fork the repo and create a branch
2. Make your changes
3. Run the checks above
4. Open a pull request with a clear description of what you changed and why

Keep commits small and focused. Write commit messages that explain what and why, not how.

## Performance matters

Glint's whole identity is being fast. If you're adding something that might slow things down, profile before and after. The hot paths are image decoding, thumbnail generation, and UI rendering. Try not to allocate memory in those paths unnecessarily.

## Questions?

Open an issue or discussion on GitHub. No question is too small.

## License

By contributing, you agree that your work will be licensed under the MIT license (same as the rest of the project).
