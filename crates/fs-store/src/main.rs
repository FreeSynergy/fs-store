//! `fs-store`: one binary for everything (ADR 0063). Pattern: Facade over the crates that stay libraries.
//!
//! ```text
//! fs-store                         the window (feature `gui`, on by default)
//! fs-store [--size WxH] [--snapshot file.png]
//! fs-store sources | available | install <name> | update <name> | installed
//! fs-store serve [--socket PATH]  the server: store.capnp on a Unix socket (reads only for now)
//! ```

use std::ffi::OsString;
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

/// True when this process runs as root. The store works in the person's own folders and asks for root itself (`pkexec`)
/// only for an rpm; started as root it would read and write `/root/.local/share/fs-store`, a second, different store.
fn is_root() -> bool {
    fs_store_platform::current().permissions.is_elevated()
}

fn dispatch(args: &[String]) -> Result<(), String> {
    if is_root() && std::env::var_os("FS_STORE_ALLOW_ROOT").is_none() {
        return Err("do not run fs-store as root (sudo): it works in your own folders and asks for the password itself when an rpm needs it. Running as root would use /root/.local/share/fs-store, a different store than yours. (FS_STORE_ALLOW_ROOT=1 overrides.)".into());
    }
    match args.first().map(String::as_str) {
        None => window(&[]),
        Some("version" | "--version" | "-V") => {
            let mode = match fs_store_engine::preferred_format() {
                fs_store_core::Format::Rpm => "installed by rpm: programs come as rpm",
                fs_store_core::Format::Deb => "installed by deb: programs come as deb",
                fs_store_core::Format::Exe => "Windows: programs come as setup.exe",
                _ => "not a system package: programs come as tar.gz",
            };
            println!("fs-store {} ({mode})", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("gui") => window(&args[1..]),
        Some(flag) if flag.starts_with("--") && flag != "--trust-key" => window(args),
        Some("serve") => match fs_store_server::serve_command(args[1..].iter().map(OsString::from))
        {
            0 => Ok(()),
            code => Err(format!("serve ended with code {code}")),
        },
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
