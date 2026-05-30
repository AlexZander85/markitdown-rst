//! Document converter trait and implementations

pub mod pdf;
pub mod docx;
pub mod xlsx;
pub mod pptx;
pub mod html;
pub mod txt;
pub mod csv;
pub mod json_conv;
pub mod zip_conv;

#[cfg(feature = "ocr")]
pub mod image_ocr;

use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata about a converted document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: usize,
    pub word_count: usize,
    pub source_format: InputFormat,
    pub source_path: String,
    pub file_size_bytes: u64,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            page_count: 0,
            word_count: 0,
            source_format: InputFormat::Unknown,
            source_path: String::new(),
            file_size_bytes: 0,
        }
    }
}

/// A single output chunk (page, slide, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputChunk {
    pub page_number: usize,
    pub data: String,
    pub label: String,
}

/// Result of a document conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub content: Vec<OutputChunk>,
    pub metadata: DocumentMetadata,
    pub output_format: OutputFormat,
}

impl ConversionResult {
    /// Create a new conversion result from a single markdown string
    pub fn from_markdown(markdown: String, metadata: DocumentMetadata) -> Self {
        let word_count = markdown.split_whitespace().count();
        Self {
            content: vec![OutputChunk {
                page_number: 1,
                data: markdown,
                label: "Full document".to_string(),
            }],
            metadata: DocumentMetadata {
                word_count,
                ..metadata
            },
            output_format: OutputFormat::default(),
        }
    }

    /// Create a conversion result from multiple pages
    pub fn from_pages(pages: Vec<(String, String)>, metadata: DocumentMetadata) -> Self {
        let total_words: usize = pages.iter().map(|(p, _)| p.split_whitespace().count()).sum();
        let page_count = pages.len();
        let content: Vec<OutputChunk> = pages
            .into_iter()
            .enumerate()
            .map(|(i, (data, label))| OutputChunk {
                page_number: i + 1,
                data,
                label,
            })
            .collect();

        Self {
            content,
            metadata: DocumentMetadata {
                word_count: total_words,
                page_count,
                ..metadata
            },
            output_format: OutputFormat::default(),
        }
    }

    /// Get the full markdown text (all pages combined)
    pub fn full_markdown(&self) -> String {
        self.content
            .iter()
            .map(|c| c.data.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Save all content to a single file
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = self.full_markdown();
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Save each chunk as a separate file in the given directory
    pub async fn save_to_directory(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        tokio::fs::create_dir_all(dir).await?;
        let mut saved = Vec::new();
        for chunk in &self.content {
            let filename = format!("page_{}.md", chunk.page_number);
            let path = dir.join(&filename);
            tokio::fs::write(&path, &chunk.data).await?;
            saved.push(path);
        }
        Ok(saved)
    }
}

/// Trait for document converters
#[async_trait::async_trait]
pub trait DocumentConverter: Send + Sync {
    /// Get the input format this converter handles
    fn format(&self) -> InputFormat;

    /// Convert a document at the given path
    async fn convert(
        &self,
        path: &Path,
        output_format: &OutputFormat,
    ) -> Result<ConversionResult>;

    /// Check if this converter can handle the given file
    fn can_convert(&self, path: &Path) -> bool {
        crate::utils::detect_format(path) == self.format()
    }
}

/// Factory function: get the appropriate converter for a file
///
/// When the `ocr` feature is enabled, image files will be handled by OCR.
/// When the `ocr` feature is disabled, image files return None (unsupported).
pub fn get_converter(path: &Path) -> Option<Box<dyn DocumentConverter>> {
    match crate::utils::detect_format(path) {
        InputFormat::Pdf => Some(Box::new(pdf::PdfConverter::new())),
        InputFormat::Docx => Some(Box::new(docx::DocxConverter::new())),
        InputFormat::Xlsx => Some(Box::new(xlsx::XlsxConverter::new())),
        InputFormat::Pptx => Some(Box::new(pptx::PptxConverter::new())),
        InputFormat::Html => Some(Box::new(html::HtmlConverter::new())),
        InputFormat::Xml => Some(Box::new(html::XmlConverter::new())),
        InputFormat::Txt | InputFormat::Markdown => Some(Box::new(txt::TxtConverter::new())),
        InputFormat::Csv | InputFormat::Tsv => Some(Box::new(csv::CsvConverter::new())),
        InputFormat::Rtf | InputFormat::Odt => Some(Box::new(txt::TxtConverter::new())),
        InputFormat::Json => Some(Box::new(json_conv::JsonConverter::new())),
        InputFormat::Zip => Some(Box::new(zip_conv::ZipConverter::new())),
        #[cfg(feature = "ocr")]
        InputFormat::Image => Some(Box::new(image_ocr::ImageOcrConverter::with_default_languages())),
        #[cfg(not(feature = "ocr"))]
        InputFormat::Image => None,
        InputFormat::Unknown => None,
    }
}

/// Factory function: get the appropriate converter for a file with specified OCR languages.
/// Only available when the `ocr` feature is enabled.
#[cfg(feature = "ocr")]
pub fn get_converter_with_ocr(path: &Path, ocr_languages: &[crate::ocr::OcrLanguage]) -> Option<Box<dyn DocumentConverter>> {
    match crate::utils::detect_format(path) {
        InputFormat::Pdf => Some(Box::new(pdf::PdfConverter::new())),
        InputFormat::Docx => Some(Box::new(docx::DocxConverter::new())),
        InputFormat::Xlsx => Some(Box::new(xlsx::XlsxConverter::new())),
        InputFormat::Pptx => Some(Box::new(pptx::PptxConverter::new())),
        InputFormat::Html => Some(Box::new(html::HtmlConverter::new())),
        InputFormat::Xml => Some(Box::new(html::XmlConverter::new())),
        InputFormat::Txt | InputFormat::Markdown => Some(Box::new(txt::TxtConverter::new())),
        InputFormat::Csv | InputFormat::Tsv => Some(Box::new(csv::CsvConverter::new())),
        InputFormat::Rtf | InputFormat::Odt => Some(Box::new(txt::TxtConverter::new())),
        InputFormat::Json => Some(Box::new(json_conv::JsonConverter::new())),
        InputFormat::Zip => Some(Box::new(zip_conv::ZipConverter::new())),
        InputFormat::Image => Some(Box::new(image_ocr::ImageOcrConverter::new(ocr_languages.to_vec()))),
        InputFormat::Unknown => None,
    }
}
