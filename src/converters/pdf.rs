//! PDF to Markdown converter — optimized single-pass loading

use super::{ConversionResult, Converter, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
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
        let file_size = tokio::fs::metadata(&path).await?.len();
        let result = tokio::task::spawn_blocking(move || {
            extract_pdf_to_markdown(&path, file_size)
        }).await??;
        Ok(result)
    }
}

fn extract_pdf_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    // Load lopdf ONCE — needed for page_count and as fallback
    let doc = lopdf::Document::load(path).ok();
    let page_count = doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(1);

    // Try pdf-extract first (better quality)
    let text = match pdf_extract::extract_text(path) {
        Ok(t) if !t.trim().is_empty() => t,
        _ => match doc {
            Some(d) => extract_text_from_doc(&d),
            None => return Err(anyhow::anyhow!("Failed to load PDF: {}", path.display())),
        },
    };

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
