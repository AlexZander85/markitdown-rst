//! ZIP to Markdown converter
//!
//! Opens ZIP files using the `zip` crate, lists all files in the archive with
//! sizes, generates statistics (total files, total size, file-type breakdown),
//! creates a markdown index, and for each supported text-based format inside
//! the ZIP, extracts and includes the content.

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{self, InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Text-based file extensions that we attempt to extract content from
const SUPPORTED_TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "html", "xml", "rtf",
];

/// ZIP to Markdown converter
pub struct ZipConverter;

impl ZipConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for ZipConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Zip
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_zip_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_zip_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let data = std::fs::read(path)?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&data))?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let mut markdown = String::new();

    // Title from filename
    if let Some(ref t) = title {
        markdown.push_str(&format!("# {}\n\n", t));
    }

    // --- Collect file entries ---
    let total_files = archive.len();
    let mut entries: Vec<ZipEntryInfo> = Vec::with_capacity(total_files);
    let mut total_uncompressed: u64 = 0;
    let mut extension_counts: HashMap<String, usize> = HashMap::new();

    for i in 0..total_files {
        let file = archive.by_index(i)?;
        let name = file.name().to_string();
        let size = file.size();
        let is_dir = file.is_dir();

        total_uncompressed += size;

        if !is_dir {
            let ext = Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_else(|| "(no ext)".to_string());

            *extension_counts.entry(ext.clone()).or_insert(0) += 1;

            entries.push(ZipEntryInfo {
                index: i,
                name,
                size,
                is_dir,
                ext,
            });
        } else {
            entries.push(ZipEntryInfo {
                index: i,
                name,
                size,
                is_dir,
                ext: String::new(),
            });
        }
    }

    // --- Statistics section ---
    markdown.push_str("## Statistics\n\n");
    markdown.push_str(&format!("- **Total entries**: {}\n", total_files));
    markdown.push_str(&format!(
        "- **Files** (non-directory): {}\n",
        entries.iter().filter(|e| !e.is_dir).count()
    ));
    markdown.push_str(&format!(
        "- **Total uncompressed size**: {}\n",
        utils::format_size(total_uncompressed)
    ));
    markdown.push_str(&format!(
        "- **Archive size**: {}\n",
        utils::format_size(file_size)
    ));

    // File-type breakdown
    if !extension_counts.is_empty() {
        markdown.push_str("\n### File Types\n\n");
        markdown.push_str("| Extension | Count |\n");
        markdown.push_str("| --- | --- |\n");
        // Sort by count descending
        let mut ext_vec: Vec<_> = extension_counts.into_iter().collect();
        ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
        for (ext, count) in &ext_vec {
            markdown.push_str(&format!("| {} | {} |\n", ext, count));
        }
        markdown.push('\n');
    }

    // --- File index section ---
    markdown.push_str("## Archive Contents\n\n");
    markdown.push_str("| # | Name | Size |\n");
    markdown.push_str("| --- | --- | --- |\n");
    for entry in &entries {
        let size_str = if entry.is_dir {
            "-".to_string()
        } else {
            utils::format_size(entry.size)
        };
        markdown.push_str(&format!(
            "| {} | {} | {} |\n",
            entry.index + 1,
            escape_pipe(&entry.name),
            size_str
        ));
    }
    markdown.push('\n');

    // --- Extract content from supported text-based files ---
    let mut extracted_count = 0;
    for entry in &entries {
        if entry.is_dir {
            continue;
        }

        let ext_lower = entry.ext.to_lowercase();
        let is_supported = SUPPORTED_TEXT_EXTENSIONS
            .iter()
            .any(|supported| ext_lower == *supported);

        if !is_supported {
            continue;
        }

        let mut file = archive.by_index(entry.index)?;
        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            // Not valid UTF-8 — skip
            continue;
        }

        // Truncate very large files to keep output manageable
        const MAX_CONTENT_LEN: usize = 50_000;
        let truncated = if content.len() > MAX_CONTENT_LEN {
            let mut s = content[..MAX_CONTENT_LEN].to_string();
            s.push_str("\n\n*... (truncated)*");
            s
        } else {
            content
        };

        extracted_count += 1;
        markdown.push_str(&format!("## File: `{}`\n\n", entry.name));

        // Choose an appropriate fenced code-block language hint
        let lang = lang_hint(&ext_lower);
        markdown.push_str(&format!("```{}\n{}\n```\n\n", lang, truncated));
    }

    if extracted_count > 0 {
        markdown.push_str(&format!("*{} text file(s) extracted*\n", extracted_count));
    }

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count: 1,
        word_count,
        source_format: InputFormat::Zip,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}

/// Lightweight info record gathered during the first pass over the archive.
struct ZipEntryInfo {
    index: usize,
    name: String,
    size: u64,
    is_dir: bool,
    ext: String,
}

/// Escape pipe characters for safe inclusion in markdown tables.
fn escape_pipe(s: &str) -> String {
    s.replace('|', "\\|")
}

/// Return a fenced-code-block language hint for a given file extension.
fn lang_hint(ext: &str) -> &str {
    match ext {
        "json" => "json",
        "html" | "htm" => "html",
        "xml" => "xml",
        "csv" => "csv",
        "md" | "markdown" => "markdown",
        "rtf" => "rtf",
        _ => "",
    }
}
