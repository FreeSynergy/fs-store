//! A program may leave a background server (`fs-<name> serve`) behind after its window closed. A running
//! program never blocks an update (the person's rule): the store stops the server in order (a request to end, wait
//! for the clean end; the server removes its socket and finishes writes), installs, and starts it again if it
//! ran. Pattern: Memento (what was running is remembered to be started again).
//! Only processes of the same person are touched, and never by force (no SIGKILL, no `taskkill /F`).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fs_store_platform::current;

/// A running `fs-<name> serve`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// Process id.
    pub pid: u32,
    /// The command line: the program, then `serve` and its options.
    pub argv: Vec<String>,
    /// The file it runs from, when known.
    pub exe: Option<PathBuf>,
}

/// True when this command line is `<name> serve ...` (the program is called by its package name).
#[must_use]
pub fn is_server_of(argv: &[String], name: &str) -> bool {
    let program = argv
        .first()
        .and_then(|a| Path::new(a).file_name())
        .and_then(|f| f.to_str());
    program.map(|f| current().paths.command_name(f)).as_deref() == Some(name)
        && argv.get(1).map(String::as_str) == Some("serve")
}

/// The servers of package `name` that run for this person now, not counting the process `ignore`.
#[must_use]
pub fn find_servers(name: &str, ignore: u32) -> Vec<Server> {
    current()
        .processes
        .mine()
        .into_iter()
        .filter(|p| p.pid != ignore && is_server_of(&p.argv, name))
        .map(|p| Server {
            pid: p.pid,
            argv: p.argv,
            exe: p.exe,
        })
        .collect()
}

/// Asks each server to end in order and waits up to `wait` for all of them.
///
/// # Errors
/// A sentence when one did not end in time; nothing is killed by force.
pub fn stop_servers(servers: &[Server], wait: Duration) -> Result<(), String> {
    let processes = &current().processes;
    for s in servers {
        processes.request_stop(s.pid)?;
    }
    let until = Instant::now() + wait;
    while servers.iter().any(|s| processes.is_alive(s.pid)) {
        if Instant::now() > until {
            return Err("a background server did not end in time".to_owned());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

/// Starts the server again with the options it had, in the background. `program` is the file to run (the
/// new version). Returns false when it could not be started.
#[must_use]
pub fn start_server(program: &Path, server: &Server) -> bool {
    current()
        .processes
        .spawn_detached(program, &server.argv[1.min(server.argv.len())..])
        .is_ok()
}

/// Where the server of `name` starts from after an install: the active version below `root` for an unpacked
/// package, else the command as the system finds it.
#[must_use]
pub fn program_of(root: &Path, name: &str, server: &Server) -> PathBuf {
    let command = server
        .argv
        .first()
        .and_then(|a| Path::new(a).file_name())
        .map_or_else(|| PathBuf::from(name), PathBuf::from);
    let unpacked = server
        .exe
        .as_ref()
        .is_some_and(|e| current().paths.is_below(e, &root.join("packages")));
    if unpacked {
        let current = root.join("current").join(name).join(&command);
        if current.exists() {
            return current;
        }
    }
    command
}
