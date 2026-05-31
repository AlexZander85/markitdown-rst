//! OCR module with dual-engine architecture
//!
//! This module provides OCR (Optical Character Recognition) functionality using
//! two engines with automatic fallback:
//!
//! 1. **Built-in OCR (ocrs)** — Pure Rust, always available, English only.
//!    Neural network models are **embedded in the binary** (~11.7 MB) via
//!    `include_bytes!` and loaded directly into memory with
//!    `rten::Model::load_static_slice` — no download, no temp files, no internet.
//!
//! 2. **Tesseract CLI** — Optional enhanced engine with 100+ languages.
//!    Requires `tesseract` binary on PATH (auto-download available from GUI).
//!    Language data is downloaded on demand.
//!
//! # Architecture
//!
//! ```text
//! ocr_image_to_markdown(path, languages)
//!   │
//!   ├─ Tesseract available + language supported? → Tesseract CLI
//!   │
//!   └─ Fallback → ocrs (English only, built-in, models embedded)
//! ```
//!
//! # Why dual-engine?
//!
//! - ocrs: Pure Rust, compiles everywhere, models embedded in binary, English only
//! - Tesseract: Best quality, 100+ languages, but requires external installation
//! - Together: English works truly out-of-the-box (offline!), other languages need Tesseract

use anyhow::{bail, Context, Result};
use std::path::Path;

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

    /// Return the download URL for this language's tessdata file.
    pub fn download_url(&self) -> &'static str {
        match self {
            OcrLanguage::Eng => "https://github.com/tesseract-ocr/tessdata_fast/raw/main/eng.traineddata",
            OcrLanguage::Rus => "https://github.com/tesseract-ocr/tessdata_fast/raw/main/rus.traineddata",
            OcrLanguage::ChiSim => "https://github.com/tesseract-ocr/tessdata_fast/raw/main/chi_sim.traineddata",
        }
    }

    /// Whether this language is supported by the built-in ocrs engine.
    pub fn is_builtin_supported(&self) -> bool {
        matches!(self, OcrLanguage::Eng)
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
/// downloaded to the application tessdata directory. If a file already exists,
/// download is skipped.
///
/// If `languages` is empty, English is used as default.
pub fn ensure_tessdata(languages: &[OcrLanguage]) -> Result<()> {
    let langs: Vec<OcrLanguage> = if languages.is_empty() {
        vec![OcrLanguage::Eng]
    } else {
        languages.to_vec()
    };

    let tessdata_path = crate::utils::tessdata_dir();

    for lang in &langs {
        let file_path = tessdata_path.join(lang.filename());

        // Skip if already downloaded
        if file_path.exists() && std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0) > 0 {
            continue;
        }

        // Download tessdata file
        tracing::info!("Downloading tessdata for {}...", lang);
        download_tessdata(lang, &file_path)?;
    }

    Ok(())
}

/// Download a single tessdata file from GitHub.
fn download_tessdata(lang: &OcrLanguage, dest: &Path) -> Result<()> {
    let url = lang.download_url();
    let response = reqwest::blocking::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .with_context(|| format!("Failed to download tessdata for {}", lang))?;

    if !response.status().is_success() {
        bail!(
            "Failed to download tessdata for {}: HTTP {}",
            lang,
            response.status()
        );
    }

    let bytes = response.bytes().with_context(|| {
        format!("Failed to read tessdata response for {}", lang)
    })?;

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(dest, &bytes).with_context(|| {
        format!(
            "Failed to write tessdata file for {} to {}",
            lang,
            dest.display()
        )
    })?;

    tracing::info!("Downloaded tessdata for {} ({} bytes)", lang, bytes.len());
    Ok(())
}

/// Check whether Tesseract is available (CLI binary on PATH).
pub fn is_tesseract_available() -> bool {
    is_tesseract_cli_available()
}

/// Detailed OCR engine status for GUI display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrEngineStatus {
    /// Both ocrs and Tesseract are available
    BothAvailable,
    /// Only built-in ocrs is available (English only)
    BuiltinOnly,
    /// Only Tesseract is available (no built-in engine)
    TesseractOnly,
    /// Neither engine is available (should not happen with ocrs compiled in)
    NoneAvailable,
}

