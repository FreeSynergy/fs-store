//! The last step of a job that installed: "end server". Pattern: Template method step of `Installer::commit_observed`.
//!
//! The servers of the program were ended before the files were replaced (a running program is never replaced).
//! This step looks again after the install: a server that runs now and is NOT the new version (one a window started in
//! between from the old files, or one that was not found before) is ended too (`Control.stop`, SIGTERM for an old one).
//! A server that already says the new version (the window of the program started it meanwhile) is left alone. The new
//! version is not started here: the window of the program starts it when needed (`connect_matching`).

use std::path::PathBuf;
use std::time::Duration;

use fs_interfaces::rpc::FINISH_TIMEOUT;
use fs_store_core::Version;

use crate::service::InstallFailure;
use crate::{Server, end_server, find_servers, server_info, socket_of};

/// How long one server gets to end: longer than the time a busy server takes to finish its work
/// ([`FINISH_TIMEOUT`] of `fs-interfaces`), so "busy" is not taken for "did not end".
pub(crate) const END_WAIT: Duration = Duration::from_secs(FINISH_TIMEOUT.as_secs() + 5);

/// Ends every server in `servers` that is not at `version` (a server that cannot say its version counts as old).
/// `socket_for` tells where to ask. Returns the sentences of the ones that could not be ended.
pub(crate) fn end_unless_current(
    servers: &[Server],
    version: Version,
    wait: Duration,
    socket_for: &dyn Fn(&Server) -> Option<PathBuf>,
) -> Vec<String> {
    let current = version.to_string();
    let mut failed = Vec::new();
    for server in servers {
        let socket = socket_for(server);
        if let Some(info) = socket.as_deref().and_then(|s| server_info(s).ok())
            && info.protocol != 0
            && info.version == current
        {
            continue; // the new server: leave it alone
        }
        if let Err(e) = end_server(server, socket.as_deref(), wait) {
            failed.push(format!(
                "{} (process {}): {e}",
                server.argv.join(" "),
                server.pid
            ));
        }
    }
    failed
}

/// Ends every old server of `name` that still runs after a good install. A failure is never silent: the result becomes
/// `ServerNotEnded` (installed, but ...). A result that is already an error is left alone.
pub(crate) fn end_old_servers(
    name: &str,
    wait: Duration,
    result: &mut Result<Version, InstallFailure>,
) {
    let Ok(version) = *result else { return };
    let servers = find_servers(name, std::process::id());
    if servers.is_empty() {
        return;
    }
    let failed = end_unless_current(&servers, version, wait, &socket_of);
    if !failed.is_empty() {
        *result = Err(InstallFailure::ServerNotEnded {
            version,
            reason: failed.join("; "),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use super::*;

    #[test]
    fn a_busy_server_gets_longer_than_it_needs_to_finish() {
        assert!(END_WAIT > FINISH_TIMEOUT);
    }

    #[test]
    fn nothing_running_changes_nothing() {
        let mut result = Ok(Version::new(1, 0, 0));
        end_old_servers("fs-ending-none", Duration::from_millis(200), &mut result);
        assert_eq!(result, Ok(Version::new(1, 0, 0)));
    }

    #[test]
    fn an_error_is_left_alone() {
        let mut result = Err(InstallFailure::Checksum);
        end_old_servers("fs-ending-none", Duration::from_millis(200), &mut result);
        assert_eq!(result, Err(InstallFailure::Checksum));
    }

    #[test]
    fn a_server_that_will_not_end_is_told_never_silent() {
        // `sh serve` shows up as the command line `fs-ending-stubborn serve`; the script ignores SIGTERM.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("serve"),
            "trap '' TERM\nwhile :; do sleep 1; done\n",
        )
        .unwrap();
        let mut child = Command::new("sh")
            .arg0("fs-ending-stubborn")
            .arg("serve")
            .current_dir(dir.path())
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let mut result = Ok(Version::new(1, 2, 3));
        end_old_servers(
            "fs-ending-stubborn",
            Duration::from_millis(300),
            &mut result,
        );
        child.kill().unwrap(); // the test's own cleanup; the store never kills
        child.wait().unwrap();
        match result {
            Err(InstallFailure::ServerNotEnded { version, reason }) => {
                assert_eq!(version, Version::new(1, 2, 3));
                assert!(reason.contains("did not end in time"), "{reason}");
            }
            other => panic!("expected ServerNotEnded, got {other:?}"),
        }
    }

    // --- fake servers: a real `fs-interfaces` server in a thread (its process is this test process) ---

    use std::path::Path;
    use std::time::Instant;

    use capnp_rpc::new_client;
    use fs_interfaces::common_capnp::blob;
    use fs_interfaces::rpc::{ServerIdentity, run_until_signal_as, stop_server};

    struct Seven;
    impl blob::Server for Seven {
        #[allow(clippy::unused_async_trait_impl)]
        async fn size(
            self: capnp::capability::Rc<Self>,
            _: blob::SizeParams,
            mut results: blob::SizeResults,
        ) -> Result<(), capnp::Error> {
            results.get().set_bytes(7);
            Ok(())
        }
    }

    /// `stop_self` is a flag of the process: one server run at a time.
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn listening(socket: &Path) -> bool {
        std::os::unix::net::UnixStream::connect(socket).is_ok()
    }

    fn start(socket: &Path, version: &'static str) -> std::thread::JoinHandle<()> {
        let socket = socket.to_owned();
        let handle = std::thread::spawn({
            let socket = socket.clone();
            move || {
                run_until_signal_as(&socket, ServerIdentity::new("fs-fake", version), || async {
                    let root: blob::Client = new_client(Seven);
                    Ok(root.client)
                })
                .unwrap();
            }
        });
        let began = Instant::now();
        while !listening(&socket) {
            assert!(began.elapsed() < Duration::from_secs(10), "no server");
            std::thread::sleep(Duration::from_millis(20));
        }
        handle
    }

    fn gone_server() -> Server {
        let mut done = Command::new("true").spawn().unwrap();
        let pid = done.id();
        done.wait().unwrap();
        Server {
            pid,
            argv: vec!["fs-fake".into(), "serve".into()],
            exe: None,
        }
    }

    #[test]
    fn a_new_server_that_is_running_already_is_left_alone() {
        let _one = ONE_AT_A_TIME.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("new.sock");
        let thread = start(&socket, "1.0.0");
        let path = socket.clone();
        let failed = end_unless_current(
            &[gone_server()],
            Version::new(1, 0, 0),
            Duration::from_secs(5),
            &move |_| Some(path.clone()),
        );
        assert!(failed.is_empty(), "{failed:?}");
        assert!(listening(&socket), "the new server must still run");
        // clean up the way the store would end it
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        tokio::task::LocalSet::new()
            .block_on(&rt, stop_server(&socket))
            .unwrap();
        thread.join().unwrap();
    }

    #[test]
    fn an_old_server_is_ended() {
        let _one = ONE_AT_A_TIME.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("old.sock");
        let thread = start(&socket, "0.9.0");
        let path = socket.clone();
        let failed = end_unless_current(
            &[gone_server()],
            Version::new(1, 0, 0),
            Duration::from_secs(5),
            &move |_| Some(path.clone()),
        );
        assert!(failed.is_empty(), "{failed:?}");
        thread.join().unwrap(); // it finished in order
        assert!(!listening(&socket));
    }
}
