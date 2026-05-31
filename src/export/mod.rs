//! Export (reverse conversion) module — Markdown → HTML, DOCX
//!
//! This module provides converters that take Markdown text (from ConversionResult)
//! and export it to other formats. This is the "Save As" functionality.
//!
//! - **MD → HTML** — uses `comrak` for rendering (already a dependency)
//! - **MD → DOCX** — uses `pulldown-cmark` for parsing + `docx-rs` for writing

pub mod md_to_html;
pub mod md_to_docx;
