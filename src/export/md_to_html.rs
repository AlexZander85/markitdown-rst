//! Markdown → HTML converter using comrak
//!
//! Produces a self-contained HTML5 document with embedded CSS.
//! Comrak is already a project dependency (used by the preview module).

use crate::utils::OutputFormat;
use anyhow::Result;

/// Convert Markdown text to HTML.
///
/// When `OutputFormat::Html { standalone: true }`, wraps the output in a full
/// HTML5 document with embedded CSS for a professional look.
/// When `standalone: false`, returns just the `<body>` inner HTML.
pub fn markdown_to_html(markdown: &str, format: &OutputFormat) -> Result<String> {
    let (standalone, include_css) = match format {
        OutputFormat::Html { standalone, include_css } => (*standalone, *include_css),
        _ => (true, true), // default to standalone with CSS
    };

    // Configure comrak for good HTML output
    let mut options = comrak::Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.superscript = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.render.hardbreaks = false;
    options.render.github_pre_lang = true;

    let body_html = comrak::markdown_to_html(markdown, &options);

    if !standalone {
        return Ok(body_html);
    }

    // Build a standalone HTML5 document
    let css = if include_css { STYLESHEET } else { "" };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MDrust Export</title>
<style>
{css}
</style>
</head>
<body>
<article>
{body_html}
</article>
</body>
</html>"#,
    );

    Ok(html)
}

/// Professional stylesheet for HTML export
const STYLESHEET: &str = r#"
:root {
    --bg: #ffffff;
    --text: #1a1a2e;
    --heading: #16213e;
    --link: #0f3460;
    --code-bg: #f4f4f8;
    --border: #e0e0e6;
    --quote-border: #0f3460;
    --table-stripe: #f8f8fc;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg: #1a1a2e;
        --text: #e0e0e6;
        --heading: #e8e8f0;
        --link: #6c9bff;
        --code-bg: #252540;
        --border: #3a3a5c;
        --quote-border: #6c9bff;
        --table-stripe: #22223a;
    }
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    font-size: 16px;
    line-height: 1.7;
    color: var(--text);
    background: var(--bg);
    max-width: 820px;
    margin: 0 auto;
    padding: 2rem 1.5rem;
}

article { max-width: 100%; }

h1, h2, h3, h4, h5, h6 {
    color: var(--heading);
    margin-top: 1.8em;
    margin-bottom: 0.6em;
    line-height: 1.3;
}

h1 { font-size: 2em; border-bottom: 2px solid var(--border); padding-bottom: 0.3em; }
h2 { font-size: 1.6em; border-bottom: 1px solid var(--border); padding-bottom: 0.25em; }
h3 { font-size: 1.3em; }
h4 { font-size: 1.1em; }

p { margin-bottom: 1em; }

a { color: var(--link); text-decoration: none; }
a:hover { text-decoration: underline; }

code {
    font-family: "JetBrains Mono", "Fira Code", "Cascadia Code", Consolas, monospace;
    font-size: 0.88em;
    background: var(--code-bg);
    padding: 0.15em 0.4em;
    border-radius: 4px;
}

pre {
    background: var(--code-bg);
    padding: 1em 1.2em;
    border-radius: 8px;
    overflow-x: auto;
    margin-bottom: 1em;
    line-height: 1.5;
}

pre code {
    background: none;
    padding: 0;
    font-size: 0.85em;
}

blockquote {
    border-left: 4px solid var(--quote-border);
    padding: 0.5em 1em;
    margin: 1em 0;
    color: var(--text);
    opacity: 0.85;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 1em;
}

th, td {
    border: 1px solid var(--border);
    padding: 0.6em 0.9em;
    text-align: left;
}

th { background: var(--code-bg); font-weight: 600; }

tr:nth-child(even) { background: var(--table-stripe); }

ul, ol { margin: 0.5em 0 1em 1.5em; }

li { margin-bottom: 0.3em; }

hr { border: none; border-top: 1px solid var(--border); margin: 2em 0; }

img { max-width: 100%; height: auto; border-radius: 6px; }

del { opacity: 0.6; }

input[type="checkbox"] {
    margin-right: 0.5em;
    accent-color: var(--link);
}
"#;
