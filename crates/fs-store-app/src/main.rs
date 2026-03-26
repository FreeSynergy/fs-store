// fs-store — FreeSynergy Store entry point.
#![deny(clippy::all, clippy::pedantic, warnings)]
// Dioxus components are PascalCase by convention — allow non_snake_case crate-wide.
#![allow(non_snake_case)]
//
// Detects at runtime whether a display is available:
//   - Display found → launches the Dioxus GUI
//   - No display    → prints a helpful message (use fs-store-cli for headless)
//
// All GUI state is managed via StoreContext (Provider Pattern).
// All rendering is delegated to views/ and the PackageView trait.

mod app;
mod context;
mod view;
mod views;

fn has_display() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok() || std::env::var("DISPLAY").is_ok()
}

fn main() {
    if !has_display() {
        eprintln!("fs-store: no display found — use 'fs-store-cli' for command-line access.");
        std::process::exit(1);
    }

    let config = dioxus_desktop::Config::default().with_window(
        dioxus_desktop::WindowBuilder::default()
            .with_title("FreeSynergy Store")
            .with_inner_size(dioxus_desktop::LogicalSize::new(1100.0_f64, 700.0_f64)),
    );
    dioxus_desktop::launch::launch(app::App, vec![], vec![Box::new(config)]);
}
