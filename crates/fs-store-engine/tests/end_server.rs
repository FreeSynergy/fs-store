//! Ending a server: by its own door (`Control.stop`), and by SIGTERM for a server older than `Control`. The servers are
//! fakes: a real `fs-interfaces` server in a thread, and a plain `serve` without `Control` in a child process (an old
//! version; this test binary started again).
#![cfg(unix)]

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use capnp_rpc::new_client;
use fs_interfaces::common_capnp::blob;
use fs_interfaces::rpc::{ServerIdentity, bind_socket, run_until_signal_as, serve};
use fs_store_engine::{Ended, Server, end_server, end_servers, find_servers, socket_of};

/// `stop_self` is a flag of the process: one server run at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

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

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn listening(socket: &Path) -> bool {
    std::os::unix::net::UnixStream::connect(socket).is_ok()
}

fn wait_listening(socket: &Path) {
    let began = Instant::now();
    while !listening(socket) {
        assert!(began.elapsed() < Duration::from_secs(10), "no server");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A server with `Control`, in a thread; the thread ends when the server does.
fn new_server(socket: &Path) -> std::thread::JoinHandle<()> {
    let socket = socket.to_owned();
    std::thread::spawn(move || {
        run_until_signal_as(&socket, ServerIdentity::new("fs-fake", "0.0.1"), || async {
            let root: blob::Client = new_client(Seven);
            Ok(root.client)
        })
        .unwrap();
    })
}

/// The old server body: serves without `Control` until SIGTERM. Runs only in the child (`FS_OLD_SERVER` is set).
#[test]
fn helper_old_server() {
    let Some(socket) = std::env::var_os("FS_OLD_SERVER") else {
        return;
    };
    let rt = runtime();
    tokio::task::LocalSet::new().block_on(&rt, async {
        let mut term =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let bound = bind_socket(Path::new(&socket)).await.unwrap();
        let root: blob::Client = new_client(Seven);
        serve(bound, root.client, async move {
            term.recv().await;
        })
        .await
        .unwrap();
    });
}

fn spawn_old_server(socket: &Path) -> Child {
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "helper_old_server", "--nocapture"])
        .env("FS_OLD_SERVER", socket)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    wait_listening(socket);
    child
}

/// A process whose command line is `<name> serve`, found by the scan; it ignores SIGTERM when `stubborn`.
fn serve_process(dir: &Path, name: &str, stubborn: bool) -> (Child, Server) {
    let script = if stubborn {
        "trap '' TERM\nwhile :; do sleep 1; done\n"
    } else {
        "sleep 30\n"
    };
    std::fs::write(dir.join("serve"), script).unwrap();
    let child = Command::new("sh")
        .arg0(name)
        .arg("serve")
        .current_dir(dir)
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let found = find_servers(name, 0);
    assert_eq!(found.len(), 1, "{found:?}");
    (child, found.into_iter().next().unwrap())
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
fn a_server_with_control_is_ended_by_its_own_door() {
    let _one = ONE_AT_A_TIME.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let socket: PathBuf = dir.path().join("fake.sock");
    let thread = new_server(&socket);
    wait_listening(&socket);
    let ended = end_server(&gone_server(), Some(&socket), Duration::from_secs(5)).unwrap();
    assert_eq!(ended, Ended::Control);
    thread.join().unwrap(); // the fake server finished in order
    assert!(!listening(&socket));
}

#[test]
fn a_server_older_than_control_gets_sigterm_through_the_owner_of_its_socket() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("old.sock");
    let mut child = spawn_old_server(&socket);
    // The scan's process id is wrong on purpose: only the owner of the socket may be signalled.
    let ended = end_server(&gone_server(), Some(&socket), Duration::from_secs(10)).unwrap();
    assert_eq!(ended, Ended::Signal);
    child.wait().unwrap(); // it ended by the signal
    assert!(!listening(&socket));
}

#[test]
fn this_process_is_never_signalled() {
    let _one = ONE_AT_A_TIME.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("self.sock");
    let (thread, stop) = {
        let socket = socket.clone();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let handle = std::thread::spawn(move || {
            let rt = runtime();
            tokio::task::LocalSet::new().block_on(&rt, async {
                let bound = bind_socket(&socket).await.unwrap();
                let root: blob::Client = new_client(Seven);
                serve(bound, root.client, async {
                    let _ = stopped.await;
                })
                .await
                .unwrap();
            });
        });
        (handle, stop)
    };
    wait_listening(&socket);
    let e = end_server(&gone_server(), Some(&socket), Duration::from_secs(1)).unwrap_err();
    assert!(e.contains("this process"), "{e}");
    let _ = stop.send(());
    thread.join().unwrap();
}

