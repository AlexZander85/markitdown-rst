//! MDrust CLI — Multi-threaded document-to-markdown converter
//!
//! Light build: core document conversion only (no OCR, no preview)
//! Full build: adds Tesseract OCR for images

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use mdrust::batch::BatchProcessor;
use mdrust::utils::{OutputFormat, collect_files, detect_format, format_size};

#[cfg(feature = "ocr")]
use mdrust::ocr::{self, OcrLanguage};

#[derive(Parser)]
#[command(
    name = "mdrust",
    version,
    about = "Multi-threaded document-to-markdown converter",
    long_about = "MDrust converts documents to Markdown format with multi-threaded\n\
                  batch processing. Full build adds OCR for images via Tesseract.\n\n\
                  Inspired by markitdown-gui, transmutation, and mdhero projects."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a single document
    Convert {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short = 'l', long)]
        optimize_llm: bool,
        #[arg(short = 'f', long, default_value = "md", value_name = "FORMAT")]
        /// Output format: md, html, docx
        format: String,
        #[cfg(feature = "ocr")]
        #[arg(short = 'L', long, value_delimiter = ',', default_value = "eng")]
        ocr_langs: Vec<String>,
    },

    /// Batch convert multiple documents
    Batch {
        #[arg(value_name = "INPUT")]
        input: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short = 'j', long)]
        jobs: Option<usize>,
        #[arg(short = 'C', long)]
        combined: bool,
        #[arg(short, long)]
        continue_on_error: bool,
        #[arg(short = 'f', long, default_value = "md", value_name = "FORMAT")]
        /// Output format: md, html, docx
        format: String,
        #[cfg(feature = "ocr")]
        #[arg(short = 'L', long, value_delimiter = ',', default_value = "eng")]
        ocr_langs: Vec<String>,
    },

    /// Show document information
    Info {
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// List supported formats
    Formats,

    #[cfg(feature = "ocr")]
    /// Check OCR (Tesseract) availability
    OcrCheck,
}

/// Parse output format string
fn parse_output_format(fmt: &str) -> OutputFormat {
    match fmt.to_lowercase().as_str() {
        "html" | "htm" => OutputFormat::Html { standalone: true, include_css: true },
        "docx" | "doc" | "word" => OutputFormat::Docx,
        "json" => OutputFormat::Json { structured: true, include_metadata: true },
        _ => OutputFormat::Markdown { split_pages: false, optimize_for_llm: true },
    }
}

#[cfg(feature = "ocr")]
fn parse_ocr_langs(lang_strs: &[String]) -> Vec<OcrLanguage> {
    let langs: Vec<OcrLanguage> = lang_strs.iter().filter_map(|s| match s.as_str() {
        "eng" => Some(OcrLanguage::Eng),
        "rus" => Some(OcrLanguage::Rus),
        "chi_sim" | "chi" => Some(OcrLanguage::ChiSim),
        _ => None,
    }).collect();
    if langs.is_empty() { vec![OcrLanguage::Eng] } else { langs }
}

