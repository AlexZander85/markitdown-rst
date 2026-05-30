//! DOCX to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// DOCX converter
pub struct DocxConverter;

impl DocxConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for DocxConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Docx
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_docx_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_docx_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let data = std::fs::read(path)?;
    let doc = docx_rs::read_docx(&data)?;

    // docx_rs returns a Document; we extract paragraphs
    let mut markdown = String::new();
    let mut title: Option<String> = None;

    // Read the document from the ZIP structure and parse XML
    // Using a more direct approach: parse the ZIP and extract document.xml
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&data))?;

    let mut document_xml = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == "word/document.xml" {
            use std::io::Read;
            file.read_to_string(&mut document_xml)?;
            break;
        }
    }

    if !document_xml.is_empty() {
        markdown = parse_docx_xml(&document_xml);
    }

    // Try to extract title from first heading
    if title.is_none() {
        title = markdown
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# ").to_string());
    }

    let word_count = markdown.split_whitespace().count();

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count: 1,
        word_count,
        source_format: InputFormat::Docx,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}

fn parse_docx_xml(xml: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut markdown = String::new();
    let mut in_paragraph = false;
    let mut in_run = false;
    let mut in_text = false;
    let mut current_text = String::new();
    let mut is_bold = false;
    let mut is_italic = false;
    let mut heading_level: Option<u32> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                let name_str = String::from_utf8_lossy(local_name.as_ref());

                match name_str.as_ref() {
                    "p" => {
                        in_paragraph = true;
                        current_text.clear();
                    }
                    "r" => {
                        in_run = true;
                    }
                    "t" => {
                        in_text = true;
                    }
                    "b" | "bCs" => {
                        is_bold = true;
                    }
                    "i" | "iCs" => {
                        is_italic = true;
                    }
                    "pStyle" => {
                        // Check for heading style
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"val" {
                                let val = String::from_utf8_lossy(&attr.value);
                                if val.starts_with("Heading") || val.starts_with("heading") {
                                    heading_level = val
                                        .chars()
                                        .filter(|c| c.is_ascii_digit())
                                        .collect::<String>()
                                        .parse()
                                        .ok();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name_str = String::from_utf8_lossy(local_name.as_ref());
                match name_str.as_ref() {
                    "b" | "bCs" => {
                        is_bold = true;
                    }
                    "i" | "iCs" => {
                        is_italic = true;
                    }
                    "br" => {
                        current_text.push('\n');
                    }
                    "tab" => {
                        current_text.push('\t');
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default();
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name_str = String::from_utf8_lossy(local_name.as_ref());

                match name_str.as_ref() {
                    "t" => {
                        in_text = false;
                    }
                    "r" => {
                        in_run = false;
                        is_bold = false;
                        is_italic = false;
                    }
                    "p" => {
                        in_paragraph = false;

                        let text = current_text.trim();
                        if !text.is_empty() {
                            if let Some(level) = heading_level.take() {
                                let hashes = "#".repeat(level as usize);
                                markdown.push_str(&format!("{} {}\n\n", hashes, text));
                            } else if is_bold {
                                markdown.push_str(&format!("**{}**\n\n", text));
                            } else if is_italic {
                                markdown.push_str(&format!("*{}*\n\n", text));
                            } else {
                                markdown.push_str(&format!("{}\n\n", text));
                            }
                        }
                        heading_level = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("XML parsing error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    markdown
}
