#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Raw FFI binding to MessageBoxW — no external crate needed, user32.dll is always available on Windows
#[cfg(all(feature = "gui", target_os = "windows"))]
mod win_msgbox {
    #[link(name = "user32")]
    extern "system" {
        pub fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, r#type: u32) -> i32;
    }

    /// Show a Windows message box with an error icon
    pub unsafe fn show_error(title: &str, msg: &str) {
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let msg_wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(0, msg_wide.as_ptr(), title_wide.as_ptr(), 0x10); // MB_ICONERROR
    }

    /// Show a Windows message box with an information icon
    pub unsafe fn show_info(title: &str, msg: &str) {
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let msg_wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        MessageBoxW(0, msg_wide.as_ptr(), title_wide.as_ptr(), 0x40); // MB_ICONINFORMATION
    }
}

/// MarkItDown-RST — Multi-threaded Document-to-Markdown Converter
fn main() -> eframe::Result<()> {
    // On Windows GUI builds, show panics in a message box instead of silently crashing
    // (without windows_subsystem = "windows", the console disappears instantly on panic)
    #[cfg(all(feature = "gui", target_os = "windows"))]
    {
        std::panic::set_hook(Box::new(|info| {
            let msg = info.to_string();
            eprintln!("PANIC: {msg}");
            unsafe { win_msgbox::show_error("MarkItDown-RST — Fatal Error", &msg); }
        }));
    }

    #[cfg(feature = "logs")]
    {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    // Check for --cli flag — redirect to CLI binary
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--cli" || args.contains(&"--cli".to_string())) {
        #[cfg(all(feature = "gui", target_os = "windows"))]
        unsafe { win_msgbox::show_info("MarkItDown-RST", "Use markitdown-cli.exe for command-line mode."); }

        #[cfg(not(all(feature = "gui", target_os = "windows")))]
        eprintln!("Use markitdown-cli binary for command-line mode.");

        std::process::exit(0);
    }

    #[cfg(feature = "gui")]
    {
        markitdown_rst::gui::run_gui()
    }

    #[cfg(not(feature = "gui"))]
    {
        eprintln!("GUI not available. Use markitdown-cli for command-line mode.");
        std::process::exit(1);
    }
}
