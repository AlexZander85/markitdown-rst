//! OCR module using Tesseract — Rust FFI bindings with CLI fallback
//!
//! This module provides OCR (Optical Character Recognition) functionality by:
//! - **Primary**: Using the `tesseract` Rust crate (FFI bindings to libtesseract)
//!   This links directly to the Tesseract C library at compile time — no CLI subprocess needed.
//! - **Fallback**: If FFI is unavailable at runtime, falls back to calling `tesseract` CLI
//! - **Embedded tessdata**: Language files (eng, rus, chi_sim) are embedded at compile time
//!   via `include_bytes!` and extracted on first run
//!
//! **Important**: The tessdata (language models) are embedded in the binary, but the
//! Tesseract **engine** (libtesseract shared library or tesseract CLI) must be installed
//! on the system. It is NOT possible to statically link the entire Tesseract engine into
//! the binary due to its complex C++ dependencies (leptonica, libpng, libjpeg, libtiff, etc.).
//!
//! # Prerequisites
//! The `libtesseract` shared library must be available on the system.
//!
//! ## Installation
//! - **Linux (Debian/Ubuntu)**: `sudo apt install libtesseract-dev libleptonica-dev`
//! - **Linux (Fedora)**: `sudo dnf install tesseract-devel leptonica-devel`
//! - **Linux (Arch)**: `sudo pacman -S tesseract leptonica`
//! - **macOS**: `brew install tesseract leptonica`
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

/// Check whether Tesseract is available — either via FFI (libtesseract) or CLI.
///
/// Tries the FFI approach first (direct library call), then falls back to
/// checking for the `tesseract` CLI binary on PATH.
pub fn is_tesseract_available() -> bool {
    // Try FFI first — if the tesseract crate is compiled with libtesseract,
    // this should work. We do a simple API test.
    if is_tesseract_ffi_available() {
        return true;
    }

    // Fallback: check CLI
    is_tesseract_cli_available()
}

/// Detailed Tesseract availability status for GUI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TesseractStatus {
    /// Tesseract is available and ready (FFI or CLI)
    Available,
    /// Tesseract engine not found — tessdata embedded but engine not installed
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
                Linux: sudo apt install libtesseract-dev libleptonica-dev\n\
                macOS: brew install tesseract leptonica\n\
                Windows: choco install tesseract",
        }
    }
}

/// Check if Tesseract FFI (libtesseract) is available at runtime.
fn is_tesseract_ffi_available() -> bool {
    // The tesseract Rust crate links to libtesseract at compile time.
    // If the shared library is available at runtime, we can use it.
    // We test by trying to create a minimal API instance.
    //
    // Since we can't easily test without a tessdata path, we check if
    // the library can be loaded by attempting a simple init.
    #[cfg(feature = "tesseract-ffi")]
    {
        // Try to initialize tesseract with a minimal setup.
        // If this succeeds, libtesseract is available at runtime.
        let tessdata_dir = crate::utils::tessdata_dir();
        let tessdata_parent = tessdata_dir
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if let Ok(mut api) = tesseract::Tesseract::new(Some(&tessdata_parent), Some("eng")) {
            // Successfully created API — libtesseract is available
            drop(api);
            return true;
        }
    }

    #[allow(unreachable_code)]
    false
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
/// - Tesseract is not available (neither FFI nor CLI).
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

    // --- Try FFI first, then CLI fallback --------------------------------------

    // Try FFI approach
    if let Ok(text) = ocr_via_ffi(image_path, languages) {
        return Ok(text);
    }

    // Fallback to CLI
    if let Ok(text) = ocr_via_cli(image_path, languages) {
        return Ok(text);
    }

    bail!(
        "Tesseract OCR is not available.\n\n\
         Please install Tesseract:\n\n\
         \x20  Linux (Debian/Ubuntu):  sudo apt install libtesseract-dev libleptonica-dev\n\
         \x20  Linux (Fedora):         sudo dnf install tesseract-devel leptonica-devel\n\
         \x20  Linux (Arch):           sudo pacman -S tesseract leptonica\n\
         \x20  macOS:                  brew install tesseract leptonica\n\
         \x20  Windows:                choco install tesseract\n\
         \x20                          or download from https://github.com/UB-Mannheim/tesseract/wiki\n"
    );
}

/// OCR via Tesseract Rust FFI bindings (links to libtesseract directly)
#[cfg(feature = "tesseract-ffi")]
fn ocr_via_ffi(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    let tessdata_dir = crate::utils::tessdata_dir();
    let tessdata_parent = tessdata_dir
        .parent()
        .context("tessdata directory has no parent")?
        .to_string_lossy()
        .to_string();

    let lang_str: String = languages
        .iter()
        .map(|l| l.tesseract_code())
        .collect::<Vec<_>>()
        .join("+");

    let mut api = tesseract::Tesseract::new(Some(&tessdata_parent), Some(&lang_str))
        .map_err(|e| anyhow::anyhow!("Tesseract FFI init failed: {}", e))?;

    let image_path_str = image_path.to_string_lossy().to_string();
    api = api.set_image(&image_path_str)
        .map_err(|e| anyhow::anyhow!("Tesseract set_image failed: {}", e))?;

    let text = api.get_text()
        .map_err(|e| anyhow::anyhow!("Tesseract get_text failed: {}", e))?;

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    Ok(postprocess_to_markdown(&text))
}

/// OCR via Tesseract Rust FFI bindings — stub when tesseract-ffi feature is disabled
#[cfg(not(feature = "tesseract-ffi"))]
fn ocr_via_ffi(_image_path: &Path, _languages: &[OcrLanguage]) -> Result<String> {
    Err(anyhow::anyhow!("Tesseract FFI not available (tesseract-ffi feature disabled)"))
}

/// OCR via Tesseract CLI subprocess (fallback method)
fn ocr_via_cli(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    if !is_tesseract_cli_available() {
        return Err(anyhow::anyhow!("Tesseract CLI not found on PATH"));
    }

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
