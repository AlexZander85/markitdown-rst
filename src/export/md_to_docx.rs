//! Markdown → DOCX converter
//!
//! Parses Markdown using `pulldown-cmark` and builds a DOCX file using `docx-rs`.
//! Supports: headings, paragraphs, bold, italic, strikethrough, code (inline + blocks),
//! lists (ordered + unordered), tables, links, horizontal rules, blockquotes.

use anyhow::{bail, Result};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Convert Markdown text to a DOCX byte vector.
pub fn markdown_to_docx(markdown: &str, title: Option<&str>) -> Result<Vec<u8>> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, opts);

    // Collect all events first, then build DOCX
    let events: Vec<Event> = parser.collect();

    let doc = DocxBuilder::new(title).build(&events)?;

    let mut buf = std::io::Cursor::new(Vec::new());
    doc.write(&mut buf)
        .map_err(|e| anyhow::anyhow!("Failed to write DOCX: {e}"))?;

    Ok(buf.into_inner())
}

// ── DOCX Builder ──────────────────────────────────────────────────────────

struct DocxBuilder {
    /// Current paragraph text runs being accumulated
    current_runs: Vec<docx_rs::Run>,
    /// Whether we're inside a bold context
    bold: bool,
    /// Whether we're inside an italic context
    italic: bool,
    /// Whether we're inside a strikethrough context
    strikethrough: bool,
    /// Whether we're inside a code context
    code: bool,
    /// Accumulated paragraphs
    paragraphs: Vec<docx_rs::Paragraph>,
    /// Current list level (0 = top)
    list_level: u8,
    /// Whether current list is ordered
    list_ordered: bool,
}

impl DocxBuilder {
    fn new(title: Option<&str>) -> Self {
        let mut paragraphs = Vec::new();

        // Add title as first heading if provided
        if let Some(t) = title {
            paragraphs.push(
                docx_rs::Paragraph::new()
                    .add_run(docx_rs::Run::new().add_text(t).size(56).bold())
                    .spacing_after(200),
            );
        }

        Self {
            current_runs: Vec::new(),
            bold: false,
            italic: false,
            strikethrough: false,
            code: false,
            paragraphs,
            list_level: 0,
            list_ordered: false,
        }
    }

    fn build(mut self, events: &[Event]) -> Result<docx_rs::Docx> {
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                Event::Start(tag) => {
                    match tag {
                        Tag::Heading { level, .. } => {
                            // Flush any accumulated text first
                            self.flush_paragraph();
                            let heading_size = match level {
                                HeadingLevel::H1 => 48,
                                HeadingLevel::H2 => 36,
                                HeadingLevel::H3 => 28,
                                HeadingLevel::H4 => 24,
                                HeadingLevel::H5 => 22,
                                HeadingLevel::H6 => 20,
                            };
                            // Consume heading content until End
                            let (runs, end_idx) = self.collect_inline(events, i + 1);
                            let mut para = docx_rs::Paragraph::new();
                            for run in runs {
                                para = para.add_run(run.size(heading_size).bold());
                            }
                            self.paragraphs.push(para.spacing_after(160));
                            i = end_idx + 1;
                            continue;
                        }
                        Tag::Paragraph => {
                            self.flush_paragraph();
                        }
                        Tag::Bold => self.bold = true,
                        Tag::Italic => self.italic = true,
                        Tag::Strikethrough => self.strikethrough = true,
                        Tag::Code(_) => self.code = true,
                        Tag::List(number) => {
                            self.list_level += 1;
                            self.list_ordered = number.is_some();
                            self.flush_paragraph();
                        }
                        Tag::Item => {
                            self.flush_paragraph();
                        }
                        Tag::BlockQuote => {
                            // Blockquote: just treat as indented paragraphs
                            self.flush_paragraph();
                        }
                        Tag::Table(_) => {
                            self.flush_paragraph();
                            let (table_para, end_idx) = self.collect_table(events, i + 1);
                            self.paragraphs.push(table_para);
                            i = end_idx + 1;
                            continue;
                        }
                        Tag::Link { dest, .. } => {
                            // We'll add the link text as a run with hyperlink
                            let _ = dest;
                            // Just add the text for now — docx-rs hyperlink support is limited
                        }
                        Tag::CodeBlock(lang) => {
                            self.flush_paragraph();
                            let (code_text, end_idx) = self.collect_code_block(events, i + 1, lang);
                            // Add as a code paragraph with monospace font
                            let run = docx_rs::Run::new()
                                .add_text(&code_text)
                                .size(20)
                                .font("Courier New");
                            let para = docx_rs::Paragraph::new()
                                .add_run(run)
                                .spacing_before(80)
                                .spacing_after(80)
                                .indent(docx_rs::Indent::new().left(360));
                            self.paragraphs.push(para);
                            i = end_idx + 1;
                            continue;
                        }
                        _ => {}
                    }
                }
                Event::End(tag) => {
                    match tag {
                        TagEnd::Paragraph => {
                            self.flush_paragraph();
                        }
                        TagEnd::Bold => self.bold = false,
                        TagEnd::Italic => self.italic = false,
                        TagEnd::Strikethrough => self.strikethrough = false,
                        TagEnd::Code => self.code = false,
                        TagEnd::List(_) => {
                            self.list_level = self.list_level.saturating_sub(1);
                        }
                        TagEnd::Item => {
                            self.flush_paragraph();
                        }
                        TagEnd::BlockQuote => {}
                        TagEnd::Link => {}
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    let mut run = docx_rs::Run::new().add_text(text.as_ref());
                    if self.bold {
                        run = run.bold();
                    }
                    if self.italic {
                        run = run.italic();
                    }
                    if self.strikethrough {
                        run = run.strike();
                    }
                    if self.code {
                        run = run.font("Courier New").size(20);
                    }
                    self.current_runs.push(run);
                }
                Event::Code(code) => {
                    // Inline code
                    let run = docx_rs::Run::new()
                        .add_text(code.as_ref())
                        .font("Courier New")
                        .size(20);
                    self.current_runs.push(run);
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.current_runs.push(docx_rs::Run::new().add_break());
                }
                Event::Rule => {
                    self.flush_paragraph();
                    // Horizontal rule — add a thin paragraph
                    self.paragraphs.push(
                        docx_rs::Paragraph::new()
                            .add_run(docx_rs::Run::new().add_text("─".repeat(40)))
                            .spacing_before(120)
                            .spacing_after(120),
                    );
                }
                Event::Html(html) => {
                    // Strip HTML tags, just use the text content
                    let text = html.replace('<', " ").replace('>', " ");
                    if !text.trim().is_empty() {
                        self.current_runs.push(docx_rs::Run::new().add_text(text.trim()));
                    }
                }
                _ => {}
            }
            i += 1;
        }

