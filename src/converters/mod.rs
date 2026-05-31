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
use std::path::Path;

/// Metadata about a converted document
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct OutputChunk {
    pub page_number: usize,
    pub data: String,
    pub label: String,
}

/// Result of a document conversion
#[derive(Debug, Clone)]
#[must_use = "ConversionResult must be saved or written"]
pub struct ConversionResult {
    pub content: Vec<OutputChunk>,
    pub metadata: DocumentMetadata,
    pub output_format: OutputFormat,
}

impl ConversionResult {
    #[inline]
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

    /// Create result WITHOUT recounting words (caller already counted)
    #[inline]
    pub fn from_markdown_no_recount(markdown: String, metadata: DocumentMetadata) -> Self {
        Self {
            content: vec![OutputChunk {
                page_number: 1,
                data: markdown,
                label: "Full document".to_string(),
            }],
            metadata,
            output_format: OutputFormat::default(),
        }
    }

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

    /// Get the full markdown text (all pages combined) with pre-allocated capacity
    pub fn full_markdown(&self) -> String {
        const SEP: &str = "\n\n---\n\n";
        let total: usize = self.content.iter().map(|c| c.data.len()).sum::<usize>()
            + SEP.len() * self.content.len().saturating_sub(1);
        let mut out = String::with_capacity(total);
        for (i, chunk) in self.content.iter().enumerate() {
            if i > 0 { out.push_str(SEP); }
            out.push_str(&chunk.data);
        }
        out
    }

    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        match self.output_format {
            OutputFormat::Html { .. } => {
                let html = crate::export::md_to_html::markdown_to_html(
                    &self.full_markdown(),
                    &self.output_format,
                )?;
                tokio::fs::write(path, html).await?;
            }
            OutputFormat::Docx => {
                let title = self.metadata.title.as_deref();
                let docx_bytes = crate::export::md_to_docx::markdown_to_docx(
                    &self.full_markdown(),
                    title,
                )?;
                tokio::fs::write(path, docx_bytes).await?;
            }
            OutputFormat::Markdown { .. } | OutputFormat::Json { .. } => {
                let content = self.full_markdown();
                tokio::fs::write(path, content).await?;
            }
        }
        Ok(())
    }

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
    fn format(&self) -> InputFormat;
    async fn convert(&self, path: &Path, output_format: &OutputFormat) -> Result<ConversionResult>;
    fn can_convert(&self, path: &Path) -> bool {
        crate::utils::detect_format(path) == self.format()
    }
}

/// Enum dispatch — avoids heap allocation for converters (all are ZSTs)
pub enum Converter {
    Pdf(pdf::PdfConverter),
    Docx(docx::DocxConverter),
    Xlsx(xlsx::XlsxConverter),
    Pptx(pptx::PptxConverter),
    Html(html::HtmlConverter),
    Xml(html::XmlConverter),
    Txt(txt::TxtConverter),
    Csv(csv::CsvConverter),
    Json(json_conv::JsonConverter),
    Zip(zip_conv::ZipConverter),
    #[cfg(feature = "ocr")]
    Image(image_ocr::ImageOcrConverter),
}

impl Converter {
    pub async fn convert(&self, path: &Path, fmt: &OutputFormat) -> Result<ConversionResult> {
        match self {
            Converter::Pdf(c) => c.convert(path, fmt).await,
            Converter::Docx(c) => c.convert(path, fmt).await,
            Converter::Xlsx(c) => c.convert(path, fmt).await,
            Converter::Pptx(c) => c.convert(path, fmt).await,
            Converter::Html(c) => c.convert(path, fmt).await,
            Converter::Xml(c) => c.convert(path, fmt).await,
            Converter::Txt(c) => c.convert(path, fmt).await,
            Converter::Csv(c) => c.convert(path, fmt).await,
            Converter::Json(c) => c.convert(path, fmt).await,
            Converter::Zip(c) => c.convert(path, fmt).await,
            #[cfg(feature = "ocr")]
            Converter::Image(c) => c.convert(path, fmt).await,
        }
    }
}

/// Factory function: get the appropriate converter for a file
pub fn get_converter(path: &Path) -> Option<Converter> {
    use InputFormat::*;
    Some(match crate::utils::detect_format(path) {
        Pdf => Converter::Pdf(pdf::PdfConverter::new()),
        Docx => Converter::Docx(docx::DocxConverter::new()),
        Xlsx => Converter::Xlsx(xlsx::XlsxConverter::new()),
        Pptx => Converter::Pptx(pptx::PptxConverter::new()),
        Html => Converter::Html(html::HtmlConverter::new()),
        Xml => Converter::Xml(html::XmlConverter::new()),
        Txt | Markdown | Rtf | Odt => Converter::Txt(txt::TxtConverter::new()),
        Csv | Tsv => Converter::Csv(csv::CsvConverter::new()),
        Json => Converter::Json(json_conv::JsonConverter::new()),
        Zip => Converter::Zip(zip_conv::ZipConverter::new()),
        #[cfg(feature = "ocr")]
        Image => Converter::Image(image_ocr::ImageOcrConverter::with_default_languages()),
        #[cfg(not(feature = "ocr"))]
        Image => return None,
        Unknown => return None,
    })
}

#[cfg(feature = "ocr")]
pub fn get_converter_with_ocr(path: &Path, ocr_languages: &[crate::ocr::OcrLanguage]) -> Option<Converter> {
    use InputFormat::*;
    Some(match crate::utils::detect_format(path) {
        Pdf => Converter::Pdf(pdf::PdfConverter::new()),
        Docx => Converter::Docx(docx::DocxConverter::new()),
        Xlsx => Converter::Xlsx(xlsx::XlsxConverter::new()),
        Pptx => Converter::Pptx(pptx::PptxConverter::new()),
        Html => Converter::Html(html::HtmlConverter::new()),
        Xml => Converter::Xml(html::XmlConverter::new()),
        Txt | Markdown | Rtf | Odt => Converter::Txt(txt::TxtConverter::new()),
        Csv | Tsv => Converter::Csv(csv::CsvConverter::new()),
        Json => Converter::Json(json_conv::JsonConverter::new()),
        Zip => Converter::Zip(zip_conv::ZipConverter::new()),
        Image => Converter::Image(image_ocr::ImageOcrConverter::new(ocr_languages.to_vec())),
        Unknown => return None,
    })
}
