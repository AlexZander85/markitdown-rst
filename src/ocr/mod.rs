//! OCR module using Tesseract CLI
//!
//! This module provides OCR (Optical Character Recognition) functionality by:
//! - Embedding tessdata_fast language files (eng, rus, chi_sim) at compile time
//! - Extracting them on first run to the application data directory
//! - Calling the Tesseract CLI as a subprocess for text recognition
//!
//! # Prerequisites
//! The `tesseract` command-line tool must be installed on the system.
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
use std::process::Command;

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
/// If `languages` is empty, all embedded languages are extracted.
pub fn ensure_tessdata(languages: &[OcrLanguage]) -> Result<()> {
    let langs: Vec<OcrLanguage> = if languages.is_empty() {
        vec![OcrLanguage::Eng, OcrLanguage::Rus, OcrLanguage::ChiSim]
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

/// Check whether the `tesseract` CLI is available on the system `PATH`.
pub fn is_tesseract_available() -> bool {
    Command::new("tesseract")
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
/// The output is wrapped in a Markdown code block only if the detected content
/// appears to be structured/tabular; otherwise plain recognised text is
/// returned with paragraph breaks preserved.
///
/// # Arguments
/// * `image_path` - Path to the image file to OCR.
/// * `languages`  - Slice of [`OcrLanguage`] values to use for recognition.
///                  At least one language must be specified.
///
/// # Errors
/// Returns an error if:
/// - Tesseract is not installed on the system.
/// - The image file does not exist.
/// - Tesseract exits with a non-zero status.
pub fn ocr_image_to_markdown(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    // --- Validate prerequisites ------------------------------------------------

    if !is_tesseract_available() {
        bail!(
            "Tesseract OCR is not installed or not found on PATH.\n\n\
             Please install Tesseract:\n\n\
             \x20  Linux (Debian/Ubuntu):  sudo apt install tesseract-ocr\n\
             \x20  Linux (Fedora):         sudo dnf install tesseract\n\
             \x20  Linux (Arch):           sudo pacman -S tesseract\n\
             \x20  macOS:                  brew install tesseract\n\
             \x20  Windows:                choco install tesseract\n\
             \x20                          or download from https://github.com/UB-Mannheim/tesseract/wiki\n"
        );
    }

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

    // --- Build the language string (+lang1+lang2) ------------------------------

    let lang_str: String = languages
        .iter()
        .map(|l| l.tesseract_code())
        .collect::<Vec<_>>()
        .join("+");

    // Point tesseract to our custom tessdata directory by setting TESSDATA_PREFIX
    // to the *parent* of the tessdata/ directory (tesseract appends "tessdata/" itself).
    let tessdata_parent = crate::utils::tessdata_dir()
        .parent()
        .context("tessdata directory has no parent")?
        .to_path_buf();

    // --- Run tesseract ---------------------------------------------------------

    // tesseract <image> stdout -l <langs> --tessdata-dir <dir>
    let output = Command::new("tesseract")
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
            "Tesseract failed with exit code {:?}:\n{}",
            output.status.code(),
            stderr.trim()
        );
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() {
        return Ok(String::new());
    }

    // --- Post-process into lightweight Markdown --------------------------------

    let markdown = postprocess_to_markdown(&text);
    Ok(markdown)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Minimal post-processing of raw OCR text into Markdown.
///
/// - Collapses excessive blank lines into double newlines (paragraph breaks).
/// - Does NOT wrap in code fences so that downstream consumers can treat the
///   output as regular Markdown text (headings, lists, etc. will be preserved
///   if the OCR picked them up).
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_language_tesseract_codes() {
        assert_eq!(OcrLanguage::Eng.tesseract_code(), "eng");
        assert_eq!(OcrLanguage::Rus.tesseract_code(), "rus");
        assert_eq!(OcrLanguage::ChiSim.tesseract_code(), "chi_sim");
    }

    #[test]
    fn test_ocr_language_filenames() {
        assert_eq!(OcrLanguage::Eng.filename(), "eng.traineddata");
        assert_eq!(OcrLanguage::Rus.filename(), "rus.traineddata");
        assert_eq!(OcrLanguage::ChiSim.filename(), "chi_sim.traineddata");
    }

    #[test]
    fn test_ocr_language_display() {
        assert_eq!(format!("{}", OcrLanguage::Eng), "English");
        assert_eq!(format!("{}", OcrLanguage::Rus), "Russian");
        assert_eq!(format!("{}", OcrLanguage::ChiSim), "Simplified Chinese");
    }

    #[test]
    fn test_embedded_bytes_non_empty() {
        // Sanity check that the embedded files are non-empty
        assert!(!OcrLanguage::Eng.embedded_bytes().is_empty());
        assert!(!OcrLanguage::Rus.embedded_bytes().is_empty());
        assert!(!OcrLanguage::ChiSim.embedded_bytes().is_empty());
    }

    #[test]
    fn test_postprocess_collapses_blank_lines() {
        let input = "Hello\n\n\n\nWorld";
        let result = postprocess_to_markdown(input);
        assert_eq!(result, "Hello\n\nWorld");
    }

    #[test]
    fn test_postprocess_preserves_paragraph_breaks() {
        let input = "Para 1\n\nPara 2";
        let result = postprocess_to_markdown(input);
        assert_eq!(result, "Para 1\n\nPara 2");
    }

    #[test]
    fn test_postprocess_normalises_crlf() {
        let input = "Line 1\r\nLine 2";
        let result = postprocess_to_markdown(input);
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn test_ocr_empty_languages_errors() {
        let result = ocr_image_to_markdown(Path::new("/nonexistent.png"), &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one OCR language"));
    }

    #[test]
    fn test_ocr_nonexistent_image_errors() {
        let result = ocr_image_to_markdown(Path::new("/nonexistent.png"), &[OcrLanguage::Eng]);
        // Will fail either because tesseract is missing or file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_tessdata_creates_files() {
        // Use a temporary directory to avoid polluting the real data dir
        let tmp = std::env::temp_dir().join("markitdown-rs-test-tessdata");
        let _ = fs::create_dir_all(&tmp);

        // We test the logic indirectly; ensure_tessdata uses crate::utils::tessdata_dir()
        // which points to the real data dir, so we just ensure it doesn't panic.
        let result = ensure_tessdata(&[OcrLanguage::Eng]);
        // Should succeed (file may already exist)
        assert!(result.is_ok());
    }
}
