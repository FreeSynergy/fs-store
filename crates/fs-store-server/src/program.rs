//! The server as a command: what `fs-store serve` runs.
//!
//! Pattern: Command (one subcommand of the single `fs-store` program, ADR 0063) — reads its options,
//! serves until a signal. Returns the exit code; the binary exits with it.

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use fs_interfaces::rpc::{default_socket, run_until_signal};

use crate::{LocalStore, StoreServer};

const USAGE: &str = "fs-store serve [--socket PATH]\nDefault: $XDG_RUNTIME_DIR/freesynergy/store.sock\nThe first slice answers reads only (sources, available, installed); writes answer `Unsupported`.";

/// The socket named by `--socket`, or the default; `Ok(None)` when help was asked for.
fn socket_of(args: impl IntoIterator<Item = OsString>) -> Result<Option<PathBuf>, String> {
    let mut args = args.into_iter();
    let mut socket = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help" | "-h") => return Ok(None),
            Some("--socket") => {
                let path = args.next().filter(|p| !p.is_empty());
                socket = Some(PathBuf::from(path.ok_or("--socket needs a path")?));
            }
            _ => return Err(format!("unknown argument {}", arg.display())),
        }
    }
    socket
        .or_else(|| default_socket("store"))
        .map(Some)
        .ok_or_else(|| "no socket: set --socket, or $XDG_RUNTIME_DIR".to_owned())
}

/// Run the server with `args` (the words after `serve`); the exit code: 0 after a signal or `--help`,
/// 2 for a mistake in the arguments, 1 when serving failed.
#[must_use]
pub fn serve_command(args: impl IntoIterator<Item = OsString>) -> i32 {
    let socket = match socket_of(args) {
        Ok(Some(socket)) => socket,
        Ok(None) => {
            println!("{USAGE}");
            return 0;
        }
        Err(problem) => {
            eprintln!("fs-store serve: {problem}");
            return 2;
        }
    };
    let served = run_until_signal(&socket, || async {
        let view: Arc<dyn crate::StoreView> = Arc::new(LocalStore::open_default());
        Ok::<_, io::Error>(StoreServer::new(view).into_client().client)
    });
    match served {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("fs-store serve: {e}");
            1
        }
    }
}
