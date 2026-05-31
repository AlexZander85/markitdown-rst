# MDrust v2.0.1 — Bugfix & Enhancement Release

## Embedded OCR Models — Truly Offline

The built-in **ocrs** neural network models (text-detection + text-recognition,
~11.7 MB total) are now **embedded directly in the binary** via `include_bytes!`
and loaded from memory with `rten::Model::load_static_slice`.

- **No downloads needed**: English OCR works instantly, offline, zero network access
- **No temp files**: Models load directly from binary memory — no disk I/O overhead
- **No contradictions**: "Works out of the box" now actually means out of the box

## Startup Optimization

- Removed `ensure_tessdata()` call on startup — tessdata for Tesseract is now
  downloaded only when Tesseract OCR is actually used, not on app launch
- Faster app startup: no network calls during initialization

## Architecture

| Component | Location | Size Impact |
|-----------|----------|-------------|
| ocrs models (det + rec) | Embedded in binary (`include_bytes!`) | +11.7 MB |
| ocrs engine code | Compiled in (pure Rust) | +0 MB (code) |
| Tesseract tessdata | Downloaded on demand | 0 MB in binary |
| pdfium-render | Optional feature (`pdf-to-image`) | +library |

## OCR Engine Summary

| Engine | Languages | Install needed? | How it works |
|--------|-----------|----------------|--------------|
| **ocrs** | English | No — built-in + models embedded | `include_bytes!` → `load_static_slice` |
| **Tesseract** | 100+ | Yes — CLI + tessdata | Subprocess + on-demand download |

## Previous: v2.0.0 Changes

- Built-in ocrs OCR engine (pure Rust, English only)
- Optional Tesseract CLI for 100+ languages with one-click install
- pdfium-render for scanned PDF support (3-tier fallback)
- Removed embedded tessdata from binary (-10 MB)
- Fixed PDF "0/1 files converted" bug
- Fixed eframe glow fallback
- Fixed CI workflow feature syntax

## Downloads

| Edition | Description | OCR | Preview |
|---------|-------------|-----|---------|
| **Full** | GUI + OCR + Preview + PDF-to-Image | ocrs (embedded) + Tesseract (optional) | Yes |
| **Light** | GUI only | No | No |
| **CLI** | Command-line only | ocrs (embedded) + Tesseract (optional) | No |
