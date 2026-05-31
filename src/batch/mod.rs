//! Multi-threaded batch processor for document conversion
//!
//! Uses tokio for async I/O and a Semaphore for concurrency control,
//! with FuturesUnordered for streaming results (lower memory than join_all).

use crate::converters::ConversionResult;
use crate::utils::{OutputFormat, collect_files};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// OCR language type — only available with the `ocr` feature
#[cfg(feature = "ocr")]
use crate::ocr::OcrLanguage;

/// Status of a single file in the batch
#[derive(Debug, Clone)]
pub enum FileStatus {
    Pending,
    Converting(f32),
    Completed,
    Failed(String),
    Skipped(String),
}

impl FileStatus {
    pub fn label(&self) -> &str {
        match self {
            FileStatus::Pending => "Pending",
            FileStatus::Converting(_) => "Converting",
            FileStatus::Completed => "Done",
            FileStatus::Failed(_) => "Failed",
            FileStatus::Skipped(_) => "Skipped",
        }
    }
}

/// Information about a file in the batch queue
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub status: FileStatus,
    pub file_size: u64,
    pub format: String,
    pub name: String,
}

/// Result of a batch conversion
#[derive(Debug)]
pub struct BatchResult {
    pub successes: Vec<(PathBuf, ConversionResult)>,
    pub failures: Vec<(PathBuf, String)>,
    pub total_files: usize,
    pub total_time_secs: f64,
}

