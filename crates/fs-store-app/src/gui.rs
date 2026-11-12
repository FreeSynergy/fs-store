//! The window: `fs-store [--size WxH] [--snapshot file.png] [--facet NAME] [--view cards|sources] [--demo]`;
//! `FS_STORE_ROOT` as for the commands. `--demo` runs the window over in-memory parts that touch no network, no file of the person and no process (see `demo`); `--view`
//! opens it on that page (both are for tests of the window, such as the typing test).
//! Pattern: Facade (one entry point over model, engine and view).

use std::sync::{Arc, Mutex};

use crate::demo::parts;
use crate::model::{StoreModel, View};
use crate::view::{Variant, start_check, store_app};
use fs_components_app::{WindowOptions, launch, parse_size};
use fs_interfaces::rpc::facet_from_args;

/// Opens the window, or draws a picture with `--snapshot`.
///
/// # Errors
/// Text for the person when an argument is wrong or the window cannot open.
pub fn run(argv: &[String]) -> Result<(), String> {
    let mut size = (900, 640);
    let mut snapshot = None;
    let mut demo = false;
    let mut view = View::Cards;
    let mut show: Vec<(String, String)> = Vec::new();
    let mut look = None;
    let mut variant = Variant::Storefront;
    let mut look_given = false;
    let mut it = argv.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--size" => size = parse_size(it.next().ok_or("--size needs WxH")?)?,
            "--snapshot" => snapshot = Some(it.next().ok_or("--snapshot needs a file")?.clone()),
            "--demo" => demo = true,
            // The look: the store wears the Craft style; `--look standard` is the default theme of the platform.
            "--look" => {
                look_given = true;
                look = match it.next().map(String::as_str) {
                    Some("craft") => Some(fs_components_styles::StyleId::Craft),
                    Some("breeze") => Some(fs_components_styles::StyleId::Breeze),
                    Some("standard") => None,
                    _ => return Err("--look needs craft, breeze or standard".to_owned()),
                };
            }
            // The window: `b` the storefront (tiles, table; Craft look), `c` compact (list and page; Breeze look)
            "--variant" => {
                variant = match it.next().map(String::as_str) {
                    Some("b") => Variant::Storefront,
                    Some("c") => Variant::Compact,
                    _ => return Err("--variant needs b or c".to_owned()),
                };
            }
            "--view" => {
                view = match it.next().map(String::as_str) {
                    Some("cards") => View::Cards,
                    Some("sources") => View::Sources,
                    _ => return Err("--view needs cards or sources".to_owned()),
                };
            }
            // The state of the storefront to draw (pictures and tests of the window): `--layout grid|table`,
            // `--section start|program|frontend|component|theme`, `--tab all|installed|updates|new`, `--search TEXT`,
            // `--open NAME` (package page), `--preview NAME` (the question before an install).
            flag @ ("--layout" | "--section" | "--tab" | "--search" | "--open" | "--preview"
            | "--job") => {
                show.push((
                    flag.to_owned(),
                    it.next().ok_or(format!("{flag} needs a value"))?.clone(),
                ));
            }
            // The facet link convention (`fs-<facet>`, ADR 0063) is read by the one shared parser. The store has a
            // single view, so a valid facet is accepted and nothing else changes; a bad one is an error.
            "--facet" => {
                facet_from_args([
                    "--facet".into(),
                    it.next().cloned().unwrap_or_default().into(),
                ])?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if !look_given {
        look = Some(match variant {
            Variant::Storefront => fs_components_styles::StyleId::Craft,
            Variant::Compact => fs_components_styles::StyleId::Breeze,
        });
    }
    fs_components_render::use_style(look);
    let parts = parts(demo);
    let mut model = StoreModel::new(parts.backend).with_updates(parts.updates);
    model.refresh();
    model.show(view);
    for (flag, value) in &show {
        crate::model::apply_show_flag(&mut model, flag, value)?;
    }
    let launcher = parts.launcher;
    let wanted = parts.check_at_start_allowed && model.check_wanted_at_start();
    let shared = Arc::new(Mutex::new(model));
    let app = store_app(&shared, &launcher, variant).map_err(|e| e.to_string())?;
    if let Some(file) = snapshot {
        app.picture(size.0, size.1, std::path::Path::new(&file))
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if wanted {
        start_check(&shared);
    }
    let mut options = WindowOptions::default();
    options.width = size.0;
    options.height = size.1;
    launch(options, app).map_err(|e| e.to_string())
}
