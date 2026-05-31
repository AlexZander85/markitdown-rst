## MDrust v1.2.0 — Major GUI Fixes, Real Fonts & Performance Optimizations

This release focuses on fixing critical GUI bugs, adding real embedded fonts with CJK support, and introducing performance optimizations.

### Bug Fixes

- **Fixed TopBar icon loading every frame** — the app icon was being decoded from PNG, resized with Lanczos3, and uploaded as a GPU texture at 60 FPS. Now cached as `TextureHandle` and loaded once.
- **Fixed `preview_text` cloned every frame** — the entire Markdown output (potentially megabytes) was being `.clone()`-d at 60 FPS. Now only clones on file selection change.
- **Fixed `Rendered` preview tab showing same as `Raw`** — both tabs displayed monospace text. Now the `Rendered` tab uses `egui_commonmark` for real Markdown rendering inside the GUI.
- **Fixed `rt.block_on()` blocking UI thread** — `save_all()` was called synchronously inside `pump_channels`, freezing the UI during file saves. Now runs in a background thread.
- **Fixed `Runtime::new().unwrap()`** — replaced with `.expect("failed to create tokio runtime")` for better panic diagnostics.
- **Fixed Windows console window** — added `#![windows_subsystem = "windows"]` for GUI builds, no more terminal window on launch.
- **Fixed Windows crash on startup** — added panic handler with `MessageBoxW` so errors are shown in a dialog instead of silently crashing.

### New Features

- **Embedded professional fonts** — Inter (Regular + Bold), JetBrains Mono, and NotoSansCJK are now embedded via `include_bytes!` and registered through `egui::FontDefinitions`. CJK characters render correctly out of the box.
- **egui_commonmark integration** — the `Rendered` preview tab now renders real Markdown with headings, bold, italic, lists, code blocks, and links directly in the GUI.

### Performance Optimizations

- **blake3** — content-hash cache for skip-identical-file conversion
- **simd-json** — SIMD-accelerated JSON parsing where available
- **compact_str** — compact string storage for file paths and titles
- **smallvec** — stack-allocated small vectors in hot paths
- **ahash** — faster hashing throughout the codebase

### Downloads

| File | Edition | OS | Arch |
|------|---------|----|------|
| `mdrust-full-linux-x64.tar.gz` | Full (GUI + OCR + Preview) | Linux | x86_64 |
| `mdrust-full-macos-x64.tar.gz` | Full | macOS | x86_64 |
| `mdrust-full-windows-x64.exe` | Full | Windows | x86_64 |
| `mdrust-light-linux-x64.tar.gz` | Light (GUI, no OCR) | Linux | x86_64 |
| `mdrust-light-macos-x64.tar.gz` | Light | macOS | x86_64 |
| `mdrust-light-windows-x64.exe` | Light | Windows | x86_64 |
| `mdrust-cli-linux-x64.tar.gz` | CLI-only (OCR) | Linux | x86_64 |
| `mdrust-cli-macos-x64.tar.gz` | CLI-only | macOS | x86_64 |
| `mdrust-cli-windows-x64.exe` | CLI-only | Windows | x86_64 |

### Quick Start

```bash
# GUI mode
./mdrust

# CLI: single file
mdrust-cli convert document.pdf

# CLI: batch with 8 threads
mdrust-cli batch ./docs --threads 8 --output ./markdown

# CLI: OCR with Russian + English
mdrust-cli convert scan.png --ocr-langs eng+rus
```

---

**Full Changelog**: https://github.com/AlexZander85/MDrust/compare/v1.1.0...v1.2.0
