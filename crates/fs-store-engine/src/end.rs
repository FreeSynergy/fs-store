//! Ending a server in order. Pattern: Strategy with a fallback (`Control.stop` first, SIGTERM for a server older than
//! `Control`); see the header of `servers.rs` for the rules (socket owner, no `/proc` process id, no external `kill`).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fs_interfaces::common_capnp::control;
use fs_interfaces::rpc::{ServerInfo, connect, default_socket, stop_server};

use crate::servers::{Server, alive, own_pid};

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

/// How long one call to a server (connect, `Control.version`, `Control.stop`) may take. A server that is stopped
/// (SIGSTOP), stuck, or not a Cap'n Proto server at all never answers; the job must not wait for ever
/// (`fs-interfaces` does the same in its handshake).
pub const CALL_LIMIT: Duration = Duration::from_secs(5);

/// `work` within [`CALL_LIMIT`]; else `TimedOut`.
async fn limited<T>(work: impl Future<Output = io::Result<T>>) -> io::Result<T> {
    tokio::time::timeout(CALL_LIMIT, work)
        .await
        .unwrap_or_else(|_| {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "the server did not answer within {} s",
                    CALL_LIMIT.as_secs()
                ),
            ))
        })
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
    block_on(limited(async {
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
    }))?
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
    let owner = match block_on(limited(socket_owner(socket))).map_err(|e| e.to_string())? {
        Ok(owner) => owner,
        Err(e) if is_nobody(&e) => return signal_scanned(server, wait),
        Err(e) => return Err(format!("{}: {e}", socket.display())),
    };
    match block_on(limited(stop_server(socket))).map_err(|e| e.to_string())? {
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
