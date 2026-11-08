//! The binary: wiring only. `fs-store-app [--size WxH] [--snapshot file.png]`; `FS_STORE_ROOT` as for the CLI.

use std::sync::{Arc, Mutex};

use fs_components_app::{WindowOptions, launch, parse_size};
use fs_store_app::engine::EngineBackend;
use fs_store_app::model::StoreModel;
use fs_store_app::view::store_app;

fn main() {
    if let Err(message) = run() {
        eprintln!("fs-store-app: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut size = (900, 640);
    let mut snapshot = None;
    let mut it = argv.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--size" => size = parse_size(it.next().ok_or("--size needs WxH")?)?,
            "--snapshot" => snapshot = Some(it.next().ok_or("--snapshot needs a file")?.clone()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let mut model = StoreModel::new(Arc::new(EngineBackend::from_environment()));
    model.refresh();
    let app = store_app(&Arc::new(Mutex::new(model))).map_err(|e| e.to_string())?;
    if let Some(file) = snapshot {
        app.picture(size.0, size.1, std::path::Path::new(&file))
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    let mut options = WindowOptions::default();
    options.width = size.0;
    options.height = size.1;
    launch(options, app).map_err(|e| e.to_string())
}
