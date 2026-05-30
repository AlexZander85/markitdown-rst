//! Internationalization (i18n) support for the markitdown-rs GUI application.
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

/// Holds the current language and provides translated `&'static str` values
/// for every user-facing string in the application.
///
/// # Example
///
/// ```ignore
/// use markitdown_rs::i18n::{I18n, Language};
///
/// let mut i = I18n::default();        // English
/// assert_eq!(i.convert(), "Convert");
/// i.set_language(Language::Ru);
/// ```
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

    // -- translated strings -----------------------------------------------

    /// App title (same in every language).
    pub fn app_title(&self) -> &'static str {
        "MarkItDown-RS"
    }

    /// Subtitle beneath the app title.
    pub fn subtitle(&self) -> &'static str {
        match self.lang {
            Language::En => "Multi-threaded Document \u{2192} Markdown Converter",
            Language::Ru => "\u{041c}\u{043d}\u{043e}\u{0433}\u{043e}\u{043f}\u{043e}\u{0442}\u{043e}\u{0447}\u{043d}\u{044b}\u{0439} \u{043a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0435}\u{0440} \u{0434}\u{043e}\u{043a}\u{0443}\u{043c}\u{0435}\u{043d}\u{0442}\u{043e}\u{0432} \u{2192} Markdown",
            Language::Zh => "\u{591a}\u{7ebf}\u{7a0b}\u{6587}\u{6863}\u{8f6c} Markdown \u{8f6c}\u{6362}\u{5668}",
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

    /// Drop-zone placeholder text (may contain newlines).
    pub fn drop_files_here(&self) -> &'static str {
        match self.lang {
            Language::En => "Drop files here\nor click Add Files",
            Language::Ru => "\u{041f}\u{0435}\u{0440}\u{0435}\u{0442}\u{0430}\u{0449}\u{0438}\u{0442}\u{0435} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b} \u{0441}\u{044e}\u{0434}\u{0430}\n\u{0438}\u{043b}\u{0438} \u{043d}\u{0430}\u{0436}\u{043c}\u{0438}\u{0442}\u{0435} \u{0414}\u{043e}\u{0431}\u{0430}\u{0432}\u{0438}\u{0442}\u{044c} \u{0444}\u{0430}\u{0439}\u{043b}\u{044b}",
            Language::Zh => "\u{62d6}\u{653e}\u{6587}\u{4ef6}\u{5230}\u{6b64}\u{5904}\n\u{6216}\u{70b9}\u{51fb}\u{6dfb}\u{52a0}\u{6587}\u{4ef6}",
        }
    }

    /// Markdown preview placeholder (may contain newlines).
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

    /// "files in queue" suffix (e.g. "5 files in queue").
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
            Language::En => "Tesseract OCR not found. Install: sudo apt install tesseract-ocr",
            Language::Ru => "Tesseract OCR \u{043d}\u{0435} \u{043d}\u{0430}\u{0439}\u{0434}\u{0435}\u{043d}. \u{0423}\u{0441}\u{0442}\u{0430}\u{043d}\u{043e}\u{0432}\u{0438}\u{0442}\u{0435}: sudo apt install tesseract-ocr",
            Language::Zh => "\u{672a}\u{627e}\u{5230} Tesseract OCR\u{3002}\u{5b89}\u{88c5}\u{ff1a}sudo apt install tesseract-ocr",
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
            Language::Zh => "\u{5173}\u{4e8e}",
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_language() {
        assert_eq!(format!("{}", Language::En), "English");
        assert_eq!(format!("{}", Language::Ru), "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}");
        assert_eq!(format!("{}", Language::Zh), "\u{4e2d}\u{6587}");
    }

    #[test]
    fn default_is_english() {
        let i18n = I18n::default();
        assert_eq!(i18n.language(), Language::En);
        assert_eq!(i18n.convert(), "Convert");
        assert_eq!(i18n.clear(), "Clear");
    }

    #[test]
    fn set_language_switches() {
        let mut i18n = I18n::default();
        i18n.set_language(Language::Ru);
        assert_eq!(i18n.language(), Language::Ru);
        assert_eq!(i18n.convert(), "\u{041a}\u{043e}\u{043d}\u{0432}\u{0435}\u{0440}\u{0442}\u{0438}\u{0440}\u{043e}\u{0432}\u{0430}\u{0442}\u{044c}");

        i18n.set_language(Language::Zh);
        assert_eq!(i18n.language(), Language::Zh);
        assert_eq!(i18n.convert(), "\u{8f6c}\u{6362}");
    }

    #[test]
    fn app_title_is_same_in_all_languages() {
        let en = I18n::new(Language::En);
        let ru = I18n::new(Language::Ru);
        let zh = I18n::new(Language::Zh);
        assert_eq!(en.app_title(), "MarkItDown-RS");
        assert_eq!(ru.app_title(), "MarkItDown-RS");
        assert_eq!(zh.app_title(), "MarkItDown-RS");
    }

    #[test]
    fn all_methods_return_non_empty() {
        for lang in [Language::En, Language::Ru, Language::Zh] {
            let i = I18n::new(lang);
            assert!(!i.app_title().is_empty());
            assert!(!i.subtitle().is_empty());
            assert!(!i.file_queue().is_empty());
            assert!(!i.add_files().is_empty());
            assert!(!i.add_folder().is_empty());
            assert!(!i.clear().is_empty());
            assert!(!i.convert().is_empty());
            assert!(!i.threads().is_empty());
            assert!(!i.output().is_empty());
            assert!(!i.combined_output().is_empty());
            assert!(!i.preview().is_empty());
            assert!(!i.open_in_browser().is_empty());
            assert!(!i.settings().is_empty());
            assert!(!i.language_label().is_empty());
            assert!(!i.ocr_languages().is_empty());
            assert!(!i.ready_status().is_empty());
            assert!(!i.converting().is_empty());
            assert!(!i.completed().is_empty());
            assert!(!i.error().is_empty());
            assert!(!i.files_converted().is_empty());
            assert!(!i.drop_files_here().is_empty());
            assert!(!i.markdown_preview().is_empty());
            assert!(!i.save().is_empty());
            assert!(!i.save_all().is_empty());
            assert!(!i.theme().is_empty());
            assert!(!i.dark().is_empty());
            assert!(!i.light().is_empty());
            assert!(!i.font_size().is_empty());
            assert!(!i.no_files_in_queue().is_empty());
            assert!(!i.files_in_queue().is_empty());
            assert!(!i.total_words().is_empty());
            assert!(!i.conversion_time().is_empty());
            assert!(!i.speed().is_empty());
            assert!(!i.tesseract_not_found().is_empty());
            assert!(!i.ocr_engine().is_empty());
            assert!(!i.supported_formats().is_empty());
            assert!(!i.editor().is_empty());
            assert!(!i.viewer().is_empty());
            assert!(!i.copy().is_empty());
            assert!(!i.word_count().is_empty());
            assert!(!i.failed_files().is_empty());
            assert!(!i.about().is_empty());
            assert!(!i.version().is_empty());
            assert!(!i.select_ocr_languages().is_empty());
        }
    }
}
