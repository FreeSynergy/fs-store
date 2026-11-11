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
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use fs_interfaces::common_capnp::control;
use fs_interfaces::rpc::{ServerInfo, connect, default_socket, stop_server};

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

fn own_pid() -> u32 {
    std::process::id()
}

fn alive(pid: u32) -> bool {
    // A zombie is gone for our purpose: its state is `Z`.
    fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|s| {
        s.rsplit(')')
            .next()
            .is_some_and(|r| !r.trim_start().starts_with('Z'))
    })
}

/// SIGTERM to `pid`: never this process, never PID 1 or lower (no process of its own), no external `kill`.
#[cfg(unix)]
fn terminate(pid: u32) -> io::Result<()> {
    if pid <= 1 || pid == own_pid() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the server is this process (or not a process of its own)",
        ));
    }
    let pid = i32::try_from(pid)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a process"))?;
    Ok(rustix::process::kill_process(
        pid,
        rustix::process::Signal::TERM,
    )?)
}

#[cfg(not(unix))]
fn terminate(_pid: u32) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no signal on this system; the Windows replacement follows in the win-port",
    ))
}

/// The process that owns the socket: asked through the socket itself (peer credentials), so it is the one that really serves.
///
/// # Errors
/// Nothing listens, or another user serves the socket (`PermissionDenied`).
#[cfg(unix)]
async fn socket_owner(socket: &Path) -> io::Result<Option<u32>> {
    let stream = tokio::net::UnixStream::connect(socket).await?;
    let cred = stream.peer_cred()?;
    if cred.uid() != rustix::process::geteuid().as_raw() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("{} is served by another user", socket.display()),
        ));
    }
    Ok(cred.pid().and_then(|p| u32::try_from(p).ok()))
}

#[cfg(not(unix))]
async fn socket_owner(_socket: &Path) -> io::Result<Option<u32>> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "win-port"))
}

/// The socket a server listens on: its `--socket PATH`, else the default one of the program (`fs-people` listens on
/// `people.sock`). `None` when neither can be told.
#[must_use]
pub fn socket_of(server: &Server) -> Option<PathBuf> {
    let mut args = server.argv.iter().skip(2);
    while let Some(arg) = args.next() {
        if arg == "--socket" {
            return args.next().map(PathBuf::from);
        }
        if let Some(path) = arg.strip_prefix("--socket=") {
            return Some(PathBuf::from(path));
        }
    }
    let program = Path::new(server.argv.first()?).file_name()?.to_str()?;
    default_socket(program.strip_prefix("fs-").unwrap_or(program))
}

/// How a server ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    /// It answered `Control.stop` and went.
    Control,
    /// It does not know `Control` (an old version) and got SIGTERM.
    Signal,
}

fn block_on<T>(work: impl Future<Output = T>) -> io::Result<T> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    Ok(tokio::task::LocalSet::new().block_on(&runtime, work))
}

fn is_nobody(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
    )
}

/// What the server on `socket` says about itself (`Control.version`); a server that does not know it is the default
/// (protocol 0).
///
/// # Errors
/// Nothing listens there, or the answer cannot be read.
pub fn server_info(socket: &Path) -> io::Result<ServerInfo> {
    block_on(async {
        let connection = connect::<control::Client>(socket).await?;
        match connection.client.version_request().send().promise.await {
            Ok(response) => {
                let r = response.get().map_err(io::Error::other)?;
                let text = |t: capnp::Result<capnp::text::Reader<'_>>| -> io::Result<String> {
                    Ok(t.map_err(io::Error::other)?
                        .to_str()
                        .map_err(io::Error::other)?
                        .to_owned())
                };
                Ok(ServerInfo {
                    program: text(r.get_program())?,
                    version: text(r.get_version())?,
                    protocol: r.get_protocol(),
                })
            }
            Err(e)
                if e.kind == capnp::ErrorKind::Unimplemented
                    || e.extra.contains("not implemented") =>
            {
                Ok(ServerInfo::default())
            }
            Err(e) => Err(io::Error::other(e)),
        }
    })?
}

