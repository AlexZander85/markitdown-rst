# MDrust v2.0.0 — Major Release

## 🚀 Built-in OCR Engine (ocrs)

MDrust now includes **ocrs** — a pure Rust OCR engine — directly in the binary. No external dependencies required for English text recognition.

- **Zero dependencies**: No Tesseract, no C/C++ libraries, no installation needed
- **Works out of the box**: English OCR is always available in Full edition
- **Cross-platform**: Compiles and runs on Linux, macOS, and Windows without issues

## 📚 Optional Tesseract Integration

For 100+ languages (Russian, Chinese, Arabic, etc.), MDrust can optionally use Tesseract:

- **Auto-download**: Click "Install Tesseract" button in the sidebar
- **On-demand tessdata**: Language data is downloaded only when needed (not embedded in binary)
- **Smart fallback**: Uses ocrs for English, Tesseract for other languages

## 📄 Scanned PDF Support

New PDF-to-image OCR pipeline for scanned documents:

- **pdfium-render** renders PDF pages to high-resolution images
- **ocrs/Tesseract** then recognizes text from those images
- Automatic fallback: text extraction → lopdf → PDF rendering + OCR

## 🏗️ Architecture Changes

- **Removed `include_bytes!` tessdata**: Binary size reduced by ~10 MB
- **Added `ocrs` + `rten`**: Pure Rust OCR engine (built-in)
- **Added `pdfium-render`**: PDF rendering for scanned documents
- **Added `reqwest`**: HTTP client for downloading models and tessdata on demand
- **Added `glow` feature** to eframe: GPU fallback renderer now works
- **New feature flag `pdf-to-image`**: Enables PDF → image → OCR pipeline

## 🐛 Bug Fixes

- Fixed PDF conversion returning "0/1 files converted" for text-based PDFs
- Fixed Tesseract status not refreshing after installation
- Fixed eframe glow fallback not working (missing feature flag)
- Fixed CI workflow feature syntax (quoted feature lists)

## 📦 Downloads

| Edition | Description | OCR | Preview |
|---------|-------------|-----|---------|
| **Full** | GUI + OCR + Preview + PDF-to-Image | ocrs + Tesseract | Yes |
| **Light** | GUI only | No | No |
| **CLI** | Command-line only | ocrs + Tesseract | No |
