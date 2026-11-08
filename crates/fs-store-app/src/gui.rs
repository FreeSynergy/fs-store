//! The window: `fs-store [--size WxH] [--snapshot file.png]`; `FS_STORE_ROOT` as for the commands.
//! Pattern: Facade (one entry point over model, engine and view).

use std::sync::{Arc, Mutex};

use crate::engine::EngineBackend;
use crate::launcher::{Launcher, ProcessLauncher};
use crate::model::StoreModel;
use crate::view::store_app;
use fs_components_app::{WindowOptions, launch, parse_size};

/// Opens the window, or draws a picture with `--snapshot`.
///
/// # Errors
/// Text for the person when an argument is wrong or the window cannot open.
pub fn run(argv: &[String]) -> Result<(), String> {
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
    let launcher: Arc<dyn Launcher> =
        Arc::new(ProcessLauncher::new(fs_store_engine::default_bin_dir()));
    let app = store_app(&Arc::new(Mutex::new(model)), &launcher).map_err(|e| e.to_string())?;
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
