//! `fs-store`: one binary for everything (ADR 0063). Pattern: Facade over the crates that stay libraries.
//!
//! ```text
//! fs-store                         the window (feature `gui`, on by default)
//! fs-store [--size WxH] [--snapshot file.png]
//! fs-store sources | available | install <name> | update <name> | installed
//! fs-store serve                   the server (not built yet)
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match dispatch(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("fs-store: {message}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None => window(&[]),
        Some("gui") => window(&args[1..]),
        Some(flag) if flag.starts_with("--") && flag != "--trust-key" => window(args),
        Some("serve") => Err("the server (`fs-store serve`) is not built yet; contract: store.capnp in fs-interfaces".into()),
        Some(_) => fs_store_cli::run(args),
    }
}

#[cfg(feature = "gui")]
fn window(args: &[String]) -> Result<(), String> {
    // The first start puts the store into the person's menu (never over a file that is not ours).
    if !args.iter().any(|a| a == "--snapshot") {
        let _ = fs_store_engine::ensure_own_menu_entry(&fs_store_engine::default_apps_dir());
    }
    fs_store_app::gui::run(args)
}

#[cfg(not(feature = "gui"))]
fn window(_args: &[String]) -> Result<(), String> {
    Err("this build has no window (built with --no-default-features); use a subcommand, e.g. `fs-store installed`".into())
}
