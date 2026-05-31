## MDrust v1.4.1 — PDF Conversion Fix, Honest OCR Documentation, Tessdata Cleanup

Bug fix release addressing PDF conversion failures and clarifying Tesseract OCR integration.

### Bug Fixes

- **Fixed: PDF conversion returning "0/1 files converted"** — `pdf-extract` can panic on malformed, encrypted, or image-based PDFs, which killed the entire conversion task. Now uses `catch_unwind` to safely handle panics and falls back to `lopdf` page-by-page extraction. If both methods fail, a clear error message explains why (encrypted, corrupted, or scanned PDF) and suggests using OCR for image-based documents.
- **Fixed: Tesseract status message misleading** — The GUI showed "Tesseract не установлен (языковые данные встроены, но сам tesseract нужно установить отдельно)" which was confusing. Now shows "OCR: engine not installed" with a hover tooltip explaining that tessdata is embedded but the Tesseract engine requires system installation.
- **Fixed: tessdata always extracted on startup** — Previously, tessdata was only extracted if Tesseract was detected. Now tessdata is always extracted on startup so it's ready immediately when the user installs Tesseract.

### Documentation

- **Honest OCR documentation** — README and code comments now clearly state: tessdata (language models) are embedded in the binary, but the Tesseract engine (libtesseract / tesseract CLI) requires system installation. It is technically impossible to statically link the entire Tesseract engine due to its complex C++ dependencies (leptonica, libpng, libjpeg, libtiff, etc.).
- **tessdata/README.md** — Added a README to the tessdata folder explaining the contents, how embedding works, and system requirements.
- **i18n updated** — Tesseract not-found message now clearly distinguishes between embedded tessdata and the required engine installation.

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

---

**Full Changelog**: https://github.com/AlexZander85/MDrust/compare/v1.4.0...v1.4.1
