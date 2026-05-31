//! Main GUI application for MDrust
//!
//! Professional dark theme with:
//! - TopBar with cached icon, title, theme toggle, language selector
//! - Sidebar (240px) with actions, settings, OCR checkboxes, Convert button
//! - Central panel with dropzone / file cards + preview
//! - File cards with format icons, status colors, progress bars
//! - Preview with Rendered (egui_commonmark) / Raw / Metadata tabs
//! - Status bar with ETA
//! - Zoom hotkeys (Ctrl+/Ctrl-/Ctrl+0)
//! - OCR defaults: eng=true, rus=false, chi_sim=false

use crate::batch::{BatchProcessor, FileEntry, FileStatus};
use crate::i18n::{I18n, Language};
#[cfg(feature = "ocr")]
use crate::ocr::{self, OcrLanguage, TesseractStatus};
#[cfg(feature = "preview")]
use crate::preview;
use crate::utils::{OutputFormat, detect_format, format_size};
use crate::gui::theme::Theme;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Application state
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Idle,
    Converting,
    Completed,
    Error(String),
}

/// Preview tab selection
#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviewTab {
    Rendered,
    Raw,
    Metadata,
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
    /// Conversion start time (for ETA)
    convert_start: Option<std::time::Instant>,
    /// Preview text (raw markdown)
    preview_text: String,
    /// Last conversion results for preview
    last_results: Vec<(PathBuf, String)>,
    /// Selected preview file index — tracked to avoid cloning every frame
    last_selected_preview: usize,
    /// Selected preview file index
    selected_preview: usize,
    /// Tokio runtime for async operations
    runtime: Arc<Runtime>,
    /// mpsc channel receiver for individual conversion results
    result_rx: std::sync::mpsc::Receiver<ConvertMessage>,
    /// mpsc channel sender for individual conversion results
    result_tx: std::sync::mpsc::Sender<ConvertMessage>,
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
    /// OCR: Tesseract availability status
    #[cfg(feature = "ocr")]
    tesseract_status: TesseractStatus,
    /// Current preview tab
    preview_tab: PreviewTab,
    /// Zoom level for preview
    zoom: f32,
    /// Notification message
    notification: Option<(String, std::time::Instant)>,
    /// Cached app icon texture — loaded once, not every frame
    app_icon: Option<egui::TextureHandle>,
    /// CommonMark cache for rendered preview
    md_cache: egui_commonmark::CommonMarkCache,
}

/// Messages sent from the conversion worker thread to the GUI
enum ConvertMessage {
    /// A single file was converted successfully
    FileDone {
        path: PathBuf,
        markdown: String,
    },
    /// A single file failed
    FileFailed {
        path: PathBuf,
        error: String,
    },
    /// All files have been processed (batch complete)
    BatchComplete,
    /// The entire batch failed with a fatal error
    BatchError(String),
}

impl MarkItDownApp {
    pub fn new() -> Self {
        let output_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir())
            .unwrap_or_default()
            .join("mdrust-output")
            .to_string_lossy()
            .to_string();

        #[cfg(feature = "ocr")]
        let tesseract_status = TesseractStatus::check();

        // Extract tessdata on startup (always, even if engine not installed,
        // so they're ready when the user installs Tesseract later)
        #[cfg(feature = "ocr")]
        {
            let _ = ocr::ensure_tessdata(&[OcrLanguage::Eng]);
        }

        let (result_tx, result_rx) = std::sync::mpsc::channel();

