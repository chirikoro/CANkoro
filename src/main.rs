#![allow(dead_code, unused_imports)]

mod app;
mod can;
mod config;
mod dbc;
mod graph;
mod log;
mod ui;
mod vector;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("CANkoro - CAN/CAN-FD Analysis"),
        ..Default::default()
    };

    eframe::run_native(
        "CANkoro",
        native_options,
        Box::new(|cc| Ok(Box::new(app::CankoroApp::new(cc)))),
    )
}