fn default_parallel_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let log_level = if cli.verbose { tracing::Level::DEBUG }
        else if cli.quiet { tracing::Level::ERROR }
        else { tracing::Level::INFO };

    #[cfg(feature = "logs")]
    tracing_subscriber::fmt().with_max_level(log_level).with_target(false).init();

    #[cfg(not(feature = "logs"))]
    {
        let _ = log_level; // suppress unused variable warning
    }

    if let Err(e) = run_command(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        // ── Convert ──────────────────────────────────────────────
        #[cfg(feature = "ocr")]
        Commands::Convert { input, output, optimize_llm, format, ocr_langs } => {
            let langs = parse_ocr_langs(&ocr_langs);
            let out_fmt = parse_output_format(&format);
            do_convert(&input, &output, optimize_llm, cli.quiet, &langs, &out_fmt).await
        }
        #[cfg(not(feature = "ocr"))]
        Commands::Convert { input, output, optimize_llm, format } => {
            let out_fmt = parse_output_format(&format);
            do_convert(&input, &output, optimize_llm, cli.quiet, &out_fmt).await
        }

        // ── Batch ────────────────────────────────────────────────
        #[cfg(feature = "ocr")]
        Commands::Batch { input, output, jobs, combined, continue_on_error: _, format, ocr_langs } => {
            let langs = parse_ocr_langs(&ocr_langs);
            let out_fmt = parse_output_format(&format);
            do_batch(&input, &output, jobs, combined, cli.quiet, &langs, &out_fmt).await
        }
        #[cfg(not(feature = "ocr"))]
        Commands::Batch { input, output, jobs, combined, continue_on_error: _, format } => {
            let out_fmt = parse_output_format(&format);
            do_batch(&input, &output, jobs, combined, cli.quiet, &out_fmt).await
        }

        // ── Info ─────────────────────────────────────────────────
        Commands::Info { input } => {
            if !input.exists() { anyhow::bail!("File not found: {}", input.display()); }
            let format = detect_format(&input);
            let metadata = std::fs::metadata(&input)?;
            println!("{}", "Document Information".cyan().bold());
            println!("  Path:   {}", input.display());
            println!("  Format: {}", format);
            println!("  Size:   {}", format_size(metadata.len()));
            println!("  Status: {}", format.status());
            println!("  Output: {}", format.output_options());
            Ok(())
        }

        // ── Formats ──────────────────────────────────────────────
        Commands::Formats => {
            println!("{}", "Supported Formats".cyan().bold());
            println!();
            let formats = [
                ("PDF","Image per page, Markdown (per page/full), JSON","Production"),
                ("DOCX","Image per page, Markdown (per page/full), JSON","Production"),
                ("XLSX","Markdown tables, CSV, JSON","Production"),
                ("PPTX","Image per slide, Markdown per slide","Production"),
                ("HTML","Markdown, JSON","Production"),
                ("XML","Markdown, JSON","Production"),
                ("TXT","Markdown, JSON","Production"),
                ("CSV","Markdown tables, JSON","Production"),
                ("TSV","Markdown tables, JSON","Production"),
                ("JSON","Pretty-printed code blocks, tabular view","Production"),
                ("ZIP","File listing, statistics, content extraction","Production"),
                ("RTF","Markdown, JSON (simplified)","Beta"),
                ("ODT","Markdown, JSON (ZIP + XML)","Beta"),
            ];
            println!("{:<8} {:<55} {}", "Format", "Description", "Status");
            println!("{}", "-".repeat(75));
            for (fmt, desc, status) in &formats {
                let s = if *status == "Production" { status.green() } else { status.yellow() };
                println!("{:<8} {:<55} {}", fmt.green(), desc, s);
            }
            #[cfg(feature = "ocr")]
            {
                println!();
                println!("{:<8} {:<55} {}", "Image".green(), "OCR via Tesseract (JPG/PNG/TIFF/BMP/GIF/WEBP)", "Production".green());
                println!();
                println!("{}", "OCR Engine: Tesseract".cyan().bold());
                println!("  Languages: English, Russian, Simplified Chinese (embedded)");
                println!("  Available: {}", if ocr::is_tesseract_available() { "Yes".green() } else { "No".red() });
            }
            #[cfg(not(feature = "ocr"))]
            {
                println!();
                println!("{}", "Note: OCR not available in this light build.".yellow());
                println!("  Install the full version for image OCR support.");
            }
            Ok(())
        }

        // ── OCR Check ────────────────────────────────────────────
        #[cfg(feature = "ocr")]
        Commands::OcrCheck => {
            println!("{}", "OCR (Tesseract) Check".cyan().bold());
            println!();
            if ocr::is_tesseract_available() {
                println!("  Tesseract: {}", "Installed".green().bold());
            } else {
                println!("  Tesseract: {}", "NOT FOUND".red().bold());
                println!();
                println!("  Install:");
                println!("    Debian/Ubuntu: sudo apt install tesseract-ocr");
                println!("    Fedora:        sudo dnf install tesseract");
                println!("    macOS:         brew install tesseract");
                println!("    Windows:       choco install tesseract");
            }
            println!();
            println!("  Embedded tessdata (tessdata_fast):");
            let all_langs = [OcrLanguage::Eng, OcrLanguage::Rus, OcrLanguage::ChiSim];
            match ocr::ensure_tessdata(&all_langs) {
                Ok(()) => {
                    for lang in &all_langs {
                        let path = mdrust::utils::tessdata_dir().join(lang.filename());
                        let size = std::fs::metadata(&path).map(|m| format_size(m.len())).unwrap_or_else(|_| "N/A".into());
                        println!("    {} ({}) - {}", lang, lang.tesseract_code(), size);
                    }
                }
                Err(e) => println!("    Error: {}", e),
            }
            Ok(())
        }
    }
}