        Self {
            files: Vec::new(),
            parallel_jobs: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            output_dir,
            save_combined: false,
            state: AppState::Idle,
            progress_done: 0,
            progress_total: 0,
            convert_start: None,
            preview_text: String::new(),
            last_results: Vec::new(),
            last_selected_preview: 0,
            selected_preview: 0,
            runtime: Arc::new(
                Runtime::new().expect("MDrust: failed to create async runtime — this usually means the system cannot create threads")
            ),
            result_rx,
            result_tx,
            output_format: OutputFormat::default(),
            i18n: I18n::default(),
            dark_theme: true,
            #[cfg(feature = "ocr")]
            ocr_eng: true,
            #[cfg(feature = "ocr")]
            ocr_rus: false,
            #[cfg(feature = "ocr")]
            ocr_chi_sim: false,
            #[cfg(feature = "ocr")]
            tesseract_status,
            preview_tab: PreviewTab::Rendered,
            zoom: 1.0,
            notification: None,
            app_icon: None,
            md_cache: egui_commonmark::CommonMarkCache::default(),
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

    /// Pump mpsc channels for results — no blocking I/O on UI thread.
    /// Processes all pending messages (not just one) so the progress bar
    /// and preview update smoothly.
    fn pump_channels(&mut self) {
        while let Ok(msg) = self.result_rx.try_recv() {
            match msg {
                ConvertMessage::FileDone { path, markdown } => {
                    self.progress_done += 1;
                    // Store the result for preview
                    self.last_results.push((path.clone(), markdown));
                    // Auto-select the first result for preview
                    if self.last_results.len() == 1 {
                        self.selected_preview = 0;
                        self.last_selected_preview = 0;
                        if let Some((_, md)) = self.last_results.first() {
                            self.preview_text = md.clone();
                        }
                    }
                    // Update file status in queue
                    if let Some(file) = self.files.iter_mut().find(|f| f.path == path) {
                        file.status = FileStatus::Completed;
                    }
                }
                ConvertMessage::FileFailed { path, error } => {
                    self.progress_done += 1;
                    tracing::warn!("Failed to convert {}: {}", path.display(), error);
                    // Update file status in queue
                    if let Some(file) = self.files.iter_mut().find(|f| f.path == path) {
                        file.status = FileStatus::Failed(error);
                    }
                }
                ConvertMessage::BatchComplete => {
                    let success_count = self.last_results.len();
                    let total = self.progress_total;
                    if success_count > 0 {
                        self.state = AppState::Completed;
                        self.notification = Some((
                            format!("{}: {}/{}", self.i18n.completed(), success_count, total),
                            std::time::Instant::now(),
                        ));
                    } else {
                        self.state = AppState::Error(
                            self.i18n.error().to_string(),
                        );
                        self.notification = Some((
                            format!("{}: 0/{}", self.i18n.error(), total),
                            std::time::Instant::now(),
                        ));
                    }
                }
                ConvertMessage::BatchError(e) => {
                    self.state = AppState::Error(e.clone());
                    self.notification = Some((
                        format!("{}: {}", self.i18n.error(), e),
                        std::time::Instant::now(),
                    ));
                }
            }
        }
    }

    /// Add files to the queue
    fn add_files(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if path.is_file() && crate::utils::is_supported(&path) {
                let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let format = detect_format(&path);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                if !self.files.iter().any(|f| f.path == path) {
                    self.files.push(FileEntry {
                        path,
                        status: FileStatus::Pending,
                        file_size,
                        format: format.to_string(),
                        name,
                    });
                }
            } else if path.is_dir() {
                let collected = crate::utils::collect_files(&[path]);
                for file_path in collected {
                    if !self.files.iter().any(|f| f.path == file_path) {
                        let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);
                        let format = detect_format(&file_path);
                        let name = file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?")
                            .to_string();
                        self.files.push(FileEntry {
                            path: file_path,
                            status: FileStatus::Pending,
                            file_size,
                            format: format.to_string(),
                            name,
                        });
                    }
                }
            }
        }
    }

    /// Start the conversion — results are streamed back via mpsc channel
    fn start_conversion(&mut self) {
        if self.files.is_empty() {
            return;
        }

        self.state = AppState::Converting;
        self.progress_done = 0;
        self.progress_total = self.files.len();
        self.convert_start = Some(std::time::Instant::now());
        self.last_results.clear();
        self.selected_preview = 0;
        self.last_selected_preview = 0;

        let paths: Vec<PathBuf> = self.files.iter().map(|f| f.path.clone()).collect();
        let output_format = self.output_format.clone();
        let parallel_jobs = self.parallel_jobs;
        let tx = self.result_tx.clone();
        let output_dir = PathBuf::from(&self.output_dir);
        let save_combined = self.save_combined;
        #[cfg(feature = "ocr")]
        let ocr_languages = self.selected_ocr_languages();

        let rt = self.runtime.clone();
        std::thread::spawn(move || {
            let result = rt.block_on(async {
                #[cfg(feature = "ocr")]
                let processor = BatchProcessor::new()
                    .add_paths(&paths)
                    .output_format(output_format)
                    .parallel(parallel_jobs)
                    .ocr_languages(ocr_languages);

                #[cfg(not(feature = "ocr"))]
                let processor = BatchProcessor::new()
                    .add_paths(&paths)
                    .output_format(output_format)
                    .parallel(parallel_jobs);

                let batch_result = processor.execute().await?;

                // Stream individual results back to GUI
                for (path, conversion) in &batch_result.successes {
                    let markdown = conversion.full_markdown();
                    let _ = tx.send(ConvertMessage::FileDone {
                        path: path.clone(),
                        markdown,
                    });
                }
                for (path, error) in &batch_result.failures {
                    let _ = tx.send(ConvertMessage::FileFailed {
                        path: path.clone(),
                        error: error.clone(),
                    });
                }

                // Auto-save on the worker thread — does NOT block UI
                if !output_dir.as_os_str().is_empty() && !batch_result.successes.is_empty() {
                    let _ = batch_result.save_all(&output_dir, save_combined).await;
                }

                let _ = tx.send(ConvertMessage::BatchComplete);

                Ok::<(), anyhow::Error>(())
            });

            if let Err(e) = result {
                let _ = tx.send(ConvertMessage::BatchError(e.to_string()));
            }
        });
    }

    // ── Drawing helpers ────────────────────────────────────────────────

    fn draw_topbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(8.0);
        ui.horizontal_centered(|ui| {
            // App icon — cached in struct, loaded once (not every frame)
            let icon = self.app_icon.get_or_insert_with(|| {
                let default_tex = || ctx.load_texture(
                    "app-icon",
                    egui::ColorImage::from_rgba_unmultiplied([1, 1], &[255, 255, 255, 0]),
                    egui::TextureOptions::LINEAR,
                );
                match image::load_from_memory(include_bytes!("../../assets/icon-256.png")) {
                    Ok(img) => {
                        let img = img.resize(28, 28, image::imageops::FilterType::Lanczos3);
                        let rgba = img.to_rgba8();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            &rgba,
                        );
                        ctx.load_texture("app-icon", color_image, egui::TextureOptions::LINEAR)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load app icon: {e}");
                        default_tex()
                    }
                }
            });
            ui.image(&*icon);
            ui.add_space(6.0);
            ui.label(egui::RichText::new("MDrust").size(16.0).strong());
            ui.label(egui::RichText::new("\u{00b7} Document to Markdown")
                .small().color(Theme::TEXT_DIM));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Language selector
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(self.i18n.lang_label())
                    .show_ui(ui, |ui| {
                        let i18n_en = I18n::new(Language::En);
                        let i18n_ru = I18n::new(Language::Ru);
                        let i18n_zh = I18n::new(Language::Zh);
                        ui.selectable_value(&mut self.i18n, i18n_en, "English");
                        ui.selectable_value(&mut self.i18n, i18n_ru, "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}");
                        ui.selectable_value(&mut self.i18n, i18n_zh, "\u{4e2d}\u{6587}");
                    });

                ui.add_space(8.0);

                // Theme toggle
                let theme_label = if self.dark_theme { "\u{2600}" } else { "\u{263d}" }; // ☀ / ☽
                if ui.button(theme_label).clicked() {
                    self.dark_theme = !self.dark_theme;
                    Theme::apply(ctx, self.dark_theme);
                }
            });
        });
    }

    fn draw_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            match &self.state {
                AppState::Idle => {
                    ui.label(egui::RichText::new(self.i18n.ready_status())
                        .small().color(Theme::SUCCESS));
                }
                AppState::Converting => {
                    ui.spinner();
                    ui.label(egui::RichText::new(format!(
                        "{} {}/{}",
                        self.i18n.converting(),
                        self.progress_done, self.progress_total
                    )).small().color(Theme::WARNING));

                    if let Some(start) = self.convert_start {
                        let elapsed = start.elapsed().as_secs_f64();
                        if self.progress_done > 0 && self.progress_total > 0 {
                            let rate = self.progress_done as f64 / elapsed;
                            let remaining = (self.progress_total - self.progress_done) as f64 / rate;
                            ui.label(egui::RichText::new(format!("ETA: {:.0}s", remaining))
                                .small().color(Theme::TEXT_DIM));
                        }
                    }
                }
                AppState::Completed => {
                    let success_count = self.last_results.len();
                    ui.label(egui::RichText::new(format!(
                        "{} \u{2014} {}/{} {}",
                        self.i18n.completed(),
                        success_count, self.progress_total,
                        self.i18n.files_converted()
                    )).small().color(if success_count > 0 { Theme::SUCCESS } else { Theme::WARNING }));
                }
                AppState::Error(e) => {
                    ui.label(egui::RichText::new(format!("{}: {}", self.i18n.error(), e))
                        .small().color(Theme::ERROR));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // SIMD level indicator
                let cpu = crate::cpu::features();
                let simd_label = cpu.simd_label();
                let simd_color = match cpu.simd_level() {
                    crate::cpu::SimdLevel::Avx512 => Theme::SUCCESS,
                    crate::cpu::SimdLevel::Avx2 => Theme::SUCCESS,
                    crate::cpu::SimdLevel::Avx => egui::Color32::from_rgb(100, 200, 100),
                    crate::cpu::SimdLevel::Sse42 | crate::cpu::SimdLevel::Sse41 => Theme::WARNING,
                    crate::cpu::SimdLevel::Neon => Theme::SUCCESS,
                    crate::cpu::SimdLevel::None => Theme::ERROR,
                };
                ui.label(egui::RichText::new(format!("SIMD: {}", simd_label))
                    .small().color(simd_color));

                ui.separator();

                #[cfg(feature = "ocr")]
                {
                    let (label, color) = match self.tesseract_status {
                        TesseractStatus::Available => (self.tesseract_status.status_label(), Theme::SUCCESS),
                        TesseractStatus::NotInstalled => (self.tesseract_status.status_label(), Theme::WARNING),
                    };
                    let response = ui.label(egui::RichText::new(label).small().color(color));
                    response.on_hover_text(self.tesseract_status.tooltip());
                }

                ui.label(egui::RichText::new(format!("{} files", self.files.len()))
                    .small().color(Theme::TEXT_DIM));
            });
        });

        // Notification
        if let Some((msg, time)) = &self.notification {
            if time.elapsed().as_secs() < 3 {
                ui.colored_label(Theme::SUCCESS, egui::RichText::new(format!("\u{2705} {}", msg)).small());
            }
        }
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        // Actions section
        ui.label(egui::RichText::new(self.i18n.actions())
            .small().color(Theme::TEXT_DIM));
        ui.add_space(4.0);

        if self.sidebar_button(ui, "\u{2795}", &self.i18n.add_files()).clicked() { // ➕
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

        if self.sidebar_button(ui, "\u{1f4c2}", &self.i18n.add_folder()).clicked() { // 📂
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                self.add_files(vec![p]);
            }
        }

        if self.sidebar_button(ui, "\u{1f5d1}", &self.i18n.clear()).clicked() { // 🗑
            self.files.clear();
            self.preview_text.clear();
            self.last_results.clear();
            self.selected_preview = 0;
            self.last_selected_preview = 0;
            self.state = AppState::Idle;
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        // Settings section
        ui.label(egui::RichText::new(self.i18n.settings())
            .small().color(Theme::TEXT_DIM));
        ui.add_space(8.0);

        // Output format selector
        ui.label(egui::RichText::new(self.i18n.output_format())
            .small().color(Theme::TEXT_DIM));
        let format_label = format!("{}", self.output_format);
        egui::ComboBox::from_id_salt("output_format")
            .selected_text(&format_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.output_format,
                    OutputFormat::Markdown { split_pages: false, optimize_for_llm: true },
                    "Markdown (.md)",
                );
                ui.selectable_value(
                    &mut self.output_format,
                    OutputFormat::Html { standalone: true, include_css: true },
                    "HTML (.html)",
                );
                ui.selectable_value(
                    &mut self.output_format,
                    OutputFormat::Docx,
                    "Word (.docx)",
                );
            });

        ui.add(egui::Slider::new(&mut self.parallel_jobs, 1..=64)
            .text(self.i18n.threads()));
        ui.checkbox(&mut self.save_combined, self.i18n.combined_output());

        ui.add_space(8.0);
        ui.label(self.i18n.output_dir());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.output_dir)
                .desired_width(160.0));
            if ui.button("...").clicked() {
                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                    self.output_dir = p.to_string_lossy().to_string();
                }
            }
        });

        // OCR settings
        #[cfg(feature = "ocr")]
        {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("OCR").small().color(Theme::TEXT_DIM));
            ui.checkbox(&mut self.ocr_eng, "English");
            ui.checkbox(&mut self.ocr_rus, "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}");
            ui.checkbox(&mut self.ocr_chi_sim, "\u{4e2d}\u{6587}");
        }

        // Primary action — Convert button at bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            let enabled = !self.files.is_empty() && !matches!(self.state, AppState::Converting);
            let label = format!("\u{25b6}  {}", self.i18n.convert()); // ▶
            let btn = egui::Button::new(
                egui::RichText::new(label).color(egui::Color32::WHITE).strong())
                .fill(if enabled { Theme::ACCENT } else { Theme::SURFACE_2 })
                .min_size(egui::vec2(ui.available_width(), 40.0))
                .rounding(8.0);
            if ui.add_enabled(enabled, btn).clicked() {
                self.start_conversion();
            }
        });
    }

    fn sidebar_button(&self, ui: &mut egui::Ui, ic: &str, label: &str) -> egui::Response {
        ui.add(egui::Button::new(format!("{}  {}", ic, label))
            .min_size(egui::vec2(ui.available_width(), 32.0))
            .rounding(6.0))
    }

    fn draw_main(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.files.is_empty() {
            self.draw_dropzone(ui, ctx);
            return;
        }

        // File list + preview split
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label(egui::RichText::new(self.i18n.file_queue()).heading());
                ui.label(egui::RichText::new(format!("{} files", self.files.len()))
                    .small().color(Theme::TEXT_DIM));
                ui.add_space(8.0);

                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    let mut to_remove: Option<usize> = None;
                    for (idx, file) in self.files.iter().enumerate() {
                        if self.draw_file_card(ui, file) {
                            to_remove = Some(idx);
                        }
                    }
                    if let Some(idx) = to_remove {
                        self.files.remove(idx);
                    }
                });
            });

            cols[1].vertical(|ui| {
                self.draw_preview(ui, ctx);
            });
        });
    }

    fn draw_dropzone(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
        let rect = ui.available_rect_before_wrap().shrink(24.0);
        let stroke = if hovering {
            egui::Stroke::new(2.5, Theme::ACCENT)
        } else {
            egui::Stroke::new(1.5, Theme::BORDER)
        };
        let fill = if hovering {
            Theme::ACCENT.gamma_multiply(0.08)
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, egui::Rounding::same(16), fill);
        ui.painter().rect_stroke(rect, egui::Rounding::same(16), stroke, egui::StrokeKind::Outside);

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(rect.height() * 0.30);
                ui.label(egui::RichText::new("\u{1f4e4}") // 📤
                    .size(56.0).color(Theme::TEXT_DIM));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(self.i18n.drop_files_here())
                    .size(18.0).color(Theme::TEXT));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(self.i18n.or_click_add())
                    .size(12.0).color(Theme::TEXT_DIM));
            });
        });
    }

    fn draw_file_card(&self, ui: &mut egui::Ui, file: &FileEntry) -> bool {
        let mut remove = false;
        let format: crate::utils::InputFormat = crate::utils::detect_format(&file.path);
        let status_color = match &file.status {
            FileStatus::Pending => Theme::TEXT_DIM,
            FileStatus::Converting(_) => Theme::WARNING,
            FileStatus::Completed => Theme::SUCCESS,
            FileStatus::Failed(_) => Theme::ERROR,
            FileStatus::Skipped(_) => Theme::TEXT_DIM,
        };

        egui::Frame::none()
            .fill(if self.dark_theme { Theme::SURFACE } else { Theme::LIGHT_SURFACE })
            .stroke(egui::Stroke::new(1.0, if self.dark_theme { Theme::BORDER } else { Theme::LIGHT_BORDER }))
            .rounding(8.0)
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format_icon(&format)).size(20.0));
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&file.name).strong());
                        ui.label(egui::RichText::new(format!(
                            "{} \u{00b7} {} \u{00b7} {}",
                            file.format,
                            format_size(file.file_size),
                            file.status.label()
                        )).small().color(status_color));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("\u{2715}").clicked() { remove = true; } // ✕
                    });
                });
                if let FileStatus::Converting(p) = file.status {
                    ui.add_space(4.0);
                    ui.add(egui::ProgressBar::new(p)
                        .desired_height(4.0)
                        .fill(Theme::ACCENT));
                }
            });
        ui.add_space(6.0);
        remove
    }

    fn draw_preview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(self.i18n.preview()).heading());

            if self.last_results.len() > 1 {
                ui.add_space(8.0);
                let prev_selected = self.selected_preview;
                egui::ComboBox::from_id_salt("preview_select")
                    .selected_text(
                        self.last_results.get(self.selected_preview)
                            .map(|(p, _)| p.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                            .unwrap_or("")
                    )
                    .show_ui(ui, |ui| {
                        for (i, (path, _)) in self.last_results.iter().enumerate() {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            ui.selectable_value(&mut self.selected_preview, i, name);
                        }
                    });

                // Only clone when selection actually changes
                if self.selected_preview != prev_selected {
                    if let Some((_, md)) = self.last_results.get(self.selected_preview) {
                        self.preview_text = md.clone();
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.preview_text.is_empty() {
                    // Copy button
                    if ui.button("\u{1f4cb}  Copy").clicked() { // 📋
                        ui.output_mut(|o| o.copied_text = self.preview_text.clone());
                    }

                    // Save button
                    if ui.button("\u{1f4be}  Save").clicked() { // 💾
                        self.save_current_preview();
                    }

                    // Open in browser
                    #[cfg(feature = "preview")]
                    if ui.button("\u{1f310}  Browser").clicked() { // 🌐
                        let title = self.last_results.get(self.selected_preview)
                            .map(|(p, _)| p.file_name().and_then(|n| n.to_str()).unwrap_or("Preview"))
                            .unwrap_or("Preview");
                        let _ = preview::open_preview_in_browser(&self.preview_text, title, self.dark_theme);
                    }
                }
            });
        });

        ui.add_space(4.0);

        // Preview tabs
        ui.horizontal(|ui| {
            let rendered_label = format!("\u{1f441}  {}", self.i18n.rendered()); // 👁
            let raw_label = format!("\u{1f4bb}  {}", self.i18n.raw_md()); // 💻
            let meta_label = format!("\u{2139}  {}", self.i18n.metadata()); // ℹ

            ui.selectable_value(&mut self.preview_tab, PreviewTab::Rendered, rendered_label);
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Raw, raw_label);
            ui.selectable_value(&mut self.preview_tab, PreviewTab::Metadata, meta_label);

            // Zoom controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(format!("{:.0}%", self.zoom * 100.0))
                    .small().color(Theme::TEXT_DIM));
                if ui.small_button("-").clicked() {
                    self.zoom = (self.zoom - 0.1).max(0.3);
                }
                if ui.small_button("+").clicked() {
                    self.zoom = (self.zoom + 0.1).min(3.0);
                }
            });
        });

        ui.add_space(4.0);

        // Preview content
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.preview_text.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.label(egui::RichText::new(self.i18n.markdown_preview())
                        .size(16.0).color(Theme::TEXT_DIM));
                });
            } else {
                match self.preview_tab {
                    PreviewTab::Rendered => {
                        // Rendered markdown using egui_commonmark — real rendering in egui
                        let viewer = egui_commonmark::CommonMarkViewer::new();
                        viewer.show(ui, &mut self.md_cache, &self.preview_text);
                    }
                    PreviewTab::Raw => {
                        ui.label(egui::RichText::new(&self.preview_text)
                            .monospace()
                            .size(12.0 * self.zoom));
                    }
                    PreviewTab::Metadata => {
                        self.draw_metadata_tab(ui);
                    }
                }
            }
        });
    }

    fn draw_metadata_tab(&self, ui: &mut egui::Ui) {
        if let Some((path, _)) = self.last_results.get(self.selected_preview) {
            egui::Grid::new("metadata_grid")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 8.0))
                .show(ui, |ui| {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    ui.strong("File:");
                    ui.label(filename);
                    ui.end_row();

                    ui.strong("Path:");
                    ui.label(path.display().to_string());
                    ui.end_row();

                    let size = std::fs::metadata(path).map(|m| format_size(m.len())).unwrap_or_default();
                    ui.strong("Size:");
                    ui.label(size);
                    ui.end_row();

                    let fmt = detect_format(path);
                    ui.strong("Format:");
                    ui.label(fmt.to_string());
                    ui.end_row();

                    let words: usize = self.preview_text.split_whitespace().count();
                    ui.strong("Words:");
                    ui.label(words.to_string());
                    ui.end_row();

                    let lines = self.preview_text.lines().count();
                    ui.strong("Lines:");
                    ui.label(lines.to_string());
                    ui.end_row();
                });
        }
    }

    fn save_current_preview(&self) {
        let ext = self.output_format.extension();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Output", &[ext])
            .add_filter("All Files", &["*"])
            .save_file()
        {
            let path = if path.extension().is_none() {
                path.with_extension(ext)
            } else {
                path
            };
            match self.output_format {
                OutputFormat::Html { .. } => {
                    if let Ok(html) = crate::export::md_to_html::markdown_to_html(
                        &self.preview_text,
                        &self.output_format,
                    ) {
                        let _ = std::fs::write(&path, html);
                    }
                }
                OutputFormat::Docx => {
                    let title = self.last_results.get(self.selected_preview)
                        .and_then(|(p, _)| p.file_stem())
                        .and_then(|s| s.to_str());
                    if let Ok(docx_bytes) = crate::export::md_to_docx::markdown_to_docx(
                        &self.preview_text,
                        title,
                    ) {
                        let _ = std::fs::write(&path, docx_bytes);
                    }
                }
                _ => {
                    let _ = std::fs::write(&path, &self.preview_text);
                }
            }
        }
    }
}

