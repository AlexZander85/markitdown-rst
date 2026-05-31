# MDrust Tessdata

This directory contains Tesseract language data files that are used for OCR.

**Note:** Starting from v2.0.0, tessdata files are **NOT** embedded in the binary.
They are downloaded on demand when the user selects a language for OCR.

## Languages Available

| File | Language | Size |
|------|----------|------|
| `eng.traineddata` | English | ~3.9 MB |
| `rus.traineddata` | Russian | ~3.7 MB |
| `chi_sim.traineddata` | Simplified Chinese | ~2.4 MB |

## Download Location

Tessdata files are downloaded from the [tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast) 
repository and stored in the MDrust application data directory:

- **Linux**: `~/.local/share/mdrust/tessdata/`
- **macOS**: `~/Library/Application Support/mdrust/tessdata/`
- **Windows**: `%APPDATA%/mdrust/tessdata/`

## Built-in OCR

MDrust v2.0.0 includes a built-in OCR engine (ocrs) that supports English 
text recognition without any external dependencies or downloads. Tesseract 
is optional and provides support for 100+ additional languages.