/// Single-file conversion — full build with OCR
#[cfg(feature = "ocr")]
async fn do_convert(input: &PathBuf, output: &Option<PathBuf>, optimize_llm: bool, quiet: bool, ocr_languages: &Vec<OcrLanguage>, output_format: &OutputFormat) -> Result<()> {
    if !input.exists() { anyhow::bail!("Input file not found: {}", input.display()); }
    if !quiet { println!("{}", "Converting document...".cyan().bold()); println!("  Input:  {}", input.display()); }
    let ext = output_format.extension();
    let output_path = output.clone().unwrap_or_else(|| { let mut p = input.clone(); p.set_extension(ext); p });
    if !quiet { println!("  Output: {} ({})", output_path.display(), output_format); }
    let result = BatchProcessor::new()
        .add_paths(&[input.clone()])
        .output_format(output_format.clone())
        .parallel(1)
        .ocr_languages(ocr_languages.clone())
        .execute().await?;
    if let Some((_, conversion)) = result.successes.first() {
        if let Some(parent) = output_path.parent() { tokio::fs::create_dir_all(parent).await?; }
        conversion.save_to_file(&output_path).await?;
        if !quiet { println!(); println!("{}", "Conversion completed!".green().bold()); println!("  Saved: {}", output_path.display()); println!("  Words: {}", conversion.metadata.word_count); }
    } else if let Some((_, error)) = result.failures.first() { anyhow::bail!("Conversion failed: {}", error); }
    Ok(())
}

/// Single-file conversion — light build without OCR
#[cfg(not(feature = "ocr"))]
async fn do_convert(input: &PathBuf, output: &Option<PathBuf>, optimize_llm: bool, quiet: bool, output_format: &OutputFormat) -> Result<()> {
    if !input.exists() { anyhow::bail!("Input file not found: {}", input.display()); }
    if !quiet { println!("{}", "Converting document...".cyan().bold()); println!("  Input:  {}", input.display()); }
    let ext = output_format.extension();
    let output_path = output.clone().unwrap_or_else(|| { let mut p = input.clone(); p.set_extension(ext); p });
    if !quiet { println!("  Output: {} ({})", output_path.display(), output_format); }
    let result = BatchProcessor::new()
        .add_paths(&[input.clone()])
        .output_format(output_format.clone())
        .parallel(1)
        .execute().await?;
    if let Some((_, conversion)) = result.successes.first() {
        if let Some(parent) = output_path.parent() { tokio::fs::create_dir_all(parent).await?; }
        conversion.save_to_file(&output_path).await?;
        if !quiet { println!(); println!("{}", "Conversion completed!".green().bold()); println!("  Saved: {}", output_path.display()); println!("  Words: {}", conversion.metadata.word_count); }
    } else if let Some((_, error)) = result.failures.first() { anyhow::bail!("Conversion failed: {}", error); }
    Ok(())
}

