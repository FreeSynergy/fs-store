//! A program may leave a background server (`fs-<name> serve`) behind after its window closed. A running
//! program never blocks an update (the person's rule): the store stops the server in order (SIGTERM, wait
//! for the clean end; the server removes its socket and finishes writes), installs, and starts it again if it
//! ran. Pattern: Memento (what was running is remembered to be started again).
//! Only processes of the same person are touched, and never with SIGKILL.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
    program == Some(name) && argv.get(1).map(String::as_str) == Some("serve")
}

/// The servers of package `name` that run for this person now, not counting the process `ignore`.
#[must_use]
pub fn find_servers(name: &str, ignore: u32) -> Vec<Server> {
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

fn alive(pid: u32) -> bool {
    // A zombie is gone for our purpose: its state is `Z`.
    fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|s| {
        s.rsplit(')')
            .next()
            .is_some_and(|r| !r.trim_start().starts_with('Z'))
    })
}

/// Asks each server to end (SIGTERM) and waits up to `wait` for all of them.
///
/// # Errors
/// A sentence when one did not end in time; nothing is killed by force.
pub fn stop_servers(servers: &[Server], wait: Duration) -> Result<(), String> {
    for s in servers {
        Command::new("kill")
            .args(["-TERM", &s.pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("kill: {e}"))?;
    }
    let until = Instant::now() + wait;
    while servers.iter().any(|s| alive(s.pid)) {
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
    Command::new(program)
        .args(server.argv.iter().skip(1))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
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
        .is_some_and(|e| e.starts_with(root.join("packages")));
    if unpacked {
        let current = root.join("current").join(name).join(&command);
        if current.exists() {
            return current;
        }
    }
    command
}
