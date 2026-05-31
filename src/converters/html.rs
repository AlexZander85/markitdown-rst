//! HTML and XML to Markdown converters — using htmd (lightweight)

use super::{ConversionResult, Converter, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

pub struct HtmlConverter;

impl HtmlConverter {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl DocumentConverter for HtmlConverter {
    fn format(&self) -> InputFormat { InputFormat::Html }

    async fn convert(&self, path: &Path, _output_format: &OutputFormat) -> Result<ConversionResult> {
        let file_size = tokio::fs::metadata(path).await?.len();
        let html = tokio::fs::read_to_string(path).await?;
        let path_buf = path.to_path_buf();

        let result = tokio::task::spawn_blocking(move || -> Result<ConversionResult> {
            let converter = htmd::HtmlToMarkdown::builder()
                .skip_tags(vec!["script", "style", "noscript"])
                .build();
            let markdown = converter.convert(&html)?;

            let word_count = crate::utils::count_words(&markdown);
            let title = markdown.lines()
                .find(|l| l.starts_with("# ") || l.starts_with("## "))
                .map(|l| l.trim_start_matches('#').trim().to_string());

            let metadata = DocumentMetadata {
                title,
                author: None,
                page_count: 1,
                word_count,
                source_format: InputFormat::Html,
                source_path: path_buf.display().to_string(),
                file_size_bytes: file_size,
            };

            Ok(ConversionResult::from_markdown_no_recount(markdown, metadata))
        }).await??;

        Ok(result)
    }
}

/// XML to Markdown converter
pub struct XmlConverter;

impl XmlConverter {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl DocumentConverter for XmlConverter {
    fn format(&self) -> InputFormat { InputFormat::Xml }

    async fn convert(&self, path: &Path, _output_format: &OutputFormat) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();
        let result = tokio::task::spawn_blocking(move || {
            extract_xml_to_markdown(&path, file_size)
        }).await??;
        Ok(result)
    }
}

fn extract_xml_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let xml_content = std::fs::read_to_string(path)?;
    let markdown = xml_to_markdown(&xml_content);
    let word_count = crate::utils::count_words(&markdown);
    let metadata = DocumentMetadata {
        title: None,
        author: None,
        page_count: 1,
        word_count,
        source_format: InputFormat::Xml,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };
    Ok(ConversionResult::from_markdown_no_recount(markdown, metadata))
}

fn xml_to_markdown(xml: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut markdown = String::new();
    let mut depth: u32 = 0;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if depth == 0 { markdown.push_str(&format!("# {}\n\n", name)); }
                else if depth == 1 { markdown.push_str(&format!("## {}\n\n", name)); }
                else { markdown.push_str(&format!("**{}:** ", name)); }
                depth += 1;
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default();
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    markdown.push_str(trimmed);
                    markdown.push('\n');
                }
            }
            Ok(Event::End(_)) => { depth = depth.saturating_sub(1); }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    markdown
}