#[test]
fn nothing_listening_falls_back_to_the_signal_of_the_found_process() {
    let dir = tempfile::tempdir().unwrap();
    let (mut child, server) = serve_process(dir.path(), "fs-nobody", false);
    let reaper = std::thread::spawn(move || child.wait().unwrap());
    let socket = dir.path().join("nobody.sock");
    assert_eq!(
        end_server(&server, Some(&socket), Duration::from_secs(5)),
        Ok(Ended::Signal)
    );
    reaper.join().unwrap();
}

#[test]
fn a_process_id_that_changed_meaning_is_not_signalled() {
    let dir = tempfile::tempdir().unwrap();
    let mut other = Command::new("sleep").arg("30").spawn().unwrap();
    let mut server = gone_server();
    server.pid = other.id(); // the id now belongs to a process that is not the server found
    let e = end_server(&server, None, Duration::from_millis(300)).unwrap_err();
    assert!(e.contains("not the server found"), "{e}");
    assert!(other.try_wait().unwrap().is_none(), "no signal was sent");
    other.kill().unwrap();
    other.wait().unwrap();
    drop(dir);
}

#[test]
fn a_server_that_stays_is_an_error_with_words() {
    let dir = tempfile::tempdir().unwrap();
    let (mut stubborn, server) = serve_process(dir.path(), "fs-stubborn", true);
    let e = end_servers(std::slice::from_ref(&server), Duration::from_millis(300));
    stubborn.kill().unwrap(); // the test's own cleanup
    stubborn.wait().unwrap();
    assert!(e.unwrap_err().contains("did not end in time"));
}

#[test]
fn the_socket_is_named_by_the_command_line_or_the_default() {
    let s = |argv: &[&str]| {
        socket_of(&Server {
            pid: 1,
            argv: argv.iter().map(|a| (*a).to_owned()).collect(),
            exe: None,
        })
    };
    assert_eq!(
        s(&["fs-x", "serve", "--socket", "/tmp/a.sock"]),
        Some(PathBuf::from("/tmp/a.sock"))
    );
    assert_eq!(
        s(&["fs-x", "serve", "--socket=/tmp/b.sock"]),
        Some(PathBuf::from("/tmp/b.sock"))
    );
    if let Some(path) = s(&["/usr/bin/fs-people", "serve"]) {
        assert!(path.ends_with("people.sock"), "{}", path.display());
    }
}

/// Something listens and takes the connection but never answers (a stopped or stuck server, or not Cap'n Proto at all).
fn silent_listener(socket: &Path) -> std::thread::JoinHandle<()> {
    let listener = std::os::unix::net::UnixListener::bind(socket).unwrap();
    std::thread::spawn(move || {
        let mut held = Vec::new();
        // keep every connection open and say nothing; ends when a connection with a byte "q" comes (cleanup)
        for stream in listener.incoming().flatten() {
            let mut stream = stream;
            let mut b = [0u8; 1];
            stream
                .set_read_timeout(Some(Duration::from_millis(200)))
                .ok();
            if std::io::Read::read(&mut stream, &mut b).is_ok_and(|n| n == 1 && b[0] == b'q') {
                return;
            }
            held.push(stream);
        }
    })
}

fn stop_silent(socket: &Path, thread: std::thread::JoinHandle<()>) {
    let mut s = std::os::unix::net::UnixStream::connect(socket).unwrap();
    std::io::Write::write_all(&mut s, b"q").unwrap();
    thread.join().unwrap();
}

#[test]
fn a_server_that_never_answers_is_an_error_within_the_limit_not_a_hang() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("silent.sock");
    let thread = silent_listener(&socket);
    let began = Instant::now();
    let info = fs_store_engine::server_info(&socket).unwrap_err();
    assert_eq!(info.kind(), std::io::ErrorKind::TimedOut, "{info}");
    let e = end_server(&gone_server(), Some(&socket), Duration::from_secs(1)).unwrap_err();
    assert!(e.contains("did not answer"), "{e}");
    assert!(
        began.elapsed() < fs_store_engine::CALL_LIMIT * 2 + Duration::from_secs(3),
        "took {:?}",
        began.elapsed()
    );
    stop_silent(&socket, thread);
}
