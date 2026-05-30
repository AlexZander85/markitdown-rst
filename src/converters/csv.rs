//! CSV/TSV to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// CSV/TSV to Markdown converter
pub struct CsvConverter;

impl CsvConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for CsvConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Csv
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_csv_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_csv_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let format = crate::utils::detect_format(path);
    let is_tsv = format == InputFormat::Tsv;

    let delimiter = if is_tsv { b'\t' } else { b',' };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_path(path)?;

    let mut markdown = String::new();

    // Title from filename
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    if let Some(ref t) = title {
        markdown.push_str(&format!("# {}\n\n", t));
    }

    // Get headers
    let headers = reader.headers()?.clone();
    let col_count = headers.len();

    if col_count == 0 {
        return Ok(ConversionResult::from_markdown(
            "*Empty CSV file*\n".to_string(),
            DocumentMetadata {
                title,
                author: None,
                page_count: 1,
                word_count: 0,
                source_format: format,
                source_path: path.display().to_string(),
                file_size_bytes: file_size,
            },
        ));
    }

    // Markdown table header
    markdown.push('|');
    for header in &headers {
        markdown.push_str(&format!(" {} |", header));
    }
    markdown.push('\n');

    // Separator
    markdown.push('|');
    for _ in 0..col_count {
        markdown.push_str(" --- |");
    }
    markdown.push('\n');

    // Data rows
    let mut row_count = 0;
    for result in reader.records() {
        let record = result?;
        markdown.push('|');
        for field in record.iter() {
            // Escape pipe characters in field values
            let escaped = field.replace('|', "\\|");
            markdown.push_str(&format!(" {} |", escaped));
        }
        markdown.push('\n');
        row_count += 1;
    }

    markdown.push('\n');
    markdown.push_str(&format!("*{} rows total*\n", row_count));

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count: 1,
        word_count,
        source_format: format,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}
