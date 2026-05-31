//! PDF to Markdown converter — three-tier extraction with OCR fallback
//!
//! Extraction strategy:
//! 1. `pdf-extract` — best quality text extraction (but may panic on some PDFs)
//! 2. `lopdf` — fallback page-by-page extraction
//! 3. `pdfium-render` → OCR — for scanned/image-based PDFs (if feature enabled)
//! 4. Graceful error with diagnostics if all methods fail

use super::{ConversionResult, Converter, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::{bail, Context, Result};
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
        let file_size = tokio::fs::metadata(&path)
            .await
            .with_context(|| format!("Cannot access PDF file: {}", path.display()))?
            .len();
        let path_display = path.display().to_string();
        let result = tokio::task::spawn_blocking(move || {
            extract_pdf_to_markdown(&path, file_size)
        }).await
            .with_context(|| format!("PDF conversion task crashed: {}", path_display))??;
        Ok(result)
    }
}

fn extract_pdf_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    // Load lopdf ONCE — needed for page_count and as fallback
    let doc = lopdf::Document::load(path).ok();
    let page_count = doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(1);

    // Try pdf-extract first (better quality) — catch panics because
    // pdf-extract can panic on malformed or encrypted PDFs
    let text = match extract_text_via_pdf_extract(path) {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            tracing::debug!(
                "pdf-extract failed for {}, falling back to lopdf",
                path.display()
            );
            match doc {
                Some(d) => extract_text_from_doc(&d),
                None => {
                    // Final attempt: pdfium-render + OCR (if available)
                    #[cfg(feature = "pdf-to-image")]
                    {
                        tracing::info!("Attempting PDF → image → OCR for {}", path.display());
                        match pdf_to_ocr(path) {
                            Ok(md) => return Ok(ConversionResult::from_markdown_no_recount(
                                md,
                                DocumentMetadata {
                                    title: None,
                                    author: None,
                                    page_count,
                                    word_count: 0,
                                    source_format: InputFormat::Pdf,
                                    source_path: path.display().to_string(),
                                    file_size_bytes: file_size,
                                },
                            )),
                            Err(e) => {
                                tracing::warn!("PDF OCR fallback failed: {}", e);
                                return Err(anyhow::anyhow!(
                                    "Failed to extract text from PDF: {}. \
                                     The file may be encrypted, corrupted, or contain only images. \
                                     Try: 1) Remove PDF password, 2) Install Tesseract for better OCR",
                                    path.display()
                                ));
                            }
                        }
                    }

                    #[cfg(not(feature = "pdf-to-image"))]
                    {
                        return Err(anyhow::anyhow!(
                            "Failed to extract text from PDF: {}. \
                             The file may be encrypted, corrupted, or contain only images. \
                             Try: 1) Remove PDF password, 2) Use Full edition with OCR for scanned documents",
                            path.display()
                        ));
                    }
                }
            }
        }
    };

    // If both extractors returned empty text, try OCR fallback
    if text.trim().is_empty() {
        #[cfg(feature = "pdf-to-image")]
        {
            tracing::info!("PDF text is empty, attempting OCR for {}", path.display());
            match pdf_to_ocr(path) {
                Ok(md) if !md.trim().is_empty() => {
                    return Ok(ConversionResult::from_markdown_no_recount(
                        md,
                        DocumentMetadata {
                            title: None,
                            author: None,
                            page_count,
                            word_count: 0,
                            source_format: InputFormat::Pdf,
                            source_path: path.display().to_string(),
                            file_size_bytes: file_size,
                        },
                    ));
                }
                Ok(_) => {} // OCR also returned empty
                Err(e) => tracing::warn!("PDF OCR fallback failed: {}", e),
            }
        }

        return Err(anyhow::anyhow!(
            "PDF contains no extractable text: {}. \
             This typically means the PDF is a scanned image. \
             Use the Full edition with OCR support to recognize text from image-based PDFs.",
            path.display()
        ));
    }

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

/// Render PDF pages to images and run OCR on them.
#[cfg(feature = "pdf-to-image")]
fn pdf_to_ocr(path: &Path) -> Result<String> {
    use pdfium_render::prelude::*;

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| anyhow::anyhow!("Failed to bind PDFium library: {}", e))?
    );

    let document = pdfium.load_pdf_from_file(path, None)
        .with_context(|| format!("PDFium failed to load PDF: {}", path.display()))?;

    let mut full_text = String::new();
    let pages = document.pages();

    for (page_idx, page) in pages.iter().enumerate() {
        // Render page to bitmap (300 DPI for good OCR quality)
        let bitmap = page.render_with_config(&PdfRenderConfig::new()
            .set_target_width(2480)  // ~300 DPI for letter-size
            .set_target_height(3508)
        )?;

        // Convert to image::DynamicImage
        let (width, height) = (bitmap.width() as u32, bitmap.height() as u32);
        let rgba_data = bitmap.as_rgba_bytes();
        let img_buffer = image::ImageBuffer::from_raw(width, height, rgba_data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from PDF page"))?;
        let dynamic_image = image::DynamicImage::ImageRgba8(img_buffer);

        // Save to temp file for OCR
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("mdrust_pdf_page_{}.png", page_idx));
        dynamic_image.save(&temp_path)
            .with_context(|| format!("Failed to save PDF page {} as image", page_idx))?;

        // Run OCR on the image
        #[cfg(feature = "ocr")]
        {
            let ocr_result = crate::ocr::ocr_image_to_markdown(
                &temp_path,
                &[crate::ocr::OcrLanguage::Eng],
            );

            match ocr_result {
                Ok(text) if !text.trim().is_empty() => {
                    if !full_text.is_empty() {
                        full_text.push_str("\n\n---\n\n");
                    }
                    full_text.push_str(&format!("## Page {}\n\n{}", page_idx + 1, text));
                }
                Ok(_) => {
                    tracing::warn!("OCR returned empty text for page {}", page_idx + 1);
                }
                Err(e) => {
                    tracing::warn!("OCR failed for page {}: {}", page_idx + 1, e);
                }
            }
        }

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
    }

    if full_text.trim().is_empty() {
        bail!("OCR produced no text from any page of the PDF");
    }

    Ok(full_text)
}

/// Try to extract text using pdf-extract, catching panics.
///
/// `pdf-extract` can panic on certain PDF structures (e.g., encrypted files,
/// malformed streams, or unsupported compression). We use `catch_unwind`
/// to prevent the panic from killing the entire conversion task.
fn extract_text_via_pdf_extract(path: &Path) -> Option<String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pdf_extract::extract_text(path)
    }))
    .ok()
    .and_then(|result| result.ok())
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
