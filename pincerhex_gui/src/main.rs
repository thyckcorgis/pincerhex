#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod board;
#[cfg(debug_assertions)]
mod frame_history;

use app::PincerhexApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        maximized: true,
        ..Default::default()
    };
    eframe::run_native(
        "Pincerhex",
        options,
        Box::new(|cc| Box::new(PincerhexApp::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "pincerhex_canvas",
                web_options,
                Box::new(|cc| Box::new(PincerhexApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
