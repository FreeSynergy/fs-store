//! Ending a server: by its own door (`Control.stop`), and by SIGTERM for a server older than `Control`. The servers are
//! fakes: a real `fs-interfaces` server in a thread, and a plain `serve` without `Control` (an old version).
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use capnp_rpc::new_client;
use fs_interfaces::common_capnp::blob;
use fs_interfaces::rpc::{ServerIdentity, bind_socket, run_until_signal_as, serve};
use fs_store_engine::{Ended, Server, end_server, end_servers, socket_of};

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

/// A server without `Control`, in a thread; `stop` ends it (the test's cleanup; SIGTERM only reaches the process
/// the person sees, which here is `sleep`).
fn old_server(
    socket: &Path,
) -> (
    std::thread::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
) {
    let socket = socket.to_owned();
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
}

fn sleeper() -> Child {
    Command::new("sleep")
        .arg("30")
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
}

fn gone_process() -> u32 {
    let mut done = Command::new("true").spawn().unwrap();
    let pid = done.id();
    done.wait().unwrap();
    pid
}

fn server_of(pid: u32, argv: &[&str]) -> Server {
    Server {
        pid,
        argv: argv.iter().map(|a| (*a).to_owned()).collect(),
        exe: None,
    }
}

fn is_gone(child: &mut Child) -> bool {
    child.try_wait().unwrap().is_some()
}

#[test]
fn a_server_with_control_is_ended_by_its_own_door_not_by_a_signal() {
    let _one = ONE_AT_A_TIME.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let socket: PathBuf = dir.path().join("fake.sock");
    let thread = new_server(&socket);
    wait_listening(&socket);
    // A bystander process: a signal to "the server" would end it, Control must not.
    let mut bystander = sleeper();
    // The server's own process is gone already (the fake lives in this process), so only the ask is under test.
    let gone = server_of(gone_process(), &["fs-fake", "serve"]);
    let ended = end_server(&gone, Some(&socket), Duration::from_secs(5)).unwrap();
    assert_eq!(ended, Ended::Control);
    thread.join().unwrap(); // the fake server finished in order
    assert!(!listening(&socket));
    assert!(!is_gone(&mut bystander), "no signal was sent");
    bystander.kill().unwrap();
    bystander.wait().unwrap();
}

#[test]
fn a_server_older_than_control_gets_sigterm() {
    let _one = ONE_AT_A_TIME.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("old.sock");
    let (thread, stop) = old_server(&socket);
    wait_listening(&socket);
    let mut process = sleeper();
    let server = server_of(process.id(), &["fs-old", "serve"]);
    let reaper = std::thread::spawn(move || {
        process.wait().unwrap();
    });
    let ended = end_server(&server, Some(&socket), Duration::from_secs(5)).unwrap();
    assert_eq!(ended, Ended::Signal);
    reaper.join().unwrap(); // the process ended by the signal
    let _ = stop.send(());
    thread.join().unwrap();
}

#[test]
fn nothing_listening_falls_back_to_the_signal() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("nobody.sock");
    let mut process = sleeper();
    let server = server_of(process.id(), &["fs-nobody", "serve"]);
    let reaper = std::thread::spawn(move || {
        process.wait().unwrap();
    });
    assert_eq!(
        end_server(&server, Some(&socket), Duration::from_secs(5)),
        Ok(Ended::Signal)
    );
    reaper.join().unwrap();
}

#[test]
fn a_server_that_stays_is_an_error_with_words() {
    let dir = tempfile::tempdir().unwrap();
    let mut stubborn = Command::new("sh")
        .args(["-c", "trap '' TERM; while :; do sleep 1; done"])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let server = server_of(stubborn.id(), &["fs-stubborn", "serve"]);
    let socket = dir.path().join("none.sock");
    let e = end_servers(std::slice::from_ref(&server), Duration::from_millis(300));
    let e2 = end_server(&server, Some(&socket), Duration::from_millis(300));
    stubborn.kill().unwrap(); // the test's own cleanup
    stubborn.wait().unwrap();
    assert!(e.unwrap_err().contains("did not end in time"));
    assert!(e2.unwrap_err().contains("did not end in time"));
}

#[test]
fn the_socket_is_named_by_the_command_line_or_the_default() {
    let s = |argv: &[&str]| socket_of(&server_of(1, argv));
    assert_eq!(
        s(&["fs-x", "serve", "--socket", "/tmp/a.sock"]),
        Some(PathBuf::from("/tmp/a.sock"))
    );
    assert_eq!(
        s(&["fs-x", "serve", "--socket=/tmp/b.sock"]),
        Some(PathBuf::from("/tmp/b.sock"))
    );
    // without it: the default place of the program, which depends on the person's runtime folder
    if let Some(path) = s(&["/usr/bin/fs-people", "serve"]) {
        assert!(path.ends_with("people.sock"), "{}", path.display());
    }
}