        self.flush_paragraph();

        // Build the document
        let mut doc = docx_rs::Docx::new();
        for para in self.paragraphs {
            doc = doc.add_paragraph(para);
        }

        Ok(doc)
    }

    /// Flush accumulated runs into a paragraph
    fn flush_paragraph(&mut self) {
        if self.current_runs.is_empty() {
            return;
        }

        let mut para = docx_rs::Paragraph::new();
        for run in std::mem::take(&mut self.current_runs) {
            para = para.add_run(run);
        }

        // Add list indentation
        if self.list_level > 0 {
            let indent = 360 * self.list_level as usize;
            para = para.indent(docx_rs::Indent::new().left(indent));
        }

        self.paragraphs.push(para.spacing_after(80));
    }

    /// Collect inline events into runs until the matching End event
    fn collect_inline(&mut self, events: &[Event], start: usize) -> (Vec<docx_rs::Run>, usize) {
        let mut runs = Vec::new();
        let mut depth = 1;
        let mut i = start;

        while i < events.len() && depth > 0 {
            match &events[i] {
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    depth -= 1;
                    if depth == 0 {
                        return (runs, i);
                    }
                }
                Event::Text(text) => {
                    let mut run = docx_rs::Run::new().add_text(text.as_ref());
                    if self.bold { run = run.bold(); }
                    if self.italic { run = run.italic(); }
                    runs.push(run);
                }
                Event::Code(code) => {
                    runs.push(docx_rs::Run::new().add_text(code.as_ref()).font("Courier New").size(20));
                }
                Event::SoftBreak | Event::HardBreak => {
                    runs.push(docx_rs::Run::new().add_break());
                }
                _ => {}
            }
            i += 1;
        }

        (runs, i.saturating_sub(1))
    }

    /// Collect a code block's content
    fn collect_code_block(
        &self,
        events: &[Event],
        start: usize,
        _lang: &str,
    ) -> (String, usize) {
        let mut code = String::new();
        let mut i = start;

        while i < events.len() {
            match &events[i] {
                Event::Text(text) => {
                    code.push_str(text.as_ref());
                }
                Event::End(TagEnd::CodeBlock) => {
                    return (code, i);
                }
                _ => {}
            }
            i += 1;
        }

        (code, i.saturating_sub(1))
    }

    /// Collect a table and render it as a DOCX table
    fn collect_table(&self, events: &[Event], start: usize) -> (docx_rs::Paragraph, usize) {
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut i = start;
        let mut in_cell = false;

        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                    current_row = Vec::new();
                }
                Event::End(TagEnd::TableHead) | Event::End(TagEnd::TableRow) => {
                    if !current_cell.is_empty() {
                        current_row.push(current_cell.trim().to_string());
                        current_cell = String::new();
                    }
                    rows.push(std::mem::take(&mut current_row));
                }
                Event::Start(Tag::TableCell) => {
                    in_cell = true;
                    current_cell = String::new();
                }
                Event::End(TagEnd::TableCell) => {
                    in_cell = false;
                    current_row.push(current_cell.trim().to_string());
                    current_cell = String::new();
                }
                Event::Text(text) if in_cell => {
                    current_cell.push_str(text.as_ref());
                }
                Event::Code(code) if in_cell => {
                    current_cell.push_str(code.as_ref());
                }
                Event::End(TagEnd::Table) => {
                    // Build a simple text representation of the table
                    let mut table_text = String::new();
                    for (ri, row) in rows.iter().enumerate() {
                        for (ci, cell) in row.iter().enumerate() {
                            if ci > 0 { table_text.push('\t'); }
                            table_text.push_str(cell);
                        }
                        if ri < rows.len() - 1 { table_text.push('\n'); }
                    }

                    // For header row, make it bold
                    let mut para = docx_rs::Paragraph::new().spacing_before(120).spacing_after(120);
                    if let Some(header_row) = rows.first() {
                        let header_text = header_row.join("\t");
                        para = para.add_run(docx_rs::Run::new().add_text(&header_text).bold().size(20));
                        if rows.len() > 1 {
                            para = para.add_run(docx_rs::Run::new().add_break());
                        }
                    }
                    if rows.len() > 1 {
                        let body_rows: Vec<String> = rows[1..].iter().map(|r| r.join("\t")).collect();
                        para = para.add_run(docx_rs::Run::new().add_text(&body_rows.join("\n")).size(20));
                    }

                    return (para, i);
                }
                _ => {}
            }
            i += 1;
        }

        (
            docx_rs::Paragraph::new().add_run(docx_rs::Run::new().add_text("[Table]")),
            i.saturating_sub(1),
        )
    }
}
