# MDrust Tessdata

This directory contains Tesseract language data files for **development/testing**.

**Starting from v2.0.0, tessdata files are NOT embedded in the binary.**
They are downloaded on demand when the user selects a language for OCR.

## Built-in OCR (ocrs)

MDrust v2.0.0+ includes a built-in OCR engine (**ocrs**) with neural network
models **embedded directly in the binary** (~11.7 MB via `include_bytes!`).
English text recognition works instantly, offline, with zero downloads.

## Tesseract (optional, 100+ languages)

For Russian, Chinese, and 100+ other languages, MDrust can optionally use
Tesseract. Tessdata files are downloaded on demand from
[tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast) and stored in:

- **Linux**: `~/.local/share/mdrust/tessdata/`
- **macOS**: `~/Library/Application Support/mdrust/tessdata/`
- **Windows**: `%APPDATA%/mdrust/tessdata/`

## Files in this directory (for development)

| File | Language | Size |
|------|----------|------|
| `eng.traineddata` | English | ~3.9 MB |
| `rus.traineddata` | Russian | ~3.7 MB |
| `chi_sim.traineddata` | Simplified Chinese | ~2.4 MB |

These files are only used for local testing. End users do not need them —
tessdata is auto-downloaded when Tesseract OCR is first used.