impl OcrEngineStatus {
    /// Check and return the current OCR engine status.
    pub fn check() -> Self {
        let has_builtin = cfg!(feature = "ocr");
        let has_tesseract = is_tesseract_cli_available();

        match (has_builtin, has_tesseract) {
            (true, true) => OcrEngineStatus::BothAvailable,
            (true, false) => OcrEngineStatus::BuiltinOnly,
            (false, true) => OcrEngineStatus::TesseractOnly,
            (false, false) => OcrEngineStatus::NoneAvailable,
        }
    }

    /// Get the status bar label for the GUI
    pub fn status_label(&self) -> &'static str {
        match self {
            OcrEngineStatus::BothAvailable => "OCR: ocrs + Tesseract",
            OcrEngineStatus::BuiltinOnly => "OCR: ocrs (English)",
            OcrEngineStatus::TesseractOnly => "OCR: Tesseract",
            OcrEngineStatus::NoneAvailable => "OCR: unavailable",
        }
    }

    /// Get the tooltip/hint for the GUI
    pub fn tooltip(&self) -> &'static str {
        match self {
            OcrEngineStatus::BothAvailable => "Built-in ocrs for English + Tesseract for all languages",
            OcrEngineStatus::BuiltinOnly => "Built-in ocrs engine (English, models embedded). Install Tesseract for more languages.",
            OcrEngineStatus::TesseractOnly => "Tesseract OCR is available. Built-in engine not compiled in.",
            OcrEngineStatus::NoneAvailable => "No OCR engine available. This should not happen.",
        }
    }
}

/// Legacy TesseractStatus — kept for backward compatibility with GUI code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TesseractStatus {
    /// Tesseract CLI is available and ready
    Available,
    /// Tesseract CLI not found
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

    /// Get the status bar label for the GUI
    pub fn status_label(&self) -> &'static str {
        match self {
            TesseractStatus::Available => "OCR: Tesseract OK",
            TesseractStatus::NotInstalled => "OCR: ocrs (built-in) | Tesseract: not installed",
        }
    }

    /// Get the tooltip/hint for the GUI
    pub fn tooltip(&self) -> &'static str {
        match self {
            TesseractStatus::Available => "Tesseract OCR is available. Built-in ocrs + Tesseract for all languages.",
            TesseractStatus::NotInstalled => "Built-in ocrs works for English (models embedded in binary, no download needed). Install Tesseract for Russian, Chinese, and 100+ languages.\n\
                Linux: sudo apt install tesseract-ocr\n\
                macOS: brew install tesseract\n\
                Windows: click 'Install Tesseract' button below.",
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

/// Run OCR on an image file and return the recognised text as Markdown.
///
/// Uses Tesseract if available and the language is supported, otherwise
/// falls back to the built-in ocrs engine (English only).
///
/// # Arguments
/// * `image_path` - Path to the image file to OCR.
/// * `languages`  - Slice of [`OcrLanguage`] values to use for recognition.
///                  At least one language must be specified.
///
/// # Errors
/// Returns an error if:
/// - The image file does not exist.
/// - No OCR engine can handle the requested language.
/// - The OCR engine fails.
pub fn ocr_image_to_markdown(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    if languages.is_empty() {
        bail!("At least one OCR language must be specified");
    }

    if !image_path.exists() {
        bail!("Image file does not exist: {}", image_path.display());
    }

    // Determine which engine to use
    let needs_tesseract = languages.iter().any(|l| !l.is_builtin_supported());
    let tesseract_available = is_tesseract_cli_available();

    if needs_tesseract && tesseract_available {
        // Use Tesseract for non-English languages
        return ocr_via_tesseract(image_path, languages);
    }

    if needs_tesseract && !tesseract_available {
        // User wants non-English but Tesseract is not installed
        // Fall back to ocrs with English and warn
        tracing::warn!(
            "Tesseract not available for {:?}. Falling back to built-in ocrs (English only).",
            languages
        );
    }

    // Use built-in ocrs engine
    ocr_via_builtin(image_path)
}

/// OCR using the built-in ocrs engine (English only).
///
/// Models are embedded directly in the binary via `include_bytes!` and loaded
/// from memory with `rten::Model::load_static_slice`. No downloads, no temp
/// files, no internet required — truly works offline out of the box.
fn ocr_via_builtin(image_path: &Path) -> Result<String> {
    use ocrs::{OcrEngine, ImageSource};

    // Load models from embedded bytes — zero I/O, zero network
    let det_model = rten::Model::load_static_slice(
        include_bytes!("../../models/text-detection.rten")
    ).with_context(|| "Failed to load embedded text detection model")?;
    let rec_model = rten::Model::load_static_slice(
        include_bytes!("../../models/text-recognition.rten")
    ).with_context(|| "Failed to load embedded text recognition model")?;

    let engine = OcrEngine::new(ocrs::OcrEngineParams {
        detection_model: Some(det_model),
        recognition_model: Some(rec_model),
        ..Default::default()
    })?;

    let img = image::open(image_path)
        .with_context(|| format!("Failed to open image: {}", image_path.display()))?
        .into_rgb8();

    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())
        .map_err(|e| anyhow::anyhow!("Failed to create ImageSource: {}", e))?;

    let ocr_input = engine.prepare_input(img_source)?;
    let text = engine.get_text(&ocr_input)?;

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    Ok(postprocess_to_markdown(&text))
}

