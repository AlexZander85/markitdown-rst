//! Image OCR converter using Tesseract
//!
//! Converts images (JPG, PNG, TIFF, BMP, GIF, WEBP) to Markdown via OCR.
//! Requires the Tesseract CLI to be installed on the system.

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::ocr::{self, OcrLanguage};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Image OCR converter
pub struct ImageOcrConverter {
    languages: Vec<OcrLanguage>,
}

impl ImageOcrConverter {
    pub fn new(languages: Vec<OcrLanguage>) -> Self {
        let langs = if languages.is_empty() {
            vec![OcrLanguage::Eng]
        } else {
            languages
        };
        Self { languages: langs }
    }

    pub fn with_default_languages() -> Self {
        Self::new(vec![OcrLanguage::Eng, OcrLanguage::Rus, OcrLanguage::ChiSim])
    }
}

#[async_trait::async_trait]
impl DocumentConverter for ImageOcrConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Image
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();
        let languages = self.languages.clone();

        let result = tokio::task::spawn_blocking(move || {
            extract_image_ocr_to_markdown(&path, file_size, &languages)
        })
        .await??;

        Ok(result)
    }
}

fn extract_image_ocr_to_markdown(
    path: &Path,
    file_size: u64,
    languages: &[OcrLanguage],
) -> Result<ConversionResult> {
    // Run OCR using the ocr module
    let ocr_text = ocr::ocr_image_to_markdown(path, languages)?;

    // Build markdown output
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image");

    let mut markdown = String::new();
    markdown.push_str(&format!("# OCR Result: {}\n\n", filename));

    // Add language info
    let lang_names: Vec<String> = languages.iter().map(|l| l.to_string()).collect();
    markdown.push_str(&format!("*OCR Languages: {}*\n\n", lang_names.join(", ")));

    if ocr_text.trim().is_empty() {
        markdown.push_str("*No text detected in image*\n");
    } else {
        markdown.push_str(&ocr_text);
    }

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title: Some(filename.to_string()),
        author: None,
        page_count: 1,
        word_count,
        source_format: InputFormat::Image,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}
