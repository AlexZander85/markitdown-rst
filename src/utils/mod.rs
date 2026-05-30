//! Utility functions for format detection, path handling, and internationalization

use std::path::Path;

/// Supported input file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InputFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Html,
    Xml,
    Txt,
    Csv,
    Tsv,
    Rtf,
    Odt,
    Json,
    Zip,
    Image,
    Markdown,
    Unknown,
}

impl std::fmt::Display for InputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputFormat::Pdf => write!(f, "PDF"),
            InputFormat::Docx => write!(f, "DOCX"),
            InputFormat::Xlsx => write!(f, "XLSX"),
            InputFormat::Pptx => write!(f, "PPTX"),
            InputFormat::Html => write!(f, "HTML"),
            InputFormat::Xml => write!(f, "XML"),
            InputFormat::Txt => write!(f, "TXT"),
            InputFormat::Csv => write!(f, "CSV"),
            InputFormat::Tsv => write!(f, "TSV"),
            InputFormat::Rtf => write!(f, "RTF"),
            InputFormat::Odt => write!(f, "ODT"),
            InputFormat::Json => write!(f, "JSON"),
            InputFormat::Zip => write!(f, "ZIP"),
            InputFormat::Image => write!(f, "Image/OCR"),
            InputFormat::Markdown => write!(f, "Markdown"),
            InputFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

impl InputFormat {
    /// Get status string for the format
    pub fn status(&self) -> &'static str {
        match self {
            InputFormat::Pdf | InputFormat::Docx | InputFormat::Xlsx | InputFormat::Pptx |
            InputFormat::Html | InputFormat::Xml | InputFormat::Txt |
            InputFormat::Csv | InputFormat::Tsv | InputFormat::Json |
            InputFormat::Zip | InputFormat::Image | InputFormat::Markdown => "Production",
            InputFormat::Rtf | InputFormat::Odt => "Beta",
            InputFormat::Unknown => "N/A",
        }
    }

    /// Get the output options for this format
    pub fn output_options(&self) -> &'static str {
        match self {
            InputFormat::Pdf => "Image per page, Markdown (per page/full), JSON",
            InputFormat::Docx => "Image per page, Markdown (per page/full), JSON",
            InputFormat::Xlsx => "Markdown tables, CSV, JSON",
            InputFormat::Pptx => "Image per slide, Markdown per slide",
            InputFormat::Html => "Markdown, JSON",
            InputFormat::Xml => "Markdown, JSON",
            InputFormat::Txt => "Markdown, JSON",
            InputFormat::Csv | InputFormat::Tsv => "Markdown tables, JSON",
            InputFormat::Rtf => "Markdown, JSON",
            InputFormat::Odt => "Markdown, JSON",
            InputFormat::Json => "Pretty-printed in fenced code blocks",
            InputFormat::Zip => "File listing, statistics, Markdown index, JSON",
            InputFormat::Image => "Markdown (OCR), JSON",
            InputFormat::Markdown => "Normalized Markdown",
            InputFormat::Unknown => "",
        }
    }
}

/// Output format options
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    Markdown { split_pages: bool, optimize_for_llm: bool },
    Json { structured: bool, include_metadata: bool },
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Markdown {
            split_pages: false,
            optimize_for_llm: true,
        }
    }
}

/// Detect the input format from a file extension
pub fn detect_format(path: &Path) -> InputFormat {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => InputFormat::Pdf,
        "docx" | "doc" => InputFormat::Docx,
        "xlsx" | "xls" => InputFormat::Xlsx,
        "pptx" | "ppt" => InputFormat::Pptx,
        "html" | "htm" => InputFormat::Html,
        "xml" => InputFormat::Xml,
        "txt" | "text" | "log" | "md" | "markdown" => {
            if ext == "md" || ext == "markdown" {
                InputFormat::Markdown
            } else {
                InputFormat::Txt
            }
        }
        "csv" => InputFormat::Csv,
        "tsv" => InputFormat::Tsv,
        "rtf" => InputFormat::Rtf,
        "odt" => InputFormat::Odt,
        "json" | "jsonl" => InputFormat::Json,
        "zip" => InputFormat::Zip,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" => InputFormat::Image,
        _ => InputFormat::Unknown,
    }
}

/// Check if a file format is supported for conversion
pub fn is_supported(path: &Path) -> bool {
    !matches!(detect_format(path), InputFormat::Unknown)
}

/// Collect all supported files from a list of paths (including directories)
pub fn collect_files(paths: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() && is_supported(path) {
            files.push(path.clone());
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.is_file() && is_supported(p) {
                    files.push(p.to_path_buf());
                }
            }
        }
    }
    files
}

/// Format file size in human-readable form
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get the application data directory for storing tessdata and other resources
pub fn app_data_dir() -> std::path::PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("markitdown-rst");

    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Get the tessdata directory
pub fn tessdata_dir() -> std::path::PathBuf {
    let dir = app_data_dir().join("tessdata");
    let _ = std::fs::create_dir_all(&dir);
    dir
}