impl BatchResult {
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.successes.len() as f64 / self.total_files as f64) * 100.0
        }
    }

    /// Get total word count across all successful conversions
    pub fn total_word_count(&self) -> usize {
        self.successes
            .iter()
            .map(|(_, r)| r.metadata.word_count)
            .sum()
    }

    /// Save all successful conversions to a directory
    pub async fn save_all<P: AsRef<Path>>(&self, output_dir: P, combined: bool) -> Result<()> {
        let output_dir = output_dir.as_ref();
        tokio::fs::create_dir_all(output_dir).await?;

        // Determine the output format from the first successful result
        // (all results share the same format from BatchProcessor)
        let output_format = self
            .successes
            .first()
            .map(|(_, r)| r.output_format.clone())
            .unwrap_or_default();

        let ext = output_format.extension();

        if combined {
            match &output_format {
                OutputFormat::Html { .. } => {
                    // For HTML combined output, create a single HTML file
                    let combined_md: String = self
                        .successes
                        .iter()
                        .map(|(path, result)| {
                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            format!("## Source: {}\n\n{}", filename, result.full_markdown())
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");

                    let html = crate::export::md_to_html::markdown_to_html(
                        &combined_md,
                        &output_format,
                    )?;
                    let output_path = output_dir.join(format!("combined_output.{}", ext));
                    tokio::fs::write(&output_path, html).await?;
                }
                OutputFormat::Docx => {
                    // For DOCX, combine all markdown first, then convert once
                    let combined_md: String = self
                        .successes
                        .iter()
                        .map(|(path, result)| {
                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            format!("# Source: {}\n\n{}", filename, result.full_markdown())
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");

                    let docx_bytes = crate::export::md_to_docx::markdown_to_docx(
                        &combined_md,
                        Some("MDrust Combined Output"),
                    )?;
                    let output_path = output_dir.join(format!("combined_output.{}", ext));
                    tokio::fs::write(&output_path, docx_bytes).await?;
                }
                _ => {
                    // Markdown / JSON — original behavior
                    let combined_markdown: String = self
                        .successes
                        .iter()
                        .map(|(path, result)| {
                            let filename = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            format!("# Source: {}\n\n{}", filename, result.full_markdown())
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");

                    let output_path = output_dir.join(format!("combined_output.{}", ext));
                    tokio::fs::write(&output_path, combined_markdown).await?;
                }
            }
        } else {
            for (input_path, result) in &self.successes {
                let filename = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let output_path = output_dir.join(format!("{}.{}", filename, ext));
                result.save_to_file(&output_path).await?;
            }
        }

        Ok(())
    }
}

/// Multi-threaded batch processor
pub struct BatchProcessor {
    files: Vec<FileEntry>,
    output_format: OutputFormat,
    parallel_jobs: usize,
    #[cfg(feature = "ocr")]
    ocr_languages: Vec<OcrLanguage>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            output_format: OutputFormat::default(),
            parallel_jobs: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            #[cfg(feature = "ocr")]
            ocr_languages: vec![OcrLanguage::Eng],
        }
    }

    /// Add files from paths (supports files and directories)
    pub fn add_paths(mut self, paths: &[PathBuf]) -> Self {
        let collected = collect_files(paths);
        for path in collected {
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let format = crate::utils::detect_format(&path);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            self.files.push(FileEntry {
                path,
                status: FileStatus::Pending,
                file_size,
                format: format.to_string(),
                name,
            });
        }
        self
    }

    /// Set output format
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Set number of parallel jobs
    pub fn parallel(mut self, jobs: usize) -> Self {
        self.parallel_jobs = jobs.max(1);
        self
    }

    /// Set OCR languages (only available with `ocr` feature)
    #[cfg(feature = "ocr")]
    pub fn ocr_languages(mut self, languages: Vec<OcrLanguage>) -> Self {
        self.ocr_languages = languages;
        self
    }

    /// Get the list of files in the batch
    pub fn files(&self) -> &[FileEntry] {
        &self.files
    }

    /// Get the total number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Execute the batch conversion with multi-threaded parallelism
    pub async fn execute(self) -> Result<BatchResult> {
        let start_time = std::time::Instant::now();
        let total_files = self.files.len();

        if total_files == 0 {
            return Ok(BatchResult {
                successes: Vec::new(),
                failures: Vec::new(),
                total_files: 0,
                total_time_secs: 0.0,
            });
        }

        tracing::info!(
            "Starting batch conversion: {} files, {} parallel jobs",
            total_files,
            self.parallel_jobs
        );

        let semaphore = Arc::new(Semaphore::new(self.parallel_jobs));
        let completed = Arc::new(AtomicUsize::new(0));
        let output_format = Arc::new(self.output_format);

        let file_paths: Vec<PathBuf> = self.files.iter().map(|f| f.path.clone()).collect();

        use futures_util::stream::{FuturesUnordered, StreamExt};

        let mut tasks = FuturesUnordered::new();

        for file_path in file_paths {
            let sem = semaphore.clone();
            let counter = completed.clone();
            let out_fmt = output_format.clone();

            #[cfg(feature = "ocr")]
            let ocr_langs = self.ocr_languages.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.ok()?;

                #[cfg(feature = "ocr")]
                let result = convert_file(&file_path, &out_fmt, &ocr_langs).await;

                #[cfg(not(feature = "ocr"))]
                let result = convert_file(&file_path, &out_fmt).await;

                let _done = counter.fetch_add(1, Ordering::Relaxed) + 1;
                Some((file_path, result))
            });

            tasks.push(task);
        }

        let mut successes = Vec::new();
        let mut failures = Vec::new();

        while let Some(joined) = tasks.next().await {
            match joined {
                Ok(Some((path, Ok(mut conversion)))) => {
                    // Override the output format — converters always produce Markdown internally,
                    // but the desired export format is set by the BatchProcessor
                    conversion.output_format = (*output_format).clone();
                    successes.push((path, conversion));
                }
                Ok(Some((path, Err(e)))) => {
                    tracing::warn!("Failed to convert {}: {}", path.display(), e);
                    failures.push((path, e.to_string()));
                }
                Ok(None) => {}
                Err(join_error) => {
                    tracing::error!("Task join error: {}", join_error);
                    failures.push((PathBuf::new(), join_error.to_string()));
                }
            }
        }

        let total_time = start_time.elapsed();

        let batch_result = BatchResult {
            successes,
            failures,
            total_files,
            total_time_secs: total_time.as_secs_f64(),
        };

        tracing::info!(
            "Batch conversion complete: {}/{} successful in {:.2}s",
            batch_result.successes.len(),
            total_files,
            batch_result.total_time_secs
        );

        Ok(batch_result)
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a single file — with OCR support
#[cfg(feature = "ocr")]
async fn convert_file(
    path: &Path,
    output_format: &OutputFormat,
    ocr_languages: &[OcrLanguage],
) -> Result<ConversionResult> {
    let converter = crate::converters::get_converter_with_ocr(path, ocr_languages)
        .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", path.display()))?;

    converter.convert(path, output_format).await
}

/// Convert a single file — without OCR support
#[cfg(not(feature = "ocr"))]
async fn convert_file(
    path: &Path,
    output_format: &OutputFormat,
) -> Result<ConversionResult> {
    let converter = crate::converters::get_converter(path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", path.display()))?;

    converter.convert(path, output_format).await
}
