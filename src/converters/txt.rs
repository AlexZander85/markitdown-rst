//! Plain text / RTF / ODT / Markdown to Markdown converter

use super::{ConversionResult, DocumentConverter, DocumentMetadata};
use crate::utils::{InputFormat, OutputFormat};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Plain text / Markdown converter
pub struct TxtConverter;

impl TxtConverter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl DocumentConverter for TxtConverter {
    fn format(&self) -> InputFormat {
        InputFormat::Txt
    }

    async fn convert(
        &self,
        path: &Path,
        _output_format: &OutputFormat,
    ) -> Result<ConversionResult> {
        let path = path.to_path_buf();
        let file_size = tokio::fs::metadata(&path).await?.len();

        let result = tokio::task::spawn_blocking(move || {
            extract_txt_to_markdown(&path, file_size)
        })
        .await??;

        Ok(result)
    }
}

fn extract_txt_to_markdown(path: &Path, file_size: u64) -> Result<ConversionResult> {
    let format = crate::utils::detect_format(path);

    let markdown = match format {
        InputFormat::Rtf => convert_rtf(path)?,
        InputFormat::Odt => convert_odt(path)?,
        _ => {
            // Plain text or Markdown - just read as-is
            let content = std::fs::read_to_string(path)?;
            if format == InputFormat::Markdown {
                content
            } else {
                // Convert plain text to markdown: detect potential headers
                txt_to_markdown(&content)
            }
        }
    };

    let word_count = markdown.split_whitespace().count();
    let title = markdown
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string());

    let metadata = DocumentMetadata {
        title,
        author: None,
        page_count: 1,
        word_count,
        source_format: format,
        source_path: path.display().to_string(),
        file_size_bytes: file_size,
    };

    Ok(ConversionResult::from_markdown(markdown, metadata))
}

/// Convert plain text to markdown with heuristic header detection
fn txt_to_markdown(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut markdown = String::new();
    let mut prev_empty = true;

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if !prev_empty {
                markdown.push('\n');
            }
            prev_empty = true;
            continue;
        }

        // Heuristic: lines that are all uppercase or short and preceded by empty line
        if trimmed.len() < 60
            && prev_empty
            && (trimmed.chars().filter(|c| c.is_uppercase()).count() as f32
                / trimmed.chars().filter(|c| c.is_alphabetic()).count().max(1) as f32)
                > 0.6
        {
            markdown.push_str(&format!("## {}\n", trimmed));
        } else if trimmed.len() < 40
            && prev_empty
            && !trimmed.ends_with('.')
            && !trimmed.ends_with(',')
        {
            markdown.push_str(&format!("### {}\n", trimmed));
        } else {
            markdown.push_str(&format!("{}\n", line));
        }

        prev_empty = false;
    }

    markdown
}

/// Basic RTF to text converter (simplified)
fn convert_rtf(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    // Very basic RTF stripping - remove RTF control words
    let re = regex::Regex::new(r"\\[a-z]+\d*\s?|\\[{}]|[{}]|\\\n")?;
    let text = re.replace_all(&content, "");
    // Clean up remaining escapes
    let text = text.replace("\\par ", "\n").replace("\\par", "\n");
    Ok(txt_to_markdown(&text))
}

/// Basic ODT (OpenDocument Text) converter
fn convert_odt(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&data))?;

    let mut content_xml = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name() == "content.xml" {
            use std::io::Read;
            file.read_to_string(&mut content_xml)?;
            break;
        }
    }

    if content_xml.is_empty() {
        return Ok("*Could not extract ODT content*\n".to_string());
    }

    // Parse the XML and extract text
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(&content_xml);
    reader.config_mut().trim_text(true);

    let mut markdown = String::new();
    let mut in_text = false;
    let mut current_text = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let name = std::str::from_utf8(local.as_ref()).unwrap_or("");
                match name {
                    "p" | "h" => {
                        current_text.clear();
                    }
                    "span" | "t" => {
                        in_text = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    current_text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let name = std::str::from_utf8(local.as_ref()).unwrap_or("");
                match name {
                    "p" => {
                        let text = current_text.trim();
                        if !text.is_empty() {
                            markdown.push_str(text);
                            markdown.push('\n');
                        }
                        current_text.clear();
                    }
                    "h" => {
                        let text = current_text.trim();
                        if !text.is_empty() {
                            markdown.push_str(&format!("## {}\n", text));
                        }
                        current_text.clear();
                    }
                    "span" | "t" => {
                        in_text = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(markdown)
}
