//! MarkItDown-RS — Multi-threaded Document-to-Markdown Converter Library
//!
//! This library provides document conversion capabilities with multi-threaded
//! batch processing, optional OCR support, and optional Markdown preview.
//!
//! # Features
//! - Convert 15+ document formats to Markdown
//! - Multi-threaded batch processing (tokio + semaphore)
//! - OCR for images via Tesseract with embedded tessdata (eng/rus/chi_sim) — `ocr` feature
//! - Beautiful Markdown preview with highlight.js, KaTeX, Mermaid — `preview` feature
//! - Multilingual UI (English, Russian, Chinese)

pub mod batch;
pub mod converters;
pub mod i18n;
pub mod utils;

#[cfg(feature = "ocr")]
pub mod ocr;

#[cfg(feature = "preview")]
pub mod preview;
