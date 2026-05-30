fn main() {
    // Embed icon into Windows executable
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "MarkItDown-RST — Document to Markdown Converter");
        res.set("ProductName", "MarkItDown-RST");
        res.set("OriginalFilename", "markitdown-rst.exe");
        res.set("LegalCopyright", "MIT License");
        if let Err(e) = res.compile() {
            eprintln!("Warning: winres compile failed: {e}");
        }
    }
}
