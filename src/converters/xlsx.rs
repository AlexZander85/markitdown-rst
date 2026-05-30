//! XLSX to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// XLSX converter using umya-spreadsheet
pub struct XlsxConverter;

impl XlsxConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for XlsxConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Xlsx
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_xlsx_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_xlsx_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let book = umya_spreadsheet::reader::xlsx::read(path)?;

    let mut markdown = String::new();
    let mut sheet_count = 0;

    for sheet in book.get_sheet_collection() {
        sheet_count += 1;
        let sheet_name = sheet.get_name();

        markdown.push_str(&format!("## {}\n\n", sheet_name));

        // Get dimensions
        let (max_col, max_row) = sheet.get_highest_column_and_row();

        if max_row == 0 || max_col == 0 {
            markdown.push_str("*Empty sheet*\n\n");
            continue;
        }

        // Build table data using get_cell_collection_sorted
        let cells = sheet.get_cell_collection_sorted();

        // Organize cells into a 2D grid
        let mut grid: std::collections::HashMap<(u32, u32), String> = std::collections::HashMap::new();
        let mut actual_max_row: u32 = 0;
        let mut actual_max_col: u32 = 0;

        for cell in &cells {
            let coord = cell.get_coordinate();
            let col = *coord.get_col_num();
            let row = *coord.get_row_num();
            let value = cell.get_value().to_string();
            if !value.trim().is_empty() {
                grid.insert((col, row), value.trim().to_string());
                actual_max_row = actual_max_row.max(row);
                actual_max_col = actual_max_col.max(col);
            }
        }

        if grid.is_empty() {
            markdown.push_str("*Empty sheet*\n\n");
            continue;
        }

        // Create markdown table
        let col_count = actual_max_col as usize;
        if col_count > 0 {
            // Header row (row 1)
            markdown.push('|');
            for c in 1..=actual_max_col {
                let val = grid.get(&(c, 1)).cloned().unwrap_or_default();
                markdown.push_str(&format!(" {} |", val));
            }
            markdown.push('\n');

            // Separator
            markdown.push('|');
            for _ in 0..actual_max_col {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');

            // Data rows (starting from row 2)
            for r in 2..=actual_max_row {
                // Check if row has any data
                let has_data = (1..=actual_max_col).any(|c| grid.contains_key(&(c, r)));
                if !has_data {
                    continue;
                }
                markdown.push('|');
                for c in 1..=actual_max_col {
                    let val = grid.get(&(c, r)).cloned().unwrap_or_default();
                    markdown.push_str(&format!(" {} |", val));
                }
                markdown.push('\n');
            }
            markdown.push('\n');
        }
    }

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title: None,
        author: None,
        page_count: sheet_count,
        word_count,
        source_format: InputFormat::Xlsx,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}
