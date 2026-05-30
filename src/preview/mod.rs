//! Markdown Preview Module
//!
//! Generates beautiful HTML previews of Markdown files, inspired by mdhero.
//! Features Apple-inspired typography, dark/light theme support, KaTeX math
//! rendering, Mermaid diagrams, highlight.js syntax highlighting, and a
//! floating reader toolbar.

use std::fs;
use std::io::Write;

/// Generate a complete HTML5 preview document from Markdown source.
///
/// # Arguments
/// * `markdown` - The raw Markdown text to render
/// * `title` - The page title (used in `<title>` and `<h1>`)
/// * `dark_theme` - Whether to use the dark color scheme
/// * `font_size` - Base font size in pixels (e.g. 16)
///
/// # Returns
/// A complete HTML5 document as a `String`, with all CSS and JS inlined
/// (except CDN references for highlight.js, KaTeX, and Mermaid).
pub fn generate_preview_html(markdown: &str, title: &str, dark_theme: bool, font_size: u16) -> String {
    let body_class = if dark_theme { "dark" } else { "light" };

    // Convert markdown to HTML via comrak
    let rendered_html = comrak::markdown_to_html(markdown, &comrak::Options::default());

    // Pre-process mermaid code blocks: comrak renders ```mermaid as <code class="language-mermaid">
    // We need to convert those into <div class="mermaid"> for mermaid.js to process
    let processed_html = convert_mermaid_blocks(&rendered_html);

    let escaped_title = html_escape(title);

    // Inject the font_size into the CSS (replace placeholder)
    let css_with_font = CSS.replace("FONT_SIZE_PLACEHOLDER", &font_size.to_string());

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{escaped_title}</title>

<!-- KaTeX CSS -->
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">

<!-- highlight.js CSS (GitHub-inspired light theme, with dark override) -->
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css" id="hljs-light-theme">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css" id="hljs-dark-theme" disabled>

<style>
{css}
</style>
</head>
<body class="{body_class}">
<div class="md-container" id="md-container">
<div class="md-content">
{processed_html}
</div>
</div>

<!-- Floating toolbar -->
<div class="toolbar" id="toolbar">
  <button class="tb-btn" id="btn-theme" title="Toggle theme">🌓</button>
  <button class="tb-btn" id="btn-font-up" title="Increase font size">A+</button>
  <button class="tb-btn" id="btn-font-down" title="Decrease font size">A−</button>
  <button class="tb-btn" id="btn-line-height" title="Toggle line height">≡</button>
  <button class="tb-btn" id="btn-width" title="Toggle width">↔</button>
</div>

<!-- highlight.js -->
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>

<!-- KaTeX -->
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>

<!-- Mermaid -->
<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>

<script>
{js}
</script>
</body>
</html>"##,
        css = css_with_font,
        js = JS,
    )
}

