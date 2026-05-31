//! OCR module using Tesseract CLI subprocess
//!
//! This module provides OCR (Optical Character Recognition) functionality by:
//! - **Method**: Calling the `tesseract` CLI binary as a subprocess
//! - **Embedded tessdata**: Language files (eng, rus, chi_sim) are embedded at compile time
//!   via `include_bytes!` and extracted on first run
//!
//! # Why CLI subprocess instead of FFI?
//!
//! The previous `tesseract-ffi` feature used Rust FFI bindings to libtesseract,
//! but this required C/C++ libraries (libtesseract-dev, libleptonica-dev, vcpkg on Windows)
//! at **compile time**, making cross-platform CI builds extremely fragile.
//!
//! The CLI subprocess approach:
//! - **Zero compile-time C dependencies** — builds on any platform with just Rust
//! - **Same runtime requirement** — users need tesseract installed either way
//! - **More portable** — no pkg-config / vcpkg / cross-compilation issues
//! - **Same functionality** — identical OCR quality and language support
//!
//! # Prerequisites
//!
//! The `tesseract` CLI binary must be available on the system PATH.
//!
//! ## Installation
//! - **Linux (Debian/Ubuntu)**: `sudo apt install tesseract-ocr`
//! - **Linux (Fedora)**: `sudo dnf install tesseract`
//! - **Linux (Arch)**: `sudo pacman -S tesseract`
//! - **macOS**: `brew install tesseract`
//! - **Windows**: Download from <https://github.com/UB-Mannheim/tesseract/wiki> or `choco install tesseract`

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Embedded tessdata files (tessdata_fast)
// ---------------------------------------------------------------------------

const TESSDATA_ENG: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tessdata/eng.traineddata"));
const TESSDATA_RUS: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tessdata/rus.traineddata"));
const TESSDATA_CHI_SIM: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tessdata/chi_sim.traineddata"));

// ---------------------------------------------------------------------------
// OcrLanguage enum
// ---------------------------------------------------------------------------

/// Supported OCR languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OcrLanguage {
    /// English
    Eng,
    /// Russian
    Rus,
    /// Simplified Chinese
    ChiSim,
}

impl OcrLanguage {
    /// Return the Tesseract language code used on the command line.
    pub fn tesseract_code(&self) -> &'static str {
        match self {
            OcrLanguage::Eng => "eng",
            OcrLanguage::Rus => "rus",
            OcrLanguage::ChiSim => "chi_sim",
        }
    }

    /// Return the filename used for the traineddata file.
    pub fn filename(&self) -> &'static str {
        match self {
            OcrLanguage::Eng => "eng.traineddata",
            OcrLanguage::Rus => "rus.traineddata",
            OcrLanguage::ChiSim => "chi_sim.traineddata",
        }
    }

    /// Return the embedded bytes for this language.
    pub fn embedded_bytes(&self) -> &'static [u8] {
        match self {
            OcrLanguage::Eng => TESSDATA_ENG,
            OcrLanguage::Rus => TESSDATA_RUS,
            OcrLanguage::ChiSim => TESSDATA_CHI_SIM,
        }
    }
}

impl std::fmt::Display for OcrLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrLanguage::Eng => write!(f, "English"),
            OcrLanguage::Rus => write!(f, "Russian"),
            OcrLanguage::ChiSim => write!(f, "Simplified Chinese"),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Ensure that the tessdata files required for the given languages have been
/// extracted to the application tessdata directory. If a file already exists
/// and its size matches the embedded data, extraction is skipped.
///
/// If `languages` is empty, English is extracted as default.
pub fn ensure_tessdata(languages: &[OcrLanguage]) -> Result<()> {
    let langs: Vec<OcrLanguage> = if languages.is_empty() {
        vec![OcrLanguage::Eng]
    } else {
        languages.to_vec()
    };

    let tessdata_path = crate::utils::tessdata_dir();

    for lang in &langs {
        let file_path = tessdata_path.join(lang.filename());
        let bytes = lang.embedded_bytes();

        // Skip extraction if the file already exists with the correct size.
        if file_path.exists() {
            if let Ok(metadata) = fs::metadata(&file_path) {
                if metadata.len() as usize == bytes.len() {
                    continue;
                }
            }
        }

        fs::write(&file_path, bytes).with_context(|| {
            format!(
                "Failed to write tessdata file for {} to {}",
                lang,
                file_path.display()
            )
        })?;
    }

    Ok(())
}