/// OCR via Tesseract CLI subprocess
fn ocr_via_tesseract(image_path: &Path, languages: &[OcrLanguage]) -> Result<String> {
    // Ensure tessdata is downloaded
    ensure_tessdata(languages)?;

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
// Tesseract auto-install (GUI feature)
// ---------------------------------------------------------------------------

/// Download and install Tesseract for the current platform.
/// Returns the installed binary path on success.
///
/// This downloads pre-built Tesseract binaries and places them
/// in the MDrust data directory.
pub fn install_tesseract() -> Result<std::path::PathBuf> {
    let install_dir = crate::utils::app_data_dir().join("tesseract");
    std::fs::create_dir_all(&install_dir)?;

    #[cfg(target_os = "linux")]
    {
        bail!(
            "On Linux, install Tesseract via your package manager:\n\
             sudo apt install tesseract-ocr  (Debian/Ubuntu)\n\
             sudo dnf install tesseract      (Fedora)\n\
             sudo pacman -S tesseract        (Arch)"
        );
    }

    #[cfg(target_os = "macos")]
    {
        bail!(
            "On macOS, install Tesseract via Homebrew:\n\
             brew install tesseract"
        );
    }

    #[cfg(target_os = "windows")]
    {
        install_tesseract_windows(&install_dir)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        bail!("Automatic Tesseract installation is not supported on this platform.");
    }
}

#[cfg(target_os = "windows")]
fn install_tesseract_windows(install_dir: &Path) -> Result<std::path::PathBuf> {
    // Download UB-Mannheim Tesseract installer
    let url = "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.5.0.20241111.exe";
    let installer_path = install_dir.join("tesseract-installer.exe");

    tracing::info!("Downloading Tesseract installer...");
    let response = reqwest::blocking::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .context("Failed to download Tesseract installer")?;

    if !response.status().is_success() {
        bail!("Failed to download Tesseract: HTTP {}", response.status());
    }

    let bytes = response.bytes()?;
    std::fs::write(&installer_path, &bytes)?;

    // Run the installer silently
    let tesseract_dir = install_dir.join("tesseract");
    let status = std::process::Command::new(&installer_path)
        .arg("/S")
        .arg(format!("/D={}", tesseract_dir.display()))
        .spawn()?
        .wait()?;

    if !status.success() {
        bail!("Tesseract installer failed with exit code {:?}", status.code());
    }

    let exe_path = tesseract_dir.join("tesseract.exe");
    if exe_path.exists() {
        Ok(exe_path)
    } else {
        bail!("Tesseract was installed but executable not found at expected path");
    }
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
