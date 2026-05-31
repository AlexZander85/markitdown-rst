## MDrust v1.5.0 — CI Fix, Remove tesseract-ffi, macOS ARM64 Native

Major release fixing CI/CD pipeline failures and simplifying OCR integration.

### Breaking Changes

- **Removed `tesseract-ffi` feature** — The `tesseract-ffi` Cargo feature has been removed. OCR now uses the Tesseract CLI subprocess exclusively. This eliminates all compile-time C/C++ dependencies (libtesseract-dev, libleptonica-dev, vcpkg) while providing identical OCR functionality at runtime. The `ocr` feature still works — it just calls the `tesseract` CLI instead of linking to libtesseract via FFI.

### Bug Fixes

- **Fixed: CI builds failing on macOS and Windows** — `leptonica-sys` and `tesseract-sys` could not compile on macOS (pkg-config cross-compilation error) and Windows (vcpkg not found). Removing the FFI dependency fixes builds on all platforms.
- **Fixed: macOS target architecture** — Changed from `x86_64-apple-darwin` to `aarch64-apple-darwin` since `macos-latest` GitHub Actions runners are now Apple Silicon (M1/M2). macOS binaries are now native ARM64 (`macos-arm64` suffix).
- **Fixed: PDF conversion returning "0/1 files converted"** (carried from v1.4.1) — `pdf-extract` can panic on malformed/encrypted/image-based PDFs, killing the conversion task. Now uses `catch_unwind` to safely handle panics and falls back to `lopdf` page-by-page extraction.
- **Fixed: Tesseract status message** — GUI now clearly shows "OCR: engine not installed" with a hover tooltip explaining that tessdata is embedded but the Tesseract CLI requires system installation.

### Improvements

- **Zero compile-time C dependencies for OCR** — Building with OCR support no longer requires libtesseract-dev, libleptonica-dev, pkg-config, or vcpkg. Only the Rust toolchain is needed.
- **tessdata always extracted on startup** — Language data files are extracted on first run regardless of whether Tesseract is installed, so they're ready immediately when the user installs Tesseract later.
- **Cleaned up tessdata folder** — Removed outdated references to the old project name "markitdown-rs".

### Downloads

| File | Edition | OS | Arch |
|------|---------|----|------|
| `mdrust-full-linux-x64.tar.gz` | Full (GUI + OCR + Preview) | Linux | x86_64 |
| `mdrust-full-macos-arm64.tar.gz` | Full | macOS | ARM64 |
| `mdrust-full-windows-x64.exe` | Full | Windows | x86_64 |
| `mdrust-light-linux-x64.tar.gz` | Light (GUI, no OCR) | Linux | x86_64 |
| `mdrust-light-macos-arm64.tar.gz` | Light | macOS | ARM64 |
| `mdrust-light-windows-x64.exe` | Light | Windows | x86_64 |
| `mdrust-cli-linux-x64.tar.gz` | CLI-only (OCR) | Linux | x86_64 |
| `mdrust-cli-macos-arm64.tar.gz` | CLI-only | macOS | ARM64 |
| `mdrust-cli-windows-x64.exe` | CLI-only | Windows | x86_64 |

---

**Full Changelog**: https://github.com/AlexZander85/MDrust/compare/v1.4.0...v1.5.0
