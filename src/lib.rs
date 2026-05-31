//! MDrust — Multi-threaded Document-to-Markdown Converter Library
//!
//! This library provides document conversion capabilities with multi-threaded
//! batch processing, optional OCR support, and optional Markdown preview.
//!
//! # Features
//! - Convert 15+ document formats to Markdown
//! - Export Markdown to HTML and DOCX
//! - Multi-threaded batch processing (tokio + semaphore)
//! - SIMD-accelerated parsing (AVX-512, AVX2, SSE4.2, NEON) with runtime detection
//! - OCR for images via Tesseract with embedded tessdata (eng/rus/chi_sim) — `ocr` feature
//! - Beautiful Markdown preview with highlight.js, KaTeX, Mermaid — `preview` feature
//! - Multilingual UI (English, Russian, Chinese)

pub mod batch;
pub mod converters;
pub mod cpu;
pub mod export;
pub mod i18n;
pub mod utils;

#[cfg(feature = "ocr")]
pub mod ocr;

#[cfg(feature = "preview")]
pub mod preview;

#[cfg(feature = "gui")]
pub mod gui;
