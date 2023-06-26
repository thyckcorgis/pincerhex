mod app;
mod board;

use app::PincerhexApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        maximized: true,
        ..Default::default()
    };
    eframe::run_native(
        "Pincerhex",
        options,
        Box::new(|_cc| Box::<PincerhexApp>::default()),
    )
}
