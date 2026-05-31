//! PDF to Markdown converter — optimized single-pass loading
//!
//! Uses a three-tier extraction strategy:
//! 1. `pdf-extract` — best quality text extraction (but may panic on some PDFs)
//! 2. `lopdf` — fallback page-by-page extraction
//! 3. Graceful error with diagnostics if both fail

use super::{ConversionResult, Converter, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;

pub struct PdfConverter;

impl PdfConverter {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl DocumentConverter for PdfConverter {
    fn format(&self) -> InputFormat { InputFormat::Pdf }

    async fn convert(&self, path: &Path, _output_format: &OutputFormat) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path)
            .await
            .with_context(|| format!("Cannot access PDF file: {}", path.display()))?
            .len();
        let path_display = path.display().to_string();
        let result = tokio::task::spawn_blocking(move || {
            extract_pdf_to_markdown(&path, file_size)
        }).await
            .with_context(|| format!("PDF conversion task crashed (likely unsupported PDF structure): {}", path_display))??;
        Ok(result)
    }
}

fn extract_pdf_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    // Load lopdf ONCE — needed for page_count and as fallback
    let doc = lopdf::Document::load(path).ok();
    let page_count = doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(1);

    // Try pdf-extract first (better quality) — catch panics because
    // pdf-extract can panic on malformed or encrypted PDFs
    let text = match extract_text_via_pdf_extract(path) {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            tracing::debug!(
                "pdf-extract failed for {}, falling back to lopdf",
                path.display()
            );
            match doc {
                Some(d) => extract_text_from_doc(&d),
                None => {
                    return Err(anyhow::anyhow!(
                        "Failed to extract text from PDF: {}. \
                         The file may be encrypted, corrupted, or contain only images. \
                         Try: 1) Remove PDF password, 2) Use OCR edition for scanned documents",
                        path.display()
                    ));
                }
            }
        }
    };

    // If both extractors returned empty text, the PDF might be scanned/image-based
    if text.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "PDF contains no extractable text: {}. \
             This typically means the PDF is a scanned image. \
             Use OCR (Tesseract) to recognize text from image-based PDFs.",
            path.display()
        ));
    }

    let (markdown, title, word_count) = text_to_markdown_with_meta(&text);

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count,
        word_count,
        source_format: InputFormat::Pdf,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown_no_recount(markdown, metadata))
}

/// Try to extract text using pdf-extract, catching panics.
///
/// `pdf-extract` can panic on certain PDF structures (e.g., encrypted files,
/// malformed streams, or unsupported compression). We use `catch_unwind`
/// to prevent the panic from killing the entire conversion task.
fn extract_text_via_pdf_extract(path: &Path) -> Option<String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pdf_extract::extract_text(path)
    }))
    .ok()
    .and_then(|result| result.ok())
}

fn extract_text_from_doc(doc: &lopdf::Document) -> String {
    let mut text = String::with_capacity(4096);
    let pages = doc.get_pages();
    for (page_num, _) in pages.iter() {
        if let Ok(page_text) = doc.extract_text(&[*page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }
    text
}

/// Single-pass: markdown + title + word_count all at once, pre-allocated
fn text_to_markdown_with_meta(text: &str) -> (String, Option<String>, usize) {
    let mut markdown = String::with_capacity(text.len() + text.len() / 16);
    let mut title: Option<String> = None;
    let mut word_count = 0usize;
    let mut prev_was_empty = true;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !prev_was_empty {
                markdown.push('\n');
            }
            prev_was_empty = true;
            continue;
        }

        word_count += crate::utils::count_words(trimmed);

        let is_header = trimmed.len() < 80
            && !matches!(trimmed.as_bytes().last(), Some(b'.' | b',' | b';'))
            && trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && prev_was_empty;

        if is_header {
            markdown.push_str("## ");
            markdown.push_str(trimmed);
            markdown.push('\n');
            if title.is_none() {
                title = Some(trimmed.to_string());
            }
        } else {
            markdown.push_str(trimmed);
            markdown.push('\n');
        }

        prev_was_empty = false;
    }

    (markdown, title, word_count)
}
