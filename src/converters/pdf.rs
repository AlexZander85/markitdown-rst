//! PDF to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// PDF converter using lopdf and pdf-extract
pub struct PdfConverter;

impl PdfConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for PdfConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Pdf
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        // Run PDF extraction in a blocking task (CPU-intensive)
        let result = tokio::task::spawn_blocking(move || {
            extract_pdf_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_pdf_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    // Try pdf-extract first for better text extraction
    let text = match pdf_extract::extract_text(path) {
        Ok(t) if !t.trim().is_empty() => t,
        _ => {
            // Fallback to lopdf-based extraction
            extract_text_with_lopdf(path)?
        }
    };

    let page_count = estimate_page_count(path);
    let word_count = text.split_whitespace().count();

    // Convert plain text to markdown
    let markdown = text_to_markdown(&text);

    let title = extract_title(&markdown);

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count,
        word_count,
        source_format: InputFormat::Pdf,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}

fn extract_text_with_lopdf(path: &Path) -> Result<String> {
    let doc = lopdf::Document::load(path)?;
    let mut text = String::new();
    let pages = doc.get_pages();

    for (page_num, _) in pages.iter() {
        if let Ok(page_text) = doc.extract_text(&[page_num.clone()]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }

    Ok(text)
}

fn estimate_page_count(path: &Path) -> usize {
    lopdf::Document::load(path)
        .map(|doc| doc.get_pages().len())
        .unwrap_or(1)
}

fn text_to_markdown(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut markdown = String::new();
    let mut prev_was_empty = true;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !prev_was_empty {
                markdown.push('\n');
            }
            prev_was_empty = true;
            continue;
        }

        // Heuristic: short lines that look like headers
        if trimmed.len() < 80
            && !trimmed.ends_with('.')
            && !trimmed.ends_with(',')
            && !trimmed.ends_with(';')
            && trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && prev_was_empty
        {
            markdown.push_str("## ");
            markdown.push_str(trimmed);
            markdown.push('\n');
        } else {
            markdown.push_str(trimmed);
            markdown.push('\n');
        }

        prev_was_empty = false;
    }

    markdown
}

fn extract_title(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find(|l| l.starts_with("## "))
        .map(|l| l.trim_start_matches("## ").to_string())
}
