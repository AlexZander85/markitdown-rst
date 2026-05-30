//! GUI module using egui/eframe

mod app;

pub use app::MarkItDownApp;

/// Run the GUI application
pub fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 650.0])
            .with_title("MarkItDown-RST — Multi-threaded Document Converter"),
        ..Default::default()
    };

    eframe::run_native(
        "MarkItDown-RST",
        options,
        Box::new(|_cc| Ok(Box::new(MarkItDownApp::new()))),
    )
}
