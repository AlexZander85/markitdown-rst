## MDrust v1.3.0 — Reverse Conversion: MD → HTML & MD → DOCX

This major feature release adds the ability to export converted Markdown to HTML and DOCX formats.

### New Features

- **MD → HTML export** — Convert any document to a standalone HTML5 file with embedded CSS, auto dark/light mode, responsive layout, and professional typography. Uses `comrak` for rendering.
- **MD → DOCX export** — Convert any document to a Microsoft Word (.docx) file with proper headings, bold/italic/strikethrough, code blocks, tables, lists, and blockquotes. Uses `pulldown-cmark` for parsing and `docx-rs` for writing.
- **Output format selector in GUI** — New dropdown in the sidebar to choose between Markdown, HTML, and DOCX output formats.
- **CLI `--format` flag** — New `-f/--format` option for `convert` and `batch` commands: `md`, `html`, or `docx`.

### CLI Examples

```bash
# Convert PDF to HTML
mdrust-cli convert document.pdf -f html

# Convert DOCX to Word (re-export with MDrust formatting)
mdrust-cli convert report.docx -f docx -o reformatted.docx

# Batch convert to HTML
mdrust-cli batch ./docs -f html -o ./html-output

# Batch convert to Word
mdrust-cli batch ./docs -f docx -o ./word-output --combined
```

### Downloads

| File | Edition | OS | Arch |
|------|---------|----|------|
| `mdrust-full-linux-x64.tar.gz` | Full (GUI + OCR + Preview) | Linux | x86_64 |
| `mdrust-full-macos-x64.tar.gz` | Full | macOS | x86_64 |
| `mdrust-full-windows-x64.exe` | Full | Windows | x86_64 |
| `mdrust-light-linux-x64.tar.gz` | Light (GUI, no OCR) | Linux | x86_64 |
| `mdrust-light-macos-x64.tar.gz` | Light | macOS | x86_64 |
| `mdrust-light-windows-x64.exe` | Light | Windows | x86_64 |
| `mdrust-cli-linux-x64.tar.gz` | CLI-only (OCR) | Linux | x86_64 |
| `mdrust-cli-macos-x64.tar.gz` | CLI-only | macOS | x86_64 |
| `mdrust-cli-windows-x64.exe` | CLI-only | Windows | x86_64 |

---

**Full Changelog**: https://github.com/AlexZander85/MDrust/compare/v1.2.3...v1.3.0
