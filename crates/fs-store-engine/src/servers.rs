//! A program may leave a background server (`fs-<name> serve`) behind after its window closed. A running
//! program never blocks an update (the person's rule): the store ends the server in order (the server removes its
//! socket and finishes writes), installs, and starts it again only when the install failed.
//! Pattern: Memento (what was running is remembered to be started again).
//! Only processes of the same person are touched, and never with SIGKILL.
//!
//! How a server is ended ([`end_server`]): first by its own door, `Control.stop` over its socket. The process to wait for
//! is the OWNER OF THE SOCKET (peer credentials, `SO_PEERCRED`), not a process found by its command line. A server older
//! than `Control` answers `Unsupported`; then that owner gets SIGTERM (never this process, never PID 1 or lower, no
//! external `kill`). Only a server whose socket cannot be reached at all (still starting, hung) is signalled by the
//! process id the `/proc` scan found, and only while its command line is still the one found.
//! Known gap: a process id can be reused between the check and the signal; a `pidfd` would close it (follow-up, as in
//! `fs-interfaces`). Windows: there is no `/proc`, no peer process id and no SIGTERM here; the replacement follows in
//! the win-port, until then the functions answer "nothing found" or `Unsupported`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A running `fs-<name> serve`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// Process id found by the scan (the one to wait for is the owner of the socket, see [`end_server`]).
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
    program == Some(name) && argv.get(1).map(String::as_str) == Some("serve")
}

/// The servers of package `name` that run for this person now, not counting the process `ignore`.
#[must_use]
#[cfg(unix)]
pub fn find_servers(name: &str, ignore: u32) -> Vec<Server> {
    use std::os::unix::fs::MetadataExt;
    let Ok(me) = fs::metadata("/proc/self").map(|m| m.uid()) else {
        return Vec::new();
    };
    let Ok(procs) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    procs
        .flatten()
        .filter_map(|p| {
            let pid = p.file_name().to_str()?.parse::<u32>().ok()?;
            if pid == ignore || fs::metadata(p.path()).ok()?.uid() != me {
                return None;
            }
            let raw = fs::read(p.path().join("cmdline")).ok()?;
            let argv: Vec<String> = raw
                .split(|b| *b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect();
            is_server_of(&argv, name).then(|| Server {
                pid,
                argv,
                exe: fs::read_link(p.path().join("exe")).ok(),
            })
        })
        .collect()
}

#[cfg(not(unix))]
#[must_use]
pub fn find_servers(_name: &str, _ignore: u32) -> Vec<Server> {
    Vec::new() // Windows: the replacement follows in the win-port
}

pub(crate) fn own_pid() -> u32 {
    std::process::id()
}

pub(crate) fn alive(pid: u32) -> bool {
    // A zombie is gone for our purpose: its state is `Z`.
    fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|s| {
        s.rsplit(')')
            .next()
            .is_some_and(|r| !r.trim_start().starts_with('Z'))
    })
}

/// True while the process of `server` still lives.
#[must_use]
pub fn is_alive(server: &Server) -> bool {
    alive(server.pid)
}

/// Starts the server again with the options it had, in the background. `program` is the file to run (the
/// new version). Returns false when it could not be started.
#[must_use]
pub fn start_server(program: &Path, server: &Server) -> bool {
    let mut command = Command::new(program);
    command
        .args(server.argv.iter().skip(1))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    command.spawn().is_ok()
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
        .is_some_and(|e| e.starts_with(root.join("packages")));
    if unpacked {
        let current = root.join("current").join(name).join(&command);
        if current.exists() {
            return current;
        }
    }
    command
}
