# Tesseract Language Data (tessdata)

This directory contains Tesseract OCR language model files (`tessdata_fast`)
that are embedded into the MDrust binary at compile time via `include_bytes!`.

## Included Languages

| File | Language | Size |
|------|----------|------|
| `eng.traineddata` | English | ~4 MB |
| `rus.traineddata` | Russian | ~3.7 MB |
| `chi_sim.traineddata` | Simplified Chinese | ~2.4 MB |

## How It Works

1. At compile time, `src/ocr/mod.rs` includes these files via `include_bytes!`
2. On first run, MDrust extracts them to the app data directory
   (`~/.local/share/mdrust/tessdata/` on Linux,
    `~/Library/Application Support/mdrust/tessdata/` on macOS,
    `%APPDATA%\mdrust\tessdata\` on Windows)
3. The Tesseract engine uses these files for text recognition

## Requirements

The tessdata files are embedded, but the **Tesseract OCR engine** itself
(tesseract CLI binary) must be installed on the system:

- **Linux (Debian/Ubuntu)**: `sudo apt install tesseract-ocr`
- **Linux (Fedora)**: `sudo dnf install tesseract`
- **Linux (Arch)**: `sudo pacman -S tesseract`
- **macOS**: `brew install tesseract`
- **Windows**: Download from https://github.com/UB-Mannheim/tesseract/wiki
  or `choco install tesseract`

## Source

These are standard `tessdata_fast` models from the official Tesseract repository:
https://github.com/tesseract-ocr/tessdata_fast
