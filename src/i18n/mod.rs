//! Internationalization (i18n) support for the MDrust GUI application.
//!
//! Provides translated UI strings in English, Russian, and Chinese.

use std::fmt;

// ---------------------------------------------------------------------------
// Language enum
// ---------------------------------------------------------------------------

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    En,
    Ru,
    Zh,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::En => write!(f, "English"),
            Language::Ru => write!(f, "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}"),
            Language::Zh => write!(f, "\u{4e2d}\u{6587}"),
        }
    }
}

// ---------------------------------------------------------------------------
// I18n struct
// ---------------------------------------------------------------------------

/// Holds the current language and provides translated strings.
pub struct I18n {
    lang: Language,
}

impl Default for I18n {
    fn default() -> Self {
        Self {
            lang: Language::En,
        }
    }
}

impl PartialEq for I18n {
    fn eq(&self, other: &Self) -> bool {
        self.lang == other.lang
    }
}

impl I18n {
    // -- constructors / setters -------------------------------------------

    /// Create a new `I18n` with the given language.
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    /// Switch the active language.
    pub fn set_language(&mut self, lang: Language) {
        self.lang = lang;
    }

    /// Return the currently active language.
    pub fn language(&self) -> Language {
        self.lang
    }

    /// Return display label for the current language.
    pub fn lang_label(&self) -> &'static str {
        match self.lang {
            Language::En => "EN",
            Language::Ru => "RU",
            Language::Zh => "ZH",
        }
    }

    // -- generic key-based translation ------------------------------------

    /// Get translation by key name (for dynamic lookups).
    pub fn t(&self, key: &str) -> String {
        match key {
            "actions" => self.actions().to_string(),
            "settings" => self.settings().to_string(),
            "add_files" => self.add_files().to_string(),
            "add_folder" => self.add_folder().to_string(),
            "clear" => self.clear().to_string(),
            "drop_files_here" => self.drop_files_here().to_string(),
            "or_click_add" => self.or_click_add().to_string(),
            "threads" => self.threads().to_string(),
            "combined_output" => self.combined_output().to_string(),
            "output_dir" => self.output_dir().to_string(),
            "convert" => self.convert().to_string(),
            "file_queue" => self.file_queue().to_string(),
            "preview" => self.preview().to_string(),
            "copy" => self.copy().to_string(),
            "save" => self.save().to_string(),
            "metadata" => self.metadata().to_string(),
            "rendered" => self.rendered().to_string(),
            "raw_md" => self.raw_md().to_string(),
            "converting" => self.converting().to_string(),
            "completed" => self.completed().to_string(),
            "error" => self.error().to_string(),
            "files_converted" => self.files_converted().to_string(),
            "total_words" => self.total_words().to_string(),
            "ready_status" => self.ready_status().to_string(),
            "save_all" => self.save_all().to_string(),
            "open_in_browser" => self.open_in_browser().to_string(),
            "word_count" => self.word_count().to_string(),
            "failed_files" => self.failed_files().to_string(),
            _ => key.to_string(),
        }
    }

    // -- translated strings -----------------------------------------------

    /// App title (same in every language).
    pub fn app_title(&self) -> &'static str {
        "MDrust"
    }

    /// Subtitle beneath the app title.
    pub fn subtitle(&self) -> &'static str {
        match self.lang {
            Language::En => "Multi-threaded Document \u{2192} Markdown Converter",
            Language::Ru => "\u{041c}\u{043d}\u{043e}\u{0433}\u{043e}\u{043f}\u{043e}\u{0442}\u{043e}\u{0447}\u{043d}\u{044b}\u{0439} \u{043a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0435}\u{0440} \u{0434}\u{043e}\u{043a}\u{0443}\u{043c}\u{0435}\u{043d}\u{0442}\u{043e}\u{0432} \u{2192} Markdown",
            Language::Zh => "\u{591a}\u{7ebf}\u{7a0b}\u{6587}\u{6863}\u{8f6c} Markdown \u{8f6c}\u{6362}\u{5668}",
        }
    }

    /// "Actions" heading.
    pub fn actions(&self) -> &'static str {
        match self.lang {
            Language::En => "Actions",
            Language::Ru => "\u{0414}\u{0435}\u{0439}\u{0441}\u{0442}\u{0432}\u{0438}\u{044f}",
            Language::Zh => "\u{64cd}\u{4f5c}",
        }
    }

    /// "File Queue" heading.
    pub fn file_queue(&self) -> &'static str {
        match self.lang {
            Language::En => "File Queue",
            Language::Ru => "\u{041e}\u{0447}\u{0435}\u{0440}\u{0435}\u{0434}\u{044c} \u{0444}\u{0430}\u{0439}\u{043b}\u{043e}\u{0432}",
            Language::Zh => "\u{6587}\u{4ef6}\u{961f}\u{5217}",
        }
    }

    /// "Add Files" button label.
    pub fn add_files(&self) -> &'static str {
        match self.lang {
            Language::En => "Add Files",
            Language::Ru => "\u{0414}\u{043e}\u{0431}\u{0430}\u{0432}\u{0438}\u{0442}\u{044c} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b}",
            Language::Zh => "\u{6dfb}\u{52a0}\u{6587}\u{4ef6}",
        }
    }

    /// "Add Folder" button label.
    pub fn add_folder(&self) -> &'static str {
        match self.lang {
            Language::En => "Add Folder",
            Language::Ru => "\u{0414}\u{043e}\u{0431}\u{0430}\u{0432}\u{0438}\u{0442}\u{044c} \u{043f}\u{0430}\u{043f}\u{043a}\u{0443}",
            Language::Zh => "\u{6dfb}\u{52a0}\u{6587}\u{4ef6}\u{5939}",
        }
    }

    /// "Clear" button label.
    pub fn clear(&self) -> &'static str {
        match self.lang {
            Language::En => "Clear",
            Language::Ru => "\u{041e}\u{0447}\u{0438}\u{0441}\u{0442}\u{0438}\u{0442}\u{044c}",
            Language::Zh => "\u{6e05}\u{7a7a}",
        }
    }

    /// "Convert" button label.
    pub fn convert(&self) -> &'static str {
        match self.lang {
            Language::En => "Convert",
            Language::Ru => "\u{041a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0438}\u{0440}\u{043e}\u{0432}\u{0430}\u{0442}\u{044c}",
            Language::Zh => "\u{8f6c}\u{6362}",
        }
    }

    /// "Threads" label.
    pub fn threads(&self) -> &'static str {
        match self.lang {
            Language::En => "Threads",
            Language::Ru => "\u{041f}\u{043e}\u{0442}\u{043e}\u{043a}\u{0438}",
            Language::Zh => "\u{7ebf}\u{7a0b}\u{6570}",
        }
    }

    /// "Output" heading.
    pub fn output(&self) -> &'static str {
        match self.lang {
            Language::En => "Output",
            Language::Ru => "\u{0412}\u{044b}\u{0432}\u{043e}\u{0434}",
            Language::Zh => "\u{8f93}\u{51fa}",
        }
    }

    /// "Output Dir" label.
    pub fn output_dir(&self) -> &'static str {
        match self.lang {
            Language::En => "Output Dir",
            Language::Ru => "\u{0412}\u{044b}\u{0445}\u{043e}\u{0434}\u{043d}\u{0430}\u{044f} \u{043f}\u{0430}\u{043f}\u{043a}\u{0430}",
            Language::Zh => "\u{8f93}\u{51fa}\u{76ee}\u{5f55}",
        }
    }

    /// "Output Format" label.
    pub fn output_format(&self) -> &'static str {
        match self.lang {
            Language::En => "Output Format",
            Language::Ru => "\u{0424}\u{043e}\u{0440}\u{043c}\u{0430}\u{0442} \u{0432}\u{044b}\u{0432}\u{043e}\u{0434}\u{0430}",
            Language::Zh => "\u{8f93}\u{51fa}\u{683c}\u{5f0f}",
        }
    }

    /// "Combined output" label.
    pub fn combined_output(&self) -> &'static str {
        match self.lang {
            Language::En => "Combined output",
            Language::Ru => "\u{041e}\u{0431}\u{044a}\u{0435}\u{0434}\u{0438}\u{043d}\u{0451}\u{043d}\u{043d}\u{044b}\u{0439} \u{0432}\u{044b}\u{0432}\u{043e}\u{0434}",
            Language::Zh => "\u{5408}\u{5e76}\u{8f93}\u{51fa}",
        }
    }

    /// "Preview" tab / heading.
    pub fn preview(&self) -> &'static str {
        match self.lang {
            Language::En => "Preview",
            Language::Ru => "\u{041f}\u{0440}\u{0435}\u{0434}\u{043f}\u{0440}\u{043e}\u{0441}\u{043c}\u{043e}\u{0442}\u{0440}",
            Language::Zh => "\u{9884}\u{89c8}",
        }
    }

    /// "Open in Browser" button label.
    pub fn open_in_browser(&self) -> &'static str {
        match self.lang {
            Language::En => "Open in Browser",
            Language::Ru => "\u{041e}\u{0442}\u{043a}\u{0440}\u{044b}\u{0442}\u{044c} \u{0432} \u{0431}\u{0440}\u{0430}\u{0443}\u{0437}\u{0435}\u{0440}\u{0435}",
            Language::Zh => "\u{5728}\u{6d4f}\u{89c8}\u{5668}\u{4e2d}\u{6253}\u{5f00}",
        }
    }

    /// "Settings" heading.
    pub fn settings(&self) -> &'static str {
        match self.lang {
            Language::En => "Settings",
            Language::Ru => "\u{041d}\u{0430}\u{0441}\u{0442}\u{0440}\u{043e}\u{0439}\u{043a}\u{0438}",
            Language::Zh => "\u{8bbe}\u{7f6e}",
        }
    }

    /// "Language" label.
    pub fn language_label(&self) -> &'static str {
        match self.lang {
            Language::En => "Language",
            Language::Ru => "\u{042f}\u{0437}\u{044b}\u{043a}",
            Language::Zh => "\u{8bed}\u{8a00}",
        }
    }

    /// "OCR Languages" label.
    pub fn ocr_languages(&self) -> &'static str {
        match self.lang {
            Language::En => "OCR Languages",
            Language::Ru => "\u{042f}\u{0437}\u{044b}\u{043a}\u{0438} OCR",
            Language::Zh => "OCR \u{8bed}\u{8a00}",
        }
    }

    /// Status bar: ready message.
    pub fn ready_status(&self) -> &'static str {
        match self.lang {
            Language::En => "Ready \u{2014} Drop files or click Add Files",
            Language::Ru => "\u{0413}\u{043e}\u{0442}\u{043e}\u{0432} \u{2014} \u{041f}\u{0435}\u{0440}\u{0435}\u{0442}\u{0430}\u{0449}\u{0438}\u{0442}\u{0435} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b} \u{0438}\u{043b}\u{0438} \u{043d}\u{0430}\u{0436}\u{043c}\u{0438}\u{0442}\u{0435} \u{0414}\u{043e}\u{0431}\u{0430}\u{0432}\u{0438}\u{0442}\u{044c}",
            Language::Zh => "\u{5c31}\u{7eea} \u{2014} \u{62d6}\u{653e}\u{6587}\u{4ef6}\u{6216}\u{70b9}\u{51fb}\u{6dfb}\u{52a0}\u{6587}\u{4ef6}",
        }
    }

    /// "Converting..." status.
    pub fn converting(&self) -> &'static str {
        match self.lang {
            Language::En => "Converting...",
            Language::Ru => "\u{041a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0430}\u{0446}\u{0438}\u{044f}...",
            Language::Zh => "\u{8f6c}\u{6362}\u{4e2d}...",
        }
    }

    /// "Completed" status.
    pub fn completed(&self) -> &'static str {
        match self.lang {
            Language::En => "Completed",
            Language::Ru => "\u{0417}\u{0430}\u{0432}\u{0435}\u{0440}\u{0448}\u{0435}\u{043d}\u{043e}",
            Language::Zh => "\u{5b8c}\u{6210}",
        }
    }

    /// "Error" label.
    pub fn error(&self) -> &'static str {
        match self.lang {
            Language::En => "Error",
            Language::Ru => "\u{041e}\u{0448}\u{0438}\u{0431}\u{043a}\u{0430}",
            Language::Zh => "\u{9519}\u{8bef}",
        }
    }

    /// "files converted" suffix (e.g. "3 files converted").
    pub fn files_converted(&self) -> &'static str {
        match self.lang {
            Language::En => "files converted",
            Language::Ru => "\u{0444}\u{0430}\u{0439}\u{043b}\u{043e}\u{0432} \u{043a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0438}\u{0440}\u{043e}\u{0432}\u{0430}\u{043d}\u{043e}",
            Language::Zh => "\u{4e2a}\u{6587}\u{4ef6}\u{5df2}\u{8f6c}\u{6362}",
        }
    }

    /// Drop-zone placeholder text.
    pub fn drop_files_here(&self) -> &'static str {
        match self.lang {
            Language::En => "Drop files here",
            Language::Ru => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0442}\u{0430}\u{0449}\u{0438}\u{0442}\u{0435} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b} \u{0441}\u{044e}\u{0434}\u{0430}",
            Language::Zh => "\u{62d6}\u{653e}\u{6587}\u{4ef6}\u{5230}\u{6b64}\u{5904}",
        }
    }

    /// "or click Add Files"
    pub fn or_click_add(&self) -> &'static str {
        match self.lang {
            Language::En => "or click Add Files",
            Language::Ru => "\u{0438}\u{043b}\u{0438} \u{043d}\u{0430}\u{0436}\u{043c}\u{0438}\u{0442}\u{0435} \u{0414}\u{043e}\u{0431}\u{0430}\u{0432}\u{0438}\u{0442}\u{044c} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b}",
            Language::Zh => "\u{6216}\u{70b9}\u{51fb}\u{6dfb}\u{52a0}\u{6587}\u{4ef6}",
        }
    }

    /// "Metadata" label.
    pub fn metadata(&self) -> &'static str {
        match self.lang {
            Language::En => "Metadata",
            Language::Ru => "\u{041c}\u{0435}\u{0442}\u{0430}\u{0434}\u{0430}\u{043d}\u{043d}\u{044b}\u{0435}",
            Language::Zh => "\u{5143}\u{6570}\u{636e}",
        }
    }

    /// "Rendered" label.
    pub fn rendered(&self) -> &'static str {
        match self.lang {
            Language::En => "Rendered",
            Language::Ru => "\u{0420}\u{0435}\u{043d}\u{0434}\u{0435}\u{0440}",
            Language::Zh => "\u{6e32}\u{67d3}",
        }
    }

    /// "Raw MD" label.
    pub fn raw_md(&self) -> &'static str {
        match self.lang {
            Language::En => "Raw MD",
            Language::Ru => "\u{0418}\u{0441}\u{0445}\u{043e}\u{0434}\u{043d}\u{044b}\u{0439} MD",
            Language::Zh => "\u{539f}\u{59cb} MD",
        }
    }

    /// Markdown preview placeholder.
    pub fn markdown_preview(&self) -> &'static str {
        match self.lang {
            Language::En => "Markdown preview will appear here\nafter conversion",
            Language::Ru => "\u{041f}\u{0440}\u{0435}\u{0434}\u{043f}\u{0440}\u{043e}\u{0441}\u{043c}\u{043e}\u{0442}\u{0440} Markdown \u{043f}\u{043e}\u{044f}\u{0432}\u{0438}\u{0442}\u{0441}\u{044f} \u{0437}\u{0434}\u{0435}\u{0441}\u{044c}\n\u{043f}\u{043e}\u{0441}\u{043b}\u{0435} \u{043a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0430}\u{0446}\u{0438}\u{0438}",
            Language::Zh => "Markdown \u{9884}\u{89c8}\u{5c06}\u{5728}\u{8f6c}\u{6362}\u{540e}\n\u{663e}\u{793a}\u{5728}\u{6b64}\u{5904}",
        }
    }

    /// "Save" button label.
    pub fn save(&self) -> &'static str {
        match self.lang {
            Language::En => "Save",
            Language::Ru => "\u{0421}\u{043e}\u{0445}\u{0440}\u{0430}\u{043d}\u{0438}\u{0442}\u{044c}",
            Language::Zh => "\u{4fdd}\u{5b58}",
        }
    }

    /// "Save All" button label.
    pub fn save_all(&self) -> &'static str {
        match self.lang {
            Language::En => "Save All",
            Language::Ru => "\u{0421}\u{043e}\u{0445}\u{0440}\u{0430}\u{043d}\u{0438}\u{0442}\u{044c} \u{0432}\u{0441}\u{0435}",
            Language::Zh => "\u{5168}\u{90e8}\u{4fdd}\u{5b58}",
        }
    }

    /// "Theme" label.
    pub fn theme(&self) -> &'static str {
        match self.lang {
            Language::En => "Theme",
            Language::Ru => "\u{0422}\u{0435}\u{043c}\u{0430}",
            Language::Zh => "\u{4e3b}\u{9898}",
        }
    }

    /// "Dark" theme option.
    pub fn dark(&self) -> &'static str {
        match self.lang {
            Language::En => "Dark",
            Language::Ru => "\u{0422}\u{0451}\u{043c}\u{043d}\u{0430}\u{044f}",
            Language::Zh => "\u{6df1}\u{8272}",
        }
    }

    /// "Light" theme option.
    pub fn light(&self) -> &'static str {
        match self.lang {
            Language::En => "Light",
            Language::Ru => "\u{0421}\u{0432}\u{0435}\u{0442}\u{043b}\u{0430}\u{044f}",
            Language::Zh => "\u{6d45}\u{8272}",
        }
    }

    /// "Font Size" label.
    pub fn font_size(&self) -> &'static str {
        match self.lang {
            Language::En => "Font Size",
            Language::Ru => "\u{0420}\u{0430}\u{0437}\u{043c}\u{0435}\u{0440} \u{0448}\u{0440}\u{0438}\u{0444}\u{0442}\u{0430}",
            Language::Zh => "\u{5b57}\u{4f53}\u{5927}\u{5c0f}",
        }
    }

    /// "No files in queue" message.
    pub fn no_files_in_queue(&self) -> &'static str {
        match self.lang {
            Language::En => "No files in queue",
            Language::Ru => "\u{041d}\u{0435}\u{0442} \u{0444}\u{0430}\u{0439}\u{043b}\u{043e}\u{0432} \u{0432} \u{043e}\u{0447}\u{0435}\u{0440}\u{0435}\u{0434}\u{0438}",
            Language::Zh => "\u{961f}\u{5217}\u{4e2d}\u{6ca1}\u{6709}\u{6587}\u{4ef6}",
        }
    }

    /// "files in queue" suffix.
    pub fn files_in_queue(&self) -> &'static str {
        match self.lang {
            Language::En => "files in queue",
            Language::Ru => "\u{0444}\u{0430}\u{0439}\u{043b}\u{043e}\u{0432} \u{0432} \u{043e}\u{0447}\u{0435}\u{0440}\u{0435}\u{0434}\u{0438}",
            Language::Zh => "\u{4e2a}\u{6587}\u{4ef6}\u{5728}\u{961f}\u{5217}\u{4e2d}",
        }
    }

    /// "Total words" label.
    pub fn total_words(&self) -> &'static str {
        match self.lang {
            Language::En => "Total words",
            Language::Ru => "\u{0412}\u{0441}\u{0435}\u{0433}\u{043e} \u{0441}\u{043b}\u{043e}\u{0432}",
            Language::Zh => "\u{603b}\u{8bcd}\u{6570}",
        }
    }

    /// "Time" (conversion time) label.
    pub fn conversion_time(&self) -> &'static str {
        match self.lang {
            Language::En => "Time",
            Language::Ru => "\u{0412}\u{0440}\u{0435}\u{043c}\u{044f}",
            Language::Zh => "\u{7528}\u{65f6}",
        }
    }

    /// "Speed" label.
    pub fn speed(&self) -> &'static str {
        match self.lang {
            Language::En => "Speed",
            Language::Ru => "\u{0421}\u{043a}\u{043e}\u{0440}\u{043e}\u{0441}\u{0442}\u{044c}",
            Language::Zh => "\u{901f}\u{5ea6}",
        }
    }

    /// Tesseract OCR not-found warning.
    pub fn tesseract_not_found(&self) -> &'static str {
        match self.lang {
            Language::En => "OCR: Tesseract engine not installed (tessdata embedded, install libtesseract-dev or tesseract-ocr)",
            Language::Ru => "OCR: \u{0434}\u{0432}\u{0438}\u{0436}\u{043e}\u{043a} Tesseract \u{043d}\u{0435} \u{0443}\u{0441}\u{0442}\u{0430}\u{043d}\u{043e}\u{0432}\u{043b}\u{0435}\u{043d} (\u{044f}\u{0437}\u{044b}\u{043a}\u{043e}\u{0432}\u{044b}\u{0435} \u{0434}\u{0430}\u{043d}\u{043d}\u{044b}\u{0435} \u{0432}\u{0441}\u{0442}\u{0440}\u{043e}\u{0435}\u{043d}\u{044b}, \u{0443}\u{0441}\u{0442}\u{0430}\u{043d}\u{043e}\u{0432}\u{0438}\u{0442}\u{0435} libtesseract-dev \u{0438}\u{043b}\u{0438} tesseract-ocr)",
            Language::Zh => "OCR: Tesseract \u{5f15}\u{64ce}\u{672a}\u{5b89}\u{88c5} (\u{8bed}\u{8a00}\u{6570}\u{636e}\u{5df2}\u{5d4c}\u{5165}\u{ff0c}\u{8bf7}\u{5b89}\u{88c5} libtesseract-dev \u{6216} tesseract-ocr)",
        }
    }

    /// "OCR Engine" label.
    pub fn ocr_engine(&self) -> &'static str {
        match self.lang {
            Language::En => "OCR Engine",
            Language::Ru => "\u{0414}\u{0432}\u{0438}\u{0436}\u{043e}\u{043a} OCR",
            Language::Zh => "OCR \u{5f15}\u{64ce}",
        }
    }

    /// "Supported Formats" label.
    pub fn supported_formats(&self) -> &'static str {
        match self.lang {
            Language::En => "Supported Formats",
            Language::Ru => "\u{041f}\u{043e}\u{0434}\u{0434}\u{0435}\u{0440}\u{0436}\u{0438}\u{0432}\u{0430}\u{0435}\u{043c}\u{044b}\u{0435} \u{0444}\u{043e}\u{0440}\u{043c}\u{0430}\u{0442}\u{044b}",
            Language::Zh => "\u{652f}\u{6301}\u{7684}\u{683c}\u{5f0f}",
        }
    }

    /// "Editor" tab / label.
    pub fn editor(&self) -> &'static str {
        match self.lang {
            Language::En => "Editor",
            Language::Ru => "\u{0420}\u{0435}\u{0434}\u{0430}\u{043a}\u{0442}\u{043e}\u{0440}",
            Language::Zh => "\u{7f16}\u{8f91}\u{5668}",
        }
    }

    /// "Viewer" tab / label.
    pub fn viewer(&self) -> &'static str {
        match self.lang {
            Language::En => "Viewer",
            Language::Ru => "\u{041f}\u{0440}\u{043e}\u{0441}\u{043c}\u{043e}\u{0442}\u{0440}",
            Language::Zh => "\u{67e5}\u{770b}\u{5668}",
        }
    }

    /// "Copy" button label.
    pub fn copy(&self) -> &'static str {
        match self.lang {
            Language::En => "Copy",
            Language::Ru => "\u{041a}\u{043e}\u{043f}\u{0438}\u{0440}\u{043e}\u{0432}\u{0430}\u{0442}\u{044c}",
            Language::Zh => "\u{590d}\u{5236}",
        }
    }

    /// "words" unit (e.g. "42 words").
    pub fn word_count(&self) -> &'static str {
        match self.lang {
            Language::En => "words",
            Language::Ru => "\u{0441}\u{043b}\u{043e}\u{0432}",
            Language::Zh => "\u{8bcd}",
        }
    }

    /// "Failed" label (for failed files count).
    pub fn failed_files(&self) -> &'static str {
        match self.lang {
            Language::En => "Failed",
            Language::Ru => "\u{041d}\u{0435} \u{0443}\u{0434}\u{0430}\u{043b}\u{043e}\u{0441}\u{044c}",
            Language::Zh => "\u{5931}\u{8d25}",
        }
    }

    /// "About" label.
    pub fn about(&self) -> &'static str {
        match self.lang {
            Language::En => "About",
            Language::Ru => "\u{041e} \u{043f}\u{0440}\u{043e}\u{0433}\u{0440}\u{0430}\u{043c}\u{043c}\u{0435}",
            Language::Zh => "\u{5173}\u{043d}",
        }
    }

    /// "Version" label.
    pub fn version(&self) -> &'static str {
        match self.lang {
            Language::En => "Version",
            Language::Ru => "\u{0412}\u{0435}\u{0440}\u{0441}\u{0438}\u{044f}",
            Language::Zh => "\u{7248}\u{672c}",
        }
    }

    /// "Select OCR languages" placeholder / prompt.
    pub fn select_ocr_languages(&self) -> &'static str {
        match self.lang {
            Language::En => "Select OCR languages",
            Language::Ru => "\u{0412}\u{044b}\u{0431}\u{0435}\u{0440}\u{0438}\u{0442}\u{0435} \u{044f}\u{0437}\u{044b}\u{043a}\u{0438} OCR",
            Language::Zh => "\u{9009}\u{62e9} OCR \u{8bed}\u{8a00}",
        }
    }
}
