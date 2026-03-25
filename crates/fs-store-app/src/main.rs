// fs-store-app — FreeSynergy Store GUI application.
// Dioxus components are PascalCase by convention — allow non_snake_case crate-wide.
#![allow(non_snake_case)]
//
// Entry point: launches the Dioxus desktop window.
// All state is managed via StoreContext (Provider Pattern).
// All rendering is delegated to views/ and the PackageView trait.

mod app;
mod context;
mod view;
mod views;

fn main() {
    let config = dioxus_desktop::Config::default().with_window(
        dioxus_desktop::WindowBuilder::default()
            .with_title("FreeSynergy Store")
            .with_inner_size(dioxus_desktop::LogicalSize::new(1100.0_f64, 700.0_f64)),
    );
    dioxus_desktop::launch::launch(app::App, vec![], vec![Box::new(config)]);
}
