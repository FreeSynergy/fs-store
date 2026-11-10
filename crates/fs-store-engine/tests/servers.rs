#![allow(clippy::assert_is_empty)]
mod common;

use std::process::Command;
use std::time::Duration;

use fs_store_engine::{Server, find_servers, is_server_of, stop_servers};

fn argv(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn only_the_serve_command_of_that_package_is_a_server() {
    assert!(is_server_of(&argv(&["/x/fs-people", "serve"]), "fs-people"));
    assert!(is_server_of(
        &argv(&["fs-people", "serve", "--lease", "f"]),
        "fs-people"
    ));
    assert!(!is_server_of(&argv(&["fs-people"]), "fs-people"));
    assert!(!is_server_of(&argv(&["fs-people", "version"]), "fs-people"));
    assert!(!is_server_of(&argv(&["fs-tasks", "serve"]), "fs-people"));
    assert!(!is_server_of(&[], "fs-people"));
}

fn start(name: &str) -> (tempfile::TempDir, std::process::Child) {
    let dir = tempfile::tempdir().unwrap();
    let program = common::server_copy(dir.path(), name);
    let child = Command::new(program).arg("serve").spawn().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    (dir, child)
}

#[test]
fn a_running_server_is_found_with_its_command_line() {
    let (_dir, mut child) = start("fs-teststop");
    let found: Vec<Server> = find_servers("fs-teststop", 0);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].pid, child.id());
    assert_eq!(found[0].argv.get(1).map(String::as_str), Some("serve"));
    assert_eq!(find_servers("fs-teststop", child.id()).len(), 0);
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(find_servers("fs-teststop", 0).len(), 0);
}

#[cfg(unix)]
#[test]
fn a_running_server_ends_on_a_request_to_end() {
    let (_dir, mut child) = start("fs-teststop2");
    let found = find_servers("fs-teststop2", 0);
    assert_eq!(found.len(), 1);
    let reaper = std::thread::spawn(move || child.wait().unwrap());
    stop_servers(&found, Duration::from_secs(5)).unwrap();
    reaper.join().unwrap();
    assert_eq!(find_servers("fs-teststop2", 0).len(), 0);
}

/// A program without a window cannot be asked to end in order on Windows, and nothing is ended by force: the caller is told.
#[cfg(windows)]
#[test]
fn a_server_without_a_window_is_not_killed_and_the_caller_is_told() {
    let (_dir, mut child) = start("fs-teststop2");
    let found = find_servers("fs-teststop2", 0);
    assert_eq!(found.len(), 1);
    assert!(stop_servers(&found, Duration::from_secs(1)).is_err());
    assert!(child.try_wait().unwrap().is_none(), "it is still running");
    child.kill().unwrap();
    child.wait().unwrap();
}
