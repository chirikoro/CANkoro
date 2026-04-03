#![allow(dead_code, unused_imports)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod can;
mod config;
mod dbc;
mod graph;
mod i18n;
mod log;
mod ui;
mod vector;

use i18n::t;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title(t("app.title")),
        ..Default::default()
    };

    eframe::run_native(
        "CANkoro",
        native_options,
        Box::new(|cc| Ok(Box::new(app::CankoroApp::new(cc)))),
    )
}
