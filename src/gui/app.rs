//! Main GUI application for MarkItDown-RST
//!
//! Features:
//! - Multilingual UI (EN/RU/ZH)
//! - Drag & drop file queue
//! - Multi-threaded batch conversion
//! - OCR settings (language selection) — requires `ocr` feature
//! - MD preview in system browser (highlight.js, KaTeX, Mermaid) — requires `preview` feature
//! - Dark/Light theme

use crate::batch::{BatchProcessor, FileEntry, FileStatus};
use crate::i18n::{I18n, Language};
#[cfg(feature = "ocr")]
use crate::ocr::{self, OcrLanguage};
#[cfg(feature = "preview")]
use crate::preview;
use crate::utils::{OutputFormat, detect_format, format_size};
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// Application state
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Idle,
    Converting,
    Completed,
    Error(String),
}

/// The main application
pub struct MarkItDownApp {
    /// Files in the conversion queue
    files: Vec<FileEntry>,
    /// Number of parallel jobs
    parallel_jobs: usize,
    /// Output directory
    output_dir: String,
    /// Save as combined file
    save_combined: bool,
    /// Current application state
    state: AppState,
    /// Progress: files completed
    progress_done: usize,
    /// Progress: total files
    progress_total: usize,
    /// Conversion results (markdown preview)
    preview_text: String,
    /// Last conversion results for preview
    last_results: Vec<(PathBuf, String)>, // (path, markdown)
    /// Selected preview file index
    selected_preview: usize,
    /// Tokio runtime for async operations
    runtime: Arc<Runtime>,
    /// Shared results from background thread
    shared_result: Arc<Mutex<Option<Result<crate::batch::BatchResult, String>>>>,
    /// Output format
    output_format: OutputFormat,
    /// Internationalization
    i18n: I18n,
    /// Dark theme
    dark_theme: bool,
    /// OCR: English language enabled
    #[cfg(feature = "ocr")]
    ocr_eng: bool,
    /// OCR: Russian language enabled
    #[cfg(feature = "ocr")]
    ocr_rus: bool,
    /// OCR: Simplified Chinese language enabled
    #[cfg(feature = "ocr")]
    ocr_chi_sim: bool,
    /// OCR: Tesseract availability
    #[cfg(feature = "ocr")]
    tesseract_available: bool,
}

impl MarkItDownApp {
    pub fn new() -> Self {
        let output_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir())
            .unwrap_or_default()
            .join("markitdown-output")
            .to_string_lossy()
            .to_string();

        #[cfg(feature = "ocr")]
        let tesseract_available = ocr::is_tesseract_available();

        // Extract tessdata on startup
        #[cfg(feature = "ocr")]
        if tesseract_available {
            let _ = ocr::ensure_tessdata(&[OcrLanguage::Eng, OcrLanguage::Rus, OcrLanguage::ChiSim]);
        }

