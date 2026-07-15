mod processor;
mod io;
mod app;
use eframe::egui;
use app::gui::*;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 500.0])
            .with_transparent(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "Анализатор спутниковых снимков",
        options,
        Box::new(|_cc| Box::<SatelliteApp>::default()),
    )
}