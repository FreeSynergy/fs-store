//! Process information behaves the same on every system.
mod common;

use fs_store_platform::current;

#[test]
fn a_running_child_is_seen_with_its_file_and_a_gone_one_is_not_alive() {
    let mut child = common::sleeper();
    let processes = &current().processes;
    let pid = child.id();
    let mine = processes.mine();
    let seen = mine
        .iter()
        .find(|p| p.pid == pid)
        .expect("the child is ours");
    assert_ne!(seen.argv.len(), 0);
    assert!(processes.is_alive(pid));
    assert!(mine.iter().any(|p| p.pid == std::process::id()));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(!processes.is_alive(pid));
}

#[test]
fn this_process_is_alive_and_a_made_up_id_is_not() {
    let processes = &current().processes;
    assert!(processes.is_alive(std::process::id()));
    assert!(!processes.is_alive(u32::MAX - 1));
}
