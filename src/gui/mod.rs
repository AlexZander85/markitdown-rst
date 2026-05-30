//! GUI module using egui/eframe

mod app;
pub mod theme;
pub mod fonts;

pub use app::MarkItDownApp;

use std::sync::Arc;

/// Load the application icon from embedded PNG bytes
fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../../assets/icon-256.png");
    let img = image::load_from_memory(png_bytes).ok()?;
    let img = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

/// Run the GUI application
pub fn run_gui() -> eframe::Result<()> {
    let icon = load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_min_inner_size([900.0, 650.0]);

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(Arc::new(icon_data));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MarkItDown-RST",
        options,
        Box::new(|cc| {
            crate::gui::theme::Theme::apply(&cc.egui_ctx, true);
            crate::gui::fonts::install(&cc.egui_ctx);
            Ok(Box::new(MarkItDownApp::new()))
        }),
    )
}
