//! JSON to Markdown converter
//!
//! Reads JSON files and pretty-prints them in fenced code blocks.
//! If the JSON is an array of objects, also generates a markdown table
//! from the first-level keys of the objects.

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// JSON to Markdown converter
pub struct JsonConverter;

impl JsonConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for JsonConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Json
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_json_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_json_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let raw = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let mut markdown = String::new();

    // Title from filename
    if let Some(ref t) = title {
        markdown.push_str(&format!("# {}\n\n", t));
    }

    // Pretty-print the full JSON in a fenced code block
    let pretty = serde_json::to_string_pretty(&value)?;
    markdown.push_str("```json\n");
    markdown.push_str(&pretty);
    markdown.push_str("\n```\n\n");

    // If the JSON is an array of objects, also generate a markdown table
    if let serde_json::Value::Array(items) = &value {
        if !items.is_empty() && items.iter().all(|v| v.is_object()) {
            markdown.push_str("## Tabular View\n\n");

            // Collect ordered keys from the first object
            let first_obj = items[0].as_object().unwrap();
            let keys: Vec<String> = first_obj.keys().cloned().collect();

            if !keys.is_empty() {
                // Table header
                markdown.push('|');
                for key in &keys {
                    markdown.push_str(&format!(" {} |", key));
                }
                markdown.push('\n');

                // Separator
                markdown.push('|');
                for _ in &keys {
                    markdown.push_str(" --- |");
                }
                markdown.push('\n');

                // Data rows
                for item in items {
                    let obj = item.as_object().unwrap();
                    markdown.push('|');
                    for key in &keys {
                        let cell = match obj.get(key) {
                            Some(v) => format_json_value(v),
                            None => String::new(),
                        };
                        // Escape pipe characters in cell values
                        let escaped = cell.replace('|', "\\|");
                        markdown.push_str(&format!(" {} |", escaped));
                    }
                    markdown.push('\n');
                }

                markdown.push('\n');
                markdown.push_str(&format!("*{} rows total*\n", items.len()));
            }
        }
    }

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count: 1,
        word_count,
        source_format: InputFormat::Json,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}

/// Format a JSON value as a human-readable string for table cells.
/// Strings are shown without quotes; other types use their JSON representation.
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        // For nested objects / arrays, show compact JSON
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
        // Numbers, bools — use their JSON representation
        _ => value.to_string(),
    }
}