/// Map file format to Unicode icon
fn format_icon(format: &crate::utils::InputFormat) -> &'static str {
    use crate::utils::InputFormat::*;
    match format {
        Pdf => "\u{1f4d3}",   // 📓
        Docx => "\u{1f4c4}",  // 📄
        Xlsx => "\u{1f4ca}",  // 📊
        Pptx => "\u{1f4ac}",  // 💬
        Html => "\u{1f310}",  // 🌐
        Xml => "\u{1f4cb}",   // 📋
        Image => "\u{1f5bc}", // 🖼
        Zip => "\u{1f4e6}",   // 📦
        Json => "\u{1f4bd}",  // 💽
        Csv | Tsv => "\u{1f4c9}", // 📉
        _ => "\u{1f4c4}",     // 📄
    }
}

impl eframe::App for MarkItDownApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.pump_channels();

        // === Zoom hotkeys ===
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    self.zoom = (self.zoom + 0.1).min(3.0);
                }
                if i.key_pressed(egui::Key::Minus) {
                    self.zoom = (self.zoom - 0.1).max(0.3);
                }
                if i.key_pressed(egui::Key::Num0) {
                    self.zoom = 1.0;
                }
            }
        });

        // === TOP BAR ===
        egui::TopBottomPanel::top("topbar")
            .exact_height(48.0)
            .show(ctx, |ui| {
                self.draw_topbar(ui, ctx);
            });

        // === STATUS BAR ===
        egui::TopBottomPanel::bottom("status")
            .exact_height(32.0)
            .show(ctx, |ui| {
                self.draw_status_bar(ui);
            });

        // === SIDEBAR ===
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(240.0)
            .frame(egui::Frame::side_top_panel(&ctx.style())
                .fill(if self.dark_theme { Theme::SURFACE } else { Theme::LIGHT_SURFACE })
                .inner_margin(egui::Margin::same(16)))
            .show(ctx, |ui| {
                self.draw_sidebar(ui);
            });

        // === CENTRAL ===
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style())
                .inner_margin(egui::Margin::same(16)))
            .show(ctx, |ui| {
                self.draw_main(ui, ctx);
            });

        // Handle drag-and-drop globally
        let dropped: Vec<_> = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let paths: Vec<_> = dropped.into_iter().filter_map(|f| f.path).collect();
            self.add_files(paths);
        }

        // Repaint while converting
        if matches!(self.state, AppState::Converting) {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