/// Batch conversion — full build with OCR
#[cfg(feature = "ocr")]
async fn do_batch(input: &[PathBuf], output: &PathBuf, jobs: Option<usize>, combined: bool, quiet: bool, ocr_languages: &Vec<OcrLanguage>, output_format: &OutputFormat) -> Result<()> {
    let parallel_jobs = jobs.unwrap_or_else(default_parallel_jobs);
    if !quiet { println!("{}", "Batch converting documents...".cyan().bold()); println!("  Input:    {} paths", input.len()); println!("  Output:   {}", output.display()); println!("  Parallel: {} jobs", parallel_jobs); println!("  Format:   {}", output_format); println!("  Combined: {}", if combined { "yes" } else { "no" }); println!(); }
    let all_files = collect_files(input);
    let total = all_files.len();
    if total == 0 { if !quiet { println!("{}", "No supported files found.".yellow()); } return Ok(()); }
    if !quiet { println!("{}", format!("Found {} files:", total).green()); for f in &all_files { let fmt = detect_format(f); let sz = std::fs::metadata(f).map(|m| format_size(m.len())).unwrap_or_default(); let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("?"); println!("  {} [{}] ({})", name, fmt, sz); } println!(); }
    let pb = if !quiet { let pb = ProgressBar::new(total as u64); pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})").unwrap().progress_chars("#>-")); Some(pb) } else { None };
    let result = BatchProcessor::new().add_paths(input).output_format(output_format.clone()).parallel(parallel_jobs).ocr_languages(ocr_languages.clone()).execute().await?;
    if let Some(pb) = &pb { pb.finish_with_message("done"); }
    result.save_all(output, combined).await?;
    if !quiet { println!(); println!("{}", "Batch conversion complete!".green().bold()); println!("  Success: {}/{} ({:.1}%)", result.successes.len(), result.total_files, result.success_rate()); if !result.failures.is_empty() { println!("  Failed:  {}", result.failures.len().to_string().red()); for (p, e) in &result.failures { println!("    {}: {}", p.file_name().and_then(|n| n.to_str()).unwrap_or("?"), e); } } println!("  Time:    {:.2}s", result.total_time_secs); println!("  Words:   {}", result.total_word_count()); println!("  Speed:   {:.1} files/s", result.successes.len() as f64 / result.total_time_secs.max(0.001)); println!("  Output:  {}", output.display()); }
    Ok(())
}

/// Batch conversion — light build without OCR
#[cfg(not(feature = "ocr"))]
async fn do_batch(input: &[PathBuf], output: &PathBuf, jobs: Option<usize>, combined: bool, quiet: bool, output_format: &OutputFormat) -> Result<()> {
    let parallel_jobs = jobs.unwrap_or_else(default_parallel_jobs);
    if !quiet { println!("{}", "Batch converting documents...".cyan().bold()); println!("  Input:    {} paths", input.len()); println!("  Output:   {}", output.display()); println!("  Parallel: {} jobs", parallel_jobs); println!("  Format:   {}", output_format); println!("  Combined: {}", if combined { "yes" } else { "no" }); println!(); }
    let all_files = collect_files(input);
    let total = all_files.len();
    if total == 0 { if !quiet { println!("{}", "No supported files found.".yellow()); } return Ok(()); }
    if !quiet { println!("{}", format!("Found {} files:", total).green()); for f in &all_files { let fmt = detect_format(f); let sz = std::fs::metadata(f).map(|m| format_size(m.len())).unwrap_or_default(); let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("?"); println!("  {} [{}] ({})", name, fmt, sz); } println!(); }
    let pb = if !quiet { let pb = ProgressBar::new(total as u64); pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})").unwrap().progress_chars("#>-")); Some(pb) } else { None };
    let result = BatchProcessor::new().add_paths(input).output_format(output_format.clone()).parallel(parallel_jobs).execute().await?;
    if let Some(pb) = &pb { pb.finish_with_message("done"); }
    result.save_all(output, combined).await?;
    if !quiet { println!(); println!("{}", "Batch conversion complete!".green().bold()); println!("  Success: {}/{} ({:.1}%)", result.successes.len(), result.total_files, result.success_rate()); if !result.failures.is_empty() { println!("  Failed:  {}", result.failures.len().to_string().red()); for (p, e) in &result.failures { println!("    {}: {}", p.file_name().and_then(|n| n.to_str()).unwrap_or("?"), e); } } println!("  Time:    {:.2}s", result.total_time_secs); println!("  Words:   {}", result.total_word_count()); println!("  Speed:   {:.1} files/s", result.successes.len() as f64 / result.total_time_secs.max(0.001)); println!("  Output:  {}", output.display()); }
    Ok(())
}