/// Check whether Tesseract is available (CLI binary on PATH).
pub fn is_tesseract_available() -> bool {
    is_tesseract_cli_available()
}

/// Detailed Tesseract availability status for GUI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TesseractStatus {
    /// Tesseract CLI is available and ready
    Available,
    /// Tesseract CLI not found — tessdata embedded but engine not installed
    NotInstalled,
}

impl TesseractStatus {
    /// Check and return the current Tesseract status.
    pub fn check() -> Self {
        if is_tesseract_available() {
            TesseractStatus::Available
        } else {
            TesseractStatus::NotInstalled
        }
    }

    /// Get the status bar label for the GUI (English)
    pub fn status_label(&self) -> &'static str {
        match self {
            TesseractStatus::Available => "OCR: Tesseract OK",
            TesseractStatus::NotInstalled => "OCR: engine not installed",
        }
    }

    /// Get the tooltip/hint for the GUI (installation instructions)
    pub fn tooltip(&self) -> &'static str {
        match self {
            TesseractStatus::Available => "Tesseract OCR is available and ready",
            TesseractStatus::NotInstalled => "Tessdata (language models) are embedded, but the Tesseract engine must be installed separately.\n\
                Linux: sudo apt install tesseract-ocr\n\
                macOS: brew install tesseract\n\
                Windows: choco install tesseract",
        }
    }
}

/// Check if the `tesseract` CLI is available on the system `PATH`.
fn is_tesseract_cli_available() -> bool {
    std::process::Command::new("tesseract")
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run Tesseract OCR on an image file and return the recognised text as
/// Markdown.
///
/// # Arguments
/// * `image_path` - Path to the image file to OCR.
/// * `languages`  - Slice of [`OcrLanguage`] values to use for recognition.
///                  At least one language must be specified.
///
/// # Errors
/// Returns an error if:
/// - Tesseract is not available (CLI not on PATH).
/// - The image file does not exist.
/// - Tesseract exits with a non-zero status.
pub fn ocr_image_to_markdown(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    // --- Validate prerequisites ------------------------------------------------

    if languages.is_empty() {
        bail!("At least one OCR language must be specified");
    }

    if !image_path.exists() {
        bail!(
            "Image file does not exist: {}",
            image_path.display()
        );
    }

    // --- Ensure tessdata is extracted ------------------------------------------

    ensure_tessdata(languages)?;

    // --- Run OCR via CLI -------------------------------------------------------

    if !is_tesseract_cli_available() {
        bail!(
            "Tesseract OCR is not available.\n\n\
             Please install Tesseract:\n\n\
             \x20  Linux (Debian/Ubuntu):  sudo apt install tesseract-ocr\n\
             \x20  Linux (Fedora):         sudo dnf install tesseract\n\
             \x20  Linux (Arch):           sudo pacman -S tesseract\n\
             \x20  macOS:                  brew install tesseract\n\
             \x20  Windows:                choco install tesseract\n\
             \x20                          or download from https://github.com/UB-Mannheim/tesseract/wiki\n"
        );
    }

    ocr_via_cli(image_path, languages)
}

/// OCR via Tesseract CLI subprocess
fn ocr_via_cli(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    let lang_str: String = languages
        .iter()
        .map(|l| l.tesseract_code())
        .collect::<Vec<_>>()
        .join("+");

    let tessdata_parent = crate::utils::tessdata_dir()
        .parent()
        .context("tessdata directory has no parent")?
        .to_path_buf();

    let output = std::process::Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(&lang_str)
        .arg("--tessdata-dir")
        .arg(&tessdata_parent)
        .env("TESSDATA_PREFIX", &tessdata_parent)
        .output()
        .with_context(|| {
            format!(
                "Failed to execute `tesseract` on image {}",
                image_path.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Tesseract CLI failed with exit code {:?}:\n{}",
            output.status.code(),
            stderr.trim()
        );
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() {
        return Ok(String::new());
    }

    Ok(postprocess_to_markdown(&text))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Minimal post-processing of raw OCR text into Markdown.
fn postprocess_to_markdown(raw: &str) -> String {
    // Normalise line endings
    let text = raw.replace("\r\n", "\n").replace('\r', "\n");

    // Collapse 3+ consecutive newlines down to 2 (paragraph break)
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines: usize = 0;

    for ch in text.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            result.push(ch);
        }
    }

    result.trim().to_string()
}
