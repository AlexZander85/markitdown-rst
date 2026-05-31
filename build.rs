fn main() {
    // Embed icon into Windows executable
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "MDrust — Document to Markdown Converter");
        res.set("ProductName", "MDrust");
        res.set("OriginalFilename", "mdrust.exe");
        res.set("LegalCopyright", "MIT License");
        if let Err(e) = res.compile() {
            eprintln!("Warning: winres compile failed: {e}");
        }
    }
}