        Self {
            files: Vec::new(),
            parallel_jobs: num_cpus::get(),
            output_dir,
            save_combined: false,
            state: AppState::Idle,
            progress_done: 0,
            progress_total: 0,
            preview_text: String::new(),
            last_results: Vec::new(),
            selected_preview: 0,
            runtime: Arc::new(Runtime::new().unwrap()),
            shared_result: Arc::new(Mutex::new(None)),
            output_format: OutputFormat::default(),
            i18n: I18n::default(),
            dark_theme: true,
            #[cfg(feature = "ocr")]
            ocr_eng: true,
            #[cfg(feature = "ocr")]
            ocr_rus: true,
            #[cfg(feature = "ocr")]
            ocr_chi_sim: true,
            #[cfg(feature = "ocr")]
            tesseract_available,
        }
    }

    /// Get selected OCR languages
    #[cfg(feature = "ocr")]
    fn selected_ocr_languages(&self) -> Vec<OcrLanguage> {
        let mut langs = Vec::new();
        if self.ocr_eng { langs.push(OcrLanguage::Eng); }
        if self.ocr_rus { langs.push(OcrLanguage::Rus); }
        if self.ocr_chi_sim { langs.push(OcrLanguage::ChiSim); }
        if langs.is_empty() { langs.push(OcrLanguage::Eng); }
        langs
    }

    /// Add files to the queue
    fn add_files(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if path.is_file() && crate::utils::is_supported(&path) {
                let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let format = detect_format(&path);
                if !self.files.iter().any(|f| f.path == path) {
                    self.files.push(FileEntry {
                        path,
                        status: FileStatus::Pending,
                        file_size,
                        format: format.to_string(),
                    });
                }
            } else if path.is_dir() {
                let collected = crate::utils::collect_files(&[path]);
                for file_path in collected {
                    if !self.files.iter().any(|f| f.path == file_path) {
                        let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                        let format = detect_format(&file_path);
                        self.files.push(FileEntry {
                            path: file_path,
                            status: FileStatus::Pending,
                            file_size,
                            format: format.to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Clear all files
    fn clear_files(&mut self) {
        self.files.clear();
        self.preview_text.clear();
        self.last_results.clear();
        self.selected_preview = 0;
    }

    /// Start the conversion
    fn start_conversion(&mut self) {
        if self.files.is_empty() {
            return;
        }

        self.state = AppState::Converting;
        self.progress_done = 0;
        self.progress_total = self.files.len();

        let paths: Vec<PathBuf> = self.files.iter().map(|f| f.path.clone()).collect();
        let output_format = self.output_format.clone();
        let parallel_jobs = self.parallel_jobs;
        let shared_result = self.shared_result.clone();
        #[cfg(feature = "ocr")]
        let ocr_languages = self.selected_ocr_languages();

        *shared_result.lock().unwrap() = None;

        let rt = self.runtime.clone();
        std::thread::spawn(move || {
            let result = rt.block_on(async {
                #[cfg(feature = "ocr")]
                let mut processor = BatchProcessor::new()
                    .add_paths(&paths)
                    .output_format(output_format)
                    .parallel(parallel_jobs);

                #[cfg(not(feature = "ocr"))]
                let processor = BatchProcessor::new()
                    .add_paths(&paths)
                    .output_format(output_format)
                    .parallel(parallel_jobs);

                // Set OCR languages
                #[cfg(feature = "ocr")]
                {
                    processor = processor.ocr_languages(ocr_languages);
                }

                processor.execute().await
            });

            *shared_result.lock().unwrap() = Some(result.map_err(|e| e.to_string()));
        });
    }

    /// Check for conversion completion
    fn check_result(&mut self) {
        let result = self.shared_result.lock().unwrap().take();
        if let Some(res) = result {
            match res {
                Ok(batch_result) => {
                    self.state = AppState::Completed;
                    self.progress_done = batch_result.successes.len();

                    // Store results for preview
                    self.last_results = batch_result.successes.iter()
                        .map(|(p, r)| (p.clone(), r.full_markdown()))
                        .collect();

                    if !self.last_results.is_empty() {
                        self.selected_preview = 0;
                        self.preview_text = self.last_results[0].1.clone();
                    }

                    // Build status preview text
                    let mut preview = String::new();
                    for (path, conversion) in &batch_result.successes {
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                        preview.push_str(&format!("## {} ({} {})\n\n", filename, conversion.metadata.word_count, self.i18n.word_count()));
                        let md = conversion.full_markdown();
                        if md.len() > 2000 {
                            preview.push_str(&md[..2000]);
                            preview.push_str("\n\n... (truncated)\n\n");
                        } else {
                            preview.push_str(&md);
                            preview.push_str("\n\n");
                        }
                    }

                    if !batch_result.failures.is_empty() {
                        preview.push_str(&format!("## {} {}\n\n", self.i18n.failed_files(), batch_result.failures.len()));
                        for (path, error) in &batch_result.failures {
                            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            preview.push_str(&format!("- **{}**: {}\n", filename, error));
                        }
                    }

                    preview.push_str(&format!(
                        "\n---\n{}/{} {} | {:.2}s | {} {}",
                        batch_result.successes.len(),
                        batch_result.total_files,
                        self.i18n.files_converted(),
                        batch_result.total_time_secs,
                        batch_result.total_word_count(),
                        self.i18n.total_words()
                    ));

                    self.preview_text = preview;

                    // Auto-save
                    if !self.output_dir.is_empty() {
                        let output_dir = PathBuf::from(&self.output_dir);
                        let rt = self.runtime.clone();
                        let _ = rt.block_on(batch_result.save_all(&output_dir, self.save_combined));
                    }
                }
                Err(e) => {
                    self.state = AppState::Error(e);
                }
            }
        }
    }
}

impl eframe::App for MarkItDownApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for conversion results
        if self.state == AppState::Converting {
            self.check_result();
            ctx.request_repaint();
        }

        // Apply theme
        if self.dark_theme {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Top panel with toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(self.i18n.app_title());
                ui.label(
                    egui::RichText::new(self.i18n.subtitle())
                        .small()
                        .color(egui::Color32::GRAY),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language selector
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(self.i18n.language_label()).small());
                        let lang = self.i18n.language();
                        egui::ComboBox::from_id_salt("lang_select")
                            .selected_text(format!("{}", lang))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.i18n, I18n::new(Language::En), "English");
                                ui.selectable_value(&mut self.i18n, I18n::new(Language::Ru), "Русский");
                                ui.selectable_value(&mut self.i18n, I18n::new(Language::Zh), "中文");
                            });
                    });

                    ui.add_space(8.0);

                    // Theme toggle
                    if ui.button(if self.dark_theme { self.i18n.light() } else { self.i18n.dark() }).clicked() {
                        self.dark_theme = !self.dark_theme;
                    }
                });
            });
            ui.add_space(2.0);
        });

        // Bottom panel with status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                match &self.state {
                    AppState::Idle => {
                        ui.label(
                            egui::RichText::new(self.i18n.ready_status())
                                .color(egui::Color32::LIGHT_GREEN),
                        );
                    }
                    AppState::Converting => {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {}/{}",
                                self.i18n.converting(),
                                self.progress_done, self.progress_total
                            ))
                            .color(egui::Color32::YELLOW),
                        );
                        let progress = if self.progress_total > 0 {
                            self.progress_done as f32 / self.progress_total as f32
                        } else {
                            0.0
                        };
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .show_percentage()
                                .desired_width(200.0),
                        );
                    }
                    AppState::Completed => {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} — {}/{} {}",
                                self.i18n.completed(),
                                self.progress_done, self.progress_total,
                                self.i18n.files_converted()
                            ))
                            .color(egui::Color32::GREEN),
                        );
                    }
                    AppState::Error(e) => {
                        ui.label(
                            egui::RichText::new(format!("{}: {}", self.i18n.error(), e))
                                .color(egui::Color32::RED),
                        );
                    }
                }

                // OCR status indicator
                #[cfg(feature = "ocr")]
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.tesseract_available {
                        ui.label(
                            egui::RichText::new("OCR: Tesseract OK")
                                .small()
                                .color(egui::Color32::GREEN),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(self.i18n.tesseract_not_found())
                                .small()
                                .color(egui::Color32::from_rgb(255, 165, 0)),
                        );
                    }
                });
            });
            ui.add_space(2.0);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Handle dropped files
            let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
            if !dropped_files.is_empty() {
                let paths: Vec<PathBuf> = dropped_files
                    .into_iter()
                    .filter_map(|f| f.path)
                    .collect();
                self.add_files(paths);
            }

            // Split into left (file list + controls) and right (preview)
            ui.horizontal_top(|ui| {
                // Left panel: File queue + controls
                ui.vertical(|ui| {
                    ui.set_min_width(420.0);
                    ui.set_max_width(520.0);

                    ui.heading(self.i18n.file_queue());
                    ui.add_space(4.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.add_files()).clicked() {
                            #[cfg(feature = "ocr")]
                            let filter_exts: &[&str] = &[
                                "pdf", "docx", "xlsx", "pptx", "html", "xml", "txt",
                                "csv", "tsv", "rtf", "odt", "md", "json", "zip",
                                "jpg", "jpeg", "png", "tiff", "tif", "bmp", "gif", "webp",
                            ];
                            #[cfg(not(feature = "ocr"))]
                            let filter_exts: &[&str] = &[
                                "pdf", "docx", "xlsx", "pptx", "html", "xml", "txt",
                                "csv", "tsv", "rtf", "odt", "md", "json", "zip",
                            ];

                            if let Some(paths) = rfd::FileDialog::new()
                                .add_filter("Documents", filter_exts)
                                .add_filter("All Files", &["*"])
                                .pick_files()
                            {
                                self.add_files(paths);
                            }
                        }

                        if ui.button(self.i18n.add_folder()).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.add_files(vec![path]);
                            }
                        }

                        if ui.button(self.i18n.clear()).clicked() {
                            self.clear_files();
                        }

                        ui.add_space(8.0);

                        let is_converting = self.state == AppState::Converting;
                        let can_start = !self.files.is_empty() && !is_converting;

                        ui.add_enabled_ui(can_start, |ui| {
                            let convert_button = ui.add_sized(
                                [120.0, 30.0],
                                egui::Button::new(
                                    egui::RichText::new(self.i18n.convert()).strong(),
                                ),
                            );
                            if convert_button.clicked() {
                                self.start_conversion();
                            }
                        });
                    });

                    ui.add_space(4.0);

                    // Settings row
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.i18n.threads()));
                        ui.add(egui::DragValue::new(&mut self.parallel_jobs).range(1..=64));
                        ui.add_space(8.0);
                        ui.checkbox(&mut self.save_combined, self.i18n.combined_output());
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.i18n.output()));
                        let _output_dir_response = ui.add_sized(
                            [240.0, 20.0],
                            egui::TextEdit::singleline(&mut self.output_dir),
                        );
                        if ui.button("...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.output_dir = path.to_string_lossy().to_string();
                            }
                        }
                    });

                    // OCR language settings
                    #[cfg(feature = "ocr")]
                    {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", self.i18n.ocr_languages()));
                            ui.checkbox(&mut self.ocr_eng, "ENG");
                            ui.checkbox(&mut self.ocr_rus, "RUS");
                            ui.checkbox(&mut self.ocr_chi_sim, "CHI");
                        });
                    }

                    ui.add_space(4.0);

                    // File list
                    egui::ScrollArea::vertical()
                        .max_height(350.0)
                        .show(ui, |ui| {
                            let mut to_remove: Vec<usize> = Vec::new();
                            for (i, entry) in self.files.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let status_icon = match &entry.status {
                                        FileStatus::Pending => "⏳",
                                        FileStatus::Converting => "🔄",
                                        FileStatus::Completed => "✅",
                                        FileStatus::Failed(_) => "❌",
                                        FileStatus::Skipped(_) => "⏭",
                                    };

                                    ui.label(status_icon);

                                    let filename = entry
                                        .path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("?");

                                    ui.label(
                                        egui::RichText::new(filename).size(12.0),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} | {}",
                                                    entry.format,
                                                    format_size(entry.file_size)
                                                ))
                                                .small()
                                                .color(egui::Color32::GRAY),
                                            );
                                            if ui.small_button("✕").clicked() {
                                                to_remove.push(i);
                                            }
                                        },
                                    );
                                });
                            }

                            for i in to_remove.into_iter().rev() {
                                self.files.remove(i);
                            }

                            if self.files.is_empty() {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(40.0);
                                    ui.label(
                                        egui::RichText::new(self.i18n.drop_files_here())
                                            .size(16.0)
                                            .color(egui::Color32::GRAY),
                                    );
                                });
                            }
                        });

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("{} {}", self.files.len(), self.i18n.files_in_queue()))
                            .small()
                            .color(egui::Color32::GRAY),
                    );

                    // Supported formats info
                    ui.add_space(8.0);
                    ui.collapsing(self.i18n.supported_formats(), |ui| {
                        #[cfg(feature = "ocr")]
                        let formats_text = "PDF DOCX XLSX PPTX HTML XML TXT CSV TSV RTF ODT JSON ZIP\n\
                             Images (OCR): JPG PNG TIFF BMP GIF WEBP";
                        #[cfg(not(feature = "ocr"))]
                        let formats_text = "PDF DOCX XLSX PPTX HTML XML TXT CSV TSV RTF ODT JSON ZIP";
                        ui.label(egui::RichText::new(formats_text).small().color(egui::Color32::GRAY));
                    });
                });

                ui.separator();

                // Right panel: Preview
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(self.i18n.preview());
                        ui.add_space(8.0);

                        // File selector for preview
                        if self.last_results.len() > 1 {
                            egui::ComboBox::from_id_salt("preview_file_select")
                                .selected_text(
                                    self.last_results.get(self.selected_preview)
                                        .map(|(p, _)| p.file_name().and_then(|n| n.to_str()).unwrap_or("?"))
                                        .unwrap_or("?")
                                )
                                .show_ui(ui, |ui| {
                                    for (i, (path, _)) in self.last_results.iter().enumerate() {
                                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                                        ui.selectable_value(&mut self.selected_preview, i, name);
                                    }
                                });

                            // Update preview text when selection changes
                            if let Some((_, md)) = self.last_results.get(self.selected_preview) {
                                self.preview_text = md.clone();
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !self.preview_text.is_empty() {
                                // Open in browser button (requires preview feature)
                                #[cfg(feature = "preview")]
                                if ui.button(self.i18n.open_in_browser()).clicked() {
                                    let title = self.last_results.get(self.selected_preview)
                                        .map(|(p, _)| p.file_name().and_then(|n| n.to_str()).unwrap_or("Preview"))
                                        .unwrap_or("Preview");
                                    let _ = preview::open_preview_in_browser(&self.preview_text, title, self.dark_theme);
                                }

                                // Save preview button
                                if ui.button(self.i18n.save()).clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Markdown", &["md"])
                                        .save_file()
                                    {
                                        let path = if path.extension().is_none() {
                                            path.with_extension("md")
                                        } else {
                                            path
                                        };
                                        let _ = std::fs::write(&path, &self.preview_text);
                                    }
                                }

                                // Copy to clipboard
                                if ui.button(self.i18n.copy()).clicked() {
                                    ctx.copy_text(self.preview_text.clone());
                                }
                            }
                        });
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.preview_text.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(60.0);
                                ui.label(
                                    egui::RichText::new(self.i18n.markdown_preview())
                                        .size(16.0)
                                        .color(egui::Color32::GRAY),
                                );
                            });
                        } else {
                            // Render markdown preview as monospace text
                            ui.label(
                                egui::RichText::new(&self.preview_text)
                                    .monospace()
                                    .size(12.0),
                            );
                        }
                    });
                });
            });
        });
    }
}

// Implement PartialEq for I18n to support the selectable_value widget
impl PartialEq for I18n {
    fn eq(&self, other: &Self) -> bool {
        self.language() == other.language()
    }
}