/// Generate the preview HTML, save it to a temp file, and open it in the
/// default browser.
///
/// # Arguments
/// * `markdown` - The raw Markdown text to render
/// * `title` - The page title
/// * `dark_theme` - Whether to use the dark color scheme
pub fn open_preview_in_browser(markdown: &str, title: &str, dark_theme: bool) -> anyhow::Result<()> {
    let html = generate_preview_html(markdown, title, dark_theme, 16);

    // Write to a temp file
    let temp_dir = std::env::temp_dir();
    let safe_name = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    let file_path = temp_dir.join(format!("{}_preview.html", safe_name));

    let mut file = fs::File::create(&file_path)?;
    file.write_all(html.as_bytes())?;
    file.flush()?;

    // Open in default browser
    open::that(&file_path)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Convert mermaid `<pre><code class="language-mermaid">` blocks produced by
/// comrak into `<div class="mermaid">` blocks that mermaid.js can render.
fn convert_mermaid_blocks(html: &str) -> String {
    // Pattern: <pre><code class="language-mermaid">...\n</code></pre>
    // We use a simple state-machine approach since we can't rely on regex
    // working perfectly on arbitrary HTML.
    let mut result = String::with_capacity(html.len());
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
        // Look for <pre><code class="language-mermaid">
        if starts_with(bytes, i, b"<pre><code class=\"language-mermaid\">") {
            // Skip the opening tags
            i += b"<pre><code class=\"language-mermaid\">".len();
            // Collect content until </code></pre>
            let mut content = String::new();
            while i < bytes.len() && !starts_with(bytes, i, b"</code></pre>") {
                content.push(bytes[i] as char);
                i += 1;
            }
            // Skip the closing tags
            if starts_with(bytes, i, b"</code></pre>") {
                i += b"</code></pre>".len();
            }
            // Decode HTML entities that comrak may have produced
            let decoded = html_unescape(&content);
            result.push_str("<div class=\"mermaid\">");
            result.push_str(&decoded.trim());
            result.push_str("</div>");
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Check whether `bytes[pos..]` starts with `prefix`.
fn starts_with(bytes: &[u8], pos: usize, prefix: &[u8]) -> bool {
    if pos + prefix.len() > bytes.len() {
        return false;
    }
    &bytes[pos..pos + prefix.len()] == prefix
}

/// Minimal HTML entity decoding for common entities that comrak may emit
/// inside code blocks.
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Minimal HTML escaping for the title.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ===========================================================================
// Embedded CSS — Apple-inspired, mdhero-style
// ===========================================================================

const CSS: &str = r#"
/* ── Reset & Base ─────────────────────────────────────────────────────── */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

:root {
  --font-size: FONT_SIZE_PLACEHOLDERpx;
  --line-height: 1.7;
  --content-width: 800px;
  --transition-speed: 0.3s;

  /* Light theme (default) */
  --bg: #ffffff;
  --bg-secondary: #f6f8fa;
  --bg-code: #f6f8fa;
  --bg-inline-code: #f0f0f0;
  --bg-toolbar: rgba(255,255,255,0.85);
  --text: #1d1d1f;
  --text-secondary: #6e6e73;
  --heading: #1d1d1f;
  --link: #0071e3;
  --link-hover: #0058b0;
  --border: #d2d2d7;
  --border-h2: #d2d2d7;
  --blockquote-border: #0071e3;
  --blockquote-bg: #f0f5ff;
  --table-header-bg: #f6f8fa;
  --table-stripe: #fafafa;
  --mermaid-bg: #ffffff;
  --code-text: #24292e;
  --shadow: 0 1px 3px rgba(0,0,0,0.08);
}

body.dark {
  --bg: #0d0d0d;
  --bg-secondary: #1a1a1a;
  --bg-code: #1e1e1e;
  --bg-inline-code: #2d2d2d;
  --bg-toolbar: rgba(30,30,30,0.9);
  --text: #f5f5f7;
  --text-secondary: #a1a1a6;
  --heading: #f5f5f7;
  --link: #4da3ff;
  --link-hover: #7bb8ff;
  --border: #3a3a3c;
  --border-h2: #3a3a3c;
  --blockquote-border: #4da3ff;
  --blockquote-bg: #1a2233;
  --table-header-bg: #1e1e1e;
  --table-stripe: #141414;
  --mermaid-bg: #1e1e1e;
  --code-text: #e6edf3;
  --shadow: 0 1px 3px rgba(0,0,0,0.4);
}

html {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
    'Helvetica Neue', Arial, sans-serif;
  font-size: var(--font-size);
  line-height: var(--line-height);
  color: var(--text);
  background-color: var(--bg);
  transition: background-color var(--transition-speed) ease,
              color var(--transition-speed) ease;
  padding: 0;
  margin: 0;
}

/* ── Layout ───────────────────────────────────────────────────────────── */
.md-container {
  max-width: var(--content-width);
  margin: 0 auto;
  padding: 2rem 1.5rem 6rem;
  transition: max-width var(--transition-speed) ease;
}

.md-content {
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* ── Headings ─────────────────────────────────────────────────────────── */
h1, h2, h3, h4, h5, h6 {
  font-weight: 700;
  color: var(--heading);
  line-height: 1.25;
  margin-top: 2em;
  margin-bottom: 0.6em;
  transition: color var(--transition-speed) ease;
}

h1 {
  font-size: 2em;
  letter-spacing: -0.02em;
  margin-top: 0;
  padding-bottom: 0.3em;
}

h2 {
  font-size: 1.5em;
  letter-spacing: -0.01em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border-h2);
  transition: border-color var(--transition-speed) ease;
}

h3 {
  font-size: 1.25em;
}

h4 {
  font-size: 1.1em;
}

h5, h6 {
  font-size: 1em;
}

/* ── Paragraphs ───────────────────────────────────────────────────────── */
p {
  margin-bottom: 1.2em;
}

/* ── Links ────────────────────────────────────────────────────────────── */
a {
  color: var(--link);
  text-decoration: none;
  transition: color 0.2s ease;
}

a:hover {
  color: var(--link-hover);
  text-decoration: underline;
}

/* ── Images ───────────────────────────────────────────────────────────── */
img {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  display: block;
  margin: 1.5em auto;
  box-shadow: var(--shadow);
}

/* ── Inline Code ──────────────────────────────────────────────────────── */
code {
  font-family: 'SF Mono', 'Fira Code', 'Fira Mono', 'Roboto Mono',
    'Courier New', monospace;
  font-size: 0.875em;
}

p > code, li > code, td > code {
  background: var(--bg-inline-code);
  padding: 2px 6px;
  border-radius: 4px;
  transition: background var(--transition-speed) ease;
}

/* ── Code Blocks ──────────────────────────────────────────────────────── */
pre {
  background: var(--bg-code);
  border-radius: 12px;
  padding: 16px;
  overflow-x: auto;
  margin: 1.2em 0;
  transition: background var(--transition-speed) ease;
}

pre code {
  background: none;
  padding: 0;
  border-radius: 0;
  font-size: 0.85em;
  line-height: 1.6;
  color: var(--code-text);
}

/* ── Blockquotes ──────────────────────────────────────────────────────── */
blockquote {
  border-left: 4px solid var(--blockquote-border);
  background: var(--blockquote-bg);
  padding: 0.8em 1.2em;
  margin: 1.2em 0;
  border-radius: 0 8px 8px 0;
  font-style: italic;
  color: var(--text-secondary);
  transition: border-color var(--transition-speed) ease,
              background var(--transition-speed) ease,
              color var(--transition-speed) ease;
}

blockquote p {
  margin-bottom: 0;
}

blockquote blockquote {
  margin-top: 0.8em;
}

/* ── Lists ────────────────────────────────────────────────────────────── */
ul, ol {
  padding-left: 1.8em;
  margin-bottom: 1.2em;
}

li {
  margin-bottom: 0.4em;
}

li > ul, li > ol {
  margin-bottom: 0;
  margin-top: 0.3em;
}

/* ── Tables ───────────────────────────────────────────────────────────── */
table {
  width: 100%;
  border-collapse: collapse;
  margin: 1.5em 0;
  font-size: 0.92em;
  overflow-x: auto;
  display: block;
}

thead {
  background: var(--table-header-bg);
  transition: background var(--transition-speed) ease;
}

th {
  font-weight: 600;
  text-align: left;
  padding: 10px 14px;
  border-bottom: 2px solid var(--border);
  color: var(--heading);
  transition: border-color var(--transition-speed) ease,
              color var(--transition-speed) ease;
}

td {
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  transition: border-color var(--transition-speed) ease;
}

tbody tr:nth-child(even) {
  background: var(--table-stripe);
  transition: background var(--transition-speed) ease;
}

/* ── Horizontal Rule ──────────────────────────────────────────────────── */
hr {
  border: none;
  height: 1px;
  background: var(--border);
  margin: 2.5em 0;
  transition: background var(--transition-speed) ease;
}

/* ── KaTeX ────────────────────────────────────────────────────────────── */
.katex-display {
  margin: 1.2em 0;
  overflow-x: auto;
  overflow-y: hidden;
}

/* ── Mermaid ──────────────────────────────────────────────────────────── */
.mermaid {
  text-align: center;
  margin: 1.5em 0;
  padding: 1em;
  background: var(--mermaid-bg);
  border-radius: 12px;
  overflow-x: auto;
  transition: background var(--transition-speed) ease;
}

body.dark .mermaid {
  filter: invert(0.85) hue-rotate(180deg);
}

/* ── Task Lists ───────────────────────────────────────────────────────── */
.task-list-item {
  list-style-type: none;
  margin-left: -1.5em;
}

.task-list-item input[type="checkbox"] {
  margin-right: 0.5em;
  transform: scale(1.1);
  accent-color: var(--link);
}

/* ── Footnotes ────────────────────────────────────────────────────────── */
.footnote-definition {
  font-size: 0.9em;
  margin-bottom: 0.8em;
}

/* ── Toolbar ──────────────────────────────────────────────────────────── */
.toolbar {
  position: fixed;
  bottom: 1.5rem;
  right: 1.5rem;
  display: flex;
  gap: 6px;
  padding: 6px;
  background: var(--bg-toolbar);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border);
  border-radius: 14px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.12);
  z-index: 9999;
  transition: background var(--transition-speed) ease,
              border-color var(--transition-speed) ease,
              box-shadow var(--transition-speed) ease;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 10px;
  background: transparent;
  color: var(--text);
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease, transform 0.15s ease;
  user-select: none;
  -webkit-user-select: none;
  line-height: 1;
}

.tb-btn:hover {
  background: var(--bg-inline-code);
  transform: scale(1.08);
}

.tb-btn:active {
  transform: scale(0.95);
}

/* ── Selection ────────────────────────────────────────────────────────── */
::selection {
  background: rgba(0,113,227,0.2);
  color: inherit;
}

body.dark ::selection {
  background: rgba(77,163,255,0.25);
}

/* ── Scrollbar ────────────────────────────────────────────────────────── */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

/* ── Print ────────────────────────────────────────────────────────────── */
@media print {
  body {
    background: #fff;
    color: #000;
  }

  .toolbar {
    display: none !important;
  }

  .md-container {
    max-width: 100%;
    padding: 0;
  }

  a {
    color: #000;
    text-decoration: underline;
  }

  pre, blockquote, table, .mermaid {
    break-inside: avoid;
  }

  pre {
    border: 1px solid #ccc;
  }
}

/* ── Responsive ───────────────────────────────────────────────────────── */
@media (max-width: 640px) {
  .md-container {
    padding: 1rem 0.75rem 5rem;
  }

  h1 { font-size: 1.6em; }
  h2 { font-size: 1.3em; }
  h3 { font-size: 1.1em; }

  pre {
    border-radius: 8px;
    padding: 12px;
  }

  .toolbar {
    bottom: 0.75rem;
    right: 0.75rem;
    gap: 4px;
    padding: 4px;
  }

  .tb-btn {
    width: 32px;
    height: 32px;
    font-size: 12px;
  }
}
"#;

// ===========================================================================
// Embedded JavaScript
// ===========================================================================

const JS: &str = r#"
(function() {
  'use strict';

  // ── Replace FONT_SIZE_PLACEHOLDER with the actual size ──────────────
  // (The Rust code puts the real value into the CSS var already, but we
  //  keep a JS-side reference for the +/- buttons.)
  var root = document.documentElement;
  var currentFontSize = parseFloat(getComputedStyle(root).getPropertyValue('--font-size'));

  // ── highlight.js ────────────────────────────────────────────────────
  if (typeof hljs !== 'undefined') {
    hljs.highlightAll();
  }

  // ── KaTeX auto-render ──────────────────────────────────────────────
  function renderKatex() {
    if (typeof renderMathInElement !== 'undefined') {
      renderMathInElement(document.querySelector('.md-content'), {
        delimiters: [
          {left: '$$', right: '$$', display: true},
          {left: '$',  right: '$',  display: false},
          {left: '\\(', right: '\\)', display: false},
          {left: '\\[', right: '\\]', display: true}
        ],
        throwOnError: false
      });
    }
  }
  renderKatex();

  // ── Mermaid ─────────────────────────────────────────────────────────
  function initMermaid() {
    if (typeof mermaid !== 'undefined') {
      var isDark = document.body.classList.contains('dark');
      mermaid.initialize({
        startOnLoad: false,
        theme: isDark ? 'dark' : 'default',
        securityLevel: 'loose',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif'
      });
      mermaid.run({ querySelector: '.mermaid' });
    }
  }
  initMermaid();

  // ── Theme toggle ───────────────────────────────────────────────────
  var btnTheme = document.getElementById('btn-theme');
  btnTheme.addEventListener('click', function() {
    var body = document.body;
    var isNowDark = body.classList.toggle('dark');
    body.classList.toggle('light', !isNowDark);

    // Swap hljs theme stylesheets
    var lightSheet = document.getElementById('hljs-light-theme');
    var darkSheet  = document.getElementById('hljs-dark-theme');
    if (lightSheet && darkSheet) {
      lightSheet.disabled = isNowDark;
      darkSheet.disabled  = !isNowDark;
    }

    // Re-init mermaid with correct theme
    if (typeof mermaid !== 'undefined') {
      var mermaidDivs = document.querySelectorAll('.mermaid');
      mermaidDivs.forEach(function(div) {
        // Remove the processed diagram so mermaid can re-render
        div.removeAttribute('data-processed');
        div.innerHTML = div.getAttribute('data-original') || div.textContent;
      });
      mermaid.initialize({
        startOnLoad: false,
        theme: isNowDark ? 'dark' : 'default',
        securityLevel: 'loose',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif'
      });
      mermaid.run({ querySelector: '.mermaid' });
    }
  });

  // ── Font size +/- ──────────────────────────────────────────────────
  var btnFontUp   = document.getElementById('btn-font-up');
  var btnFontDown = document.getElementById('btn-font-down');

  btnFontUp.addEventListener('click', function() {
    currentFontSize = Math.min(currentFontSize + 2, 32);
    root.style.setProperty('--font-size', currentFontSize + 'px');
  });

  btnFontDown.addEventListener('click', function() {
    currentFontSize = Math.max(currentFontSize - 2, 10);
    root.style.setProperty('--font-size', currentFontSize + 'px');
  });

  // ── Line height toggle ─────────────────────────────────────────────
  var lineHeights = [1.5, 1.7, 2.0];
  var lhIndex = 1; // default 1.7
  var btnLH = document.getElementById('btn-line-height');

  btnLH.addEventListener('click', function() {
    lhIndex = (lhIndex + 1) % lineHeights.length;
    root.style.setProperty('--line-height', lineHeights[lhIndex]);
  });

  // ── Width toggle ───────────────────────────────────────────────────
  var widths = [680, 800, 920];
  var wIndex = 1; // default 800
  var btnWidth = document.getElementById('btn-width');

  btnWidth.addEventListener('click', function() {
    wIndex = (wIndex + 1) % widths.length;
    root.style.setProperty('--content-width', widths[wIndex] + 'px');
  });

  // ── Store original mermaid source for re-render on theme switch ────
  document.querySelectorAll('.mermaid').forEach(function(div) {
    div.setAttribute('data-original', div.textContent);
  });

})();
"#;

// ===========================================================================
// Unit tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preview_light() {
        let html = generate_preview_html("# Hello", "Test", false, 16);
        assert!(html.contains(r#"class="light""#));
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("hljs.highlightAll"));
        assert!(html.contains("katex"));
        assert!(html.contains("mermaid"));
    }

    #[test]
    fn test_generate_preview_dark() {
        let html = generate_preview_html("# Hello", "Test", true, 18);
        assert!(html.contains(r#"class="dark""#));
        assert!(html.contains("--font-size: 18px"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("A&B<C>D"), "A&amp;B&lt;C&gt;D");
    }

    #[test]
    fn test_mermaid_conversion() {
        let input = r#"<pre><code class="language-mermaid">graph TD; A--&gt;B</code></pre>"#;
        let result = convert_mermaid_blocks(input);
        assert!(result.contains(r#"<div class="mermaid">"#));
        assert!(result.contains("graph TD; A-->B"));
        assert!(!result.contains("<pre>"));
    }

    #[test]
    fn test_mermaid_no_match() {
        let input = r#"<pre><code class="language-rust">fn main() {}</code></pre>"#;
        let result = convert_mermaid_blocks(input);
        assert_eq!(result, input);
    }
}