/// Waits until the socket answers nobody any more and, when the owner is another process, until that is gone too.
fn wait_gone(owner: Option<u32>, socket: &Path, wait: Duration) -> Result<(), String> {
    let until = Instant::now() + wait;
    let owner = owner.filter(|p| *p != own_pid());
    loop {
        let listening = listening(socket);
        if !listening && owner.is_none_or(|p| !alive(p)) {
            return Ok(());
        }
        if Instant::now() > until {
            return Err(format!(
                "a background server did not end in time ({})",
                socket.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Ends one server and waits up to `wait` until it is gone. `socket` is where to ask (see [`socket_of`]).
///
/// # Errors
/// A sentence: the server refused or could not be asked, or did not end in time. Nothing is killed by force, and
/// nothing is passed over in silence.
pub fn end_server(server: &Server, socket: Option<&Path>, wait: Duration) -> Result<Ended, String> {
    let Some(socket) = socket else {
        return signal_scanned(server, wait);
    };
    let owner = match block_on(socket_owner(socket)).map_err(|e| e.to_string())? {
        Ok(owner) => owner,
        Err(e) if is_nobody(&e) => return signal_scanned(server, wait),
        Err(e) => return Err(format!("{}: {e}", socket.display())),
    };
    match block_on(stop_server(socket)).map_err(|e| e.to_string())? {
        Ok(()) => wait_gone(owner, socket, wait).map(|()| Ended::Control),
        // Older than `Control`: only the system can end it, and then the process that owns the socket.
        Err(e) if e.kind() == io::ErrorKind::Unsupported => {
            let pid = owner.ok_or_else(|| {
                format!(
                    "{}: too old to be stopped by request, and its process is unknown: end it by hand",
                    socket.display()
                )
            })?;
            terminate(pid).map_err(|e| format!("process {pid}: {e}"))?;
            wait_gone(Some(pid), socket, wait).map(|()| Ended::Signal)
        }
        Err(e) if is_nobody(&e) => signal_scanned(server, wait),
        Err(e) => Err(format!("{}: {e}", socket.display())),
    }
}

/// The server has no socket we can reach: signal the process the scan found, if its command line is still the one found.
fn signal_scanned(server: &Server, wait: Duration) -> Result<Ended, String> {
    let now = fs::read(format!("/proc/{}/cmdline", server.pid)).unwrap_or_default();
    let now: Vec<String> = now
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect();
    if now != server.argv {
        // Gone already, or the process id belongs to something else now: nothing is signalled.
        return if alive(server.pid) {
            Err(format!(
                "process {} is not the server found any more; not signalled",
                server.pid
            ))
        } else {
            Ok(Ended::Signal)
        };
    }
    terminate(server.pid).map_err(|e| format!("process {}: {e}", server.pid))?;
    let until = Instant::now() + wait;
    while alive(server.pid) {
        if Instant::now() > until {
            return Err("a background server did not end in time".to_owned());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(Ended::Signal)
}

/// Ends every server of `servers` ([`end_server`]), each one even when another failed.
///
/// # Errors
/// All the sentences of the ones that could not be ended, joined.
pub fn end_servers(servers: &[Server], wait: Duration) -> Result<(), String> {
    let failed: Vec<String> = servers
        .iter()
        .filter_map(|s| {
            end_server(s, socket_of(s).as_deref(), wait)
                .err()
                .map(|e| format!("{} (process {}): {e}", s.argv.join(" "), s.pid))
        })
        .collect();
    if failed.is_empty() {
        Ok(())
    } else {
        Err(failed.join("; "))
    }
}

/// Asks each server to end through [`end_server`] (`Control.stop`, SIGTERM for an old one) and waits up to `wait`.
///
/// # Errors
/// A sentence when one did not end in time; nothing is killed by force.
pub fn stop_servers(servers: &[Server], wait: Duration) -> Result<(), String> {
    end_servers(servers, wait)
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

/// True while something answers on the socket (a plain connect, nothing is said).
fn listening(socket: &Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::net::UnixStream::connect(socket).is_ok()
    }
    #[cfg(not(unix))]
    {
        let _ = socket;
        false
    }
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
