//! MarkItDown-RST — Multi-threaded Document-to-Markdown Converter
//!
//! A Rust rewrite of markitdown-gui with:
//! - Multi-threaded batch processing (tokio + semaphore)
//! - Optional Tesseract OCR with embedded tessdata (eng/rus/chi_sim) — `ocr` feature
//! - Optional Markdown viewer/editor with highlight.js, KaTeX, Mermaid — `preview` feature
//! - Multilingual UI (RU/EN/ZH)
//! - All in one compact binary

pub mod batch;
pub mod converters;
pub mod gui;
pub mod i18n;
pub mod utils;

#[cfg(feature = "ocr")]
pub mod ocr;

#[cfg(feature = "preview")]
pub mod preview;

fn main() -> eframe::Result<()> {
    // Check if CLI mode is requested
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && (args[1] == "--cli" || args.contains(&"--cli".to_string())) {
        println!("Use markitdown-cli binary for command-line mode.");
        std::process::exit(0);
    }

    // Run GUI
    gui::run_gui()
}
