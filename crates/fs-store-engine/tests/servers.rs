#![allow(clippy::assert_is_empty)]
use std::fs;
use std::os::unix::process::CommandExt;
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

#[test]
fn a_running_server_is_found_and_ends_on_sigterm() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("serve"), "sleep 30\n").unwrap();
    // `sh serve` shows up as the command line `fs-teststop serve`
    let mut child = Command::new("sh")
        .arg0("fs-teststop")
        .arg("serve")
        .current_dir(dir.path())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let found: Vec<Server> = find_servers("fs-teststop", 0);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].pid, child.id());
    let reaper = std::thread::spawn(move || child.wait().unwrap());
    stop_servers(&found, Duration::from_secs(5)).unwrap();
    reaper.join().unwrap();
    assert!(find_servers("fs-teststop", 0).is_empty());
}
