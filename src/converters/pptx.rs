//! PPTX to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// PPTX converter
pub struct PptxConverter;

impl PptxConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for PptxConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Pptx
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_pptx_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_pptx_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let data = std::fs::read(path)?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&data))?;

    let mut slides: Vec<(String, String)> = Vec::new();
    let mut slide_number = 0;

    // Collect slide file names and sort them
    let mut slide_files: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            slide_files.push(name);
        }
    }

    // Sort by slide number
    slide_files.sort_by(|a, b| {
        let num_a: u32 = a
            .trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse()
            .unwrap_or(0);
        let num_b: u32 = b
            .trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse()
            .unwrap_or(0);
        num_a.cmp(&num_b)
    });

    for slide_file in &slide_files {
        slide_number += 1;
        let mut xml_content = String::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == slide_file {
                use std::io::Read;
                file.read_to_string(&mut xml_content)?;
                break;
            }
        }

        if !xml_content.is_empty() {
            let slide_markdown = parse_pptx_slide_xml(&xml_content, slide_number);
            slides.push((
                slide_markdown,
                format!("Slide {}", slide_number),
            ));
        }
    }

    let total_words: usize = slides.iter().map(|(s, _)| s.split_whitespace().count()).sum();

    let metadata = DocumentMetadata {
        title: slides
            .first()
            .and_then(|(s, _)| s.lines().find(|l| l.starts_with("# ")))
            .map(|l| l.trim_start_matches("# ").to_string()),
        author: None,
        page_count: slides.len(),
        word_count: total_words,
        source_format: InputFormat::Pptx,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_pages(slides, metadata))
}

fn parse_pptx_slide_xml(xml: &str, slide_number: usize) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut markdown = format!("# Slide {}\n\n", slide_number);
    let mut in_text = false;
    let mut current_text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                let name_str = String::from_utf8_lossy(local_name.as_ref());
                if name_str == "t" {
                    in_text = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default();
                    if !text.trim().is_empty() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name_str = String::from_utf8_lossy(local_name.as_ref());
                match name_str.as_ref() {
                    "t" => {
                        in_text = false;
                    }
                    "p" => {
                        let text = current_text.trim();
                        if !text.is_empty() {
                            markdown.push_str(text);
                            markdown.push('\n');
                        }
                        current_text.clear();
                    }
                    "sp" => {
                        if !current_text.trim().is_empty() {
                            let text = current_text.trim();
                            markdown.push_str(text);
                            markdown.push('\n');
                        }
                        current_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("PPTX XML parsing error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    // If nothing was extracted, add a placeholder
    if markdown.trim() == format!("# Slide {}", slide_number) {
        markdown.push_str("*No text content on this slide*\n");
    }

    markdown
}
