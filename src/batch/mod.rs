//! Multi-threaded batch processor for document conversion
//!
//! Uses tokio for async I/O and a Semaphore for concurrency control,
//! similar to the transmutation project's architecture.

use crate::converters::ConversionResult;
use crate::utils::{OutputFormat, collect_files};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// OCR language type — only available with the `ocr` feature
#[cfg(feature = "ocr")]
use crate::ocr::OcrLanguage;

/// Status of a single file in the batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Pending,
    Converting,
    Completed,
    Failed(String),
    Skipped(String),
}

/// Information about a file in the batch queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub status: FileStatus,
    pub file_size: u64,
    pub format: String,
}

/// Result of a batch conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
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

        if combined {
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

            let output_path = output_dir.join("combined_output.md");
            tokio::fs::write(&output_path, combined_markdown).await?;
        } else {
            for (input_path, result) in &self.successes {
                let filename = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let output_path = output_dir.join(format!("{}.md", filename));
                result.save_to_file(&output_path).await?;
            }
        }

        Ok(())
    }
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(usize, usize, &PathBuf) + Send + Sync>;

/// Multi-threaded batch processor
pub struct BatchProcessor {
    files: Vec<FileEntry>,
    output_format: OutputFormat,
    parallel_jobs: usize,
    progress_callback: Option<ProgressCallback>,
    #[cfg(feature = "ocr")]
    ocr_languages: Vec<OcrLanguage>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            output_format: OutputFormat::default(),
            parallel_jobs: num_cpus::get(),
            progress_callback: None,
            #[cfg(feature = "ocr")]
            ocr_languages: vec![OcrLanguage::Eng, OcrLanguage::Rus, OcrLanguage::ChiSim],
        }
    }

    /// Add files from paths (supports files and directories)
    pub fn add_paths(mut self, paths: &[PathBuf]) -> Self {
        let collected = collect_files(paths);
        for path in collected {
            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let format = crate::utils::detect_format(&path);
            self.files.push(FileEntry {
                path,
                status: FileStatus::Pending,
                file_size,
                format: format.to_string(),
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

    /// Set progress callback
    pub fn on_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
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
        let mut tasks = Vec::new();

        for file_path in file_paths {
            let sem = semaphore.clone();
            let counter = completed.clone();
            let out_fmt = output_format.clone();

            #[cfg(feature = "ocr")]
            let ocr_langs = self.ocr_languages.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                #[cfg(feature = "ocr")]
                let result = convert_file(&file_path, &out_fmt, &ocr_langs).await;

                #[cfg(not(feature = "ocr"))]
                let result = convert_file(&file_path, &out_fmt).await;

                let done = counter.fetch_add(1, Ordering::Relaxed) + 1;
                (file_path, result, done)
            });

            tasks.push(task);
        }

        let results = futures::future::join_all(tasks).await;

        let total_time = start_time.elapsed();
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for task_result in results {
            match task_result {
                Ok((path, Ok(conversion), _done)) => {
                    successes.push((path, conversion));
                }
                Ok((path, Err(e), _done)) => {
                    tracing::warn!("Failed to convert {}: {}", path.display(), e);
                    failures.push((path, e.to_string()));
                }
                Err(join_error) => {
                    tracing::error!("Task join error: {}", join_error);
                }
            }
        }

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
