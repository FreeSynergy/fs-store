mod common;

use std::fs;

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{RemoveError, TarGz, remove};

struct Busy;
impl RunningCheck for Busy {
    fn is_running(&self, _: &str) -> bool {
        true
    }
}

fn installed(tmp: &std::path::Path) -> std::path::PathBuf {
    let root = tmp.join("root");
    let a = archive(tmp, "one", "v1");
    let v = Version::new(1, 0, 0);
    run_update(
        &Update {
            package: &verified("tasks", v),
            previous: None,
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    root
}

#[test]
fn remove_takes_link_and_versions_away() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    remove(&root, "tasks", &Idle).unwrap();
    assert!(fs::symlink_metadata(root.join("current/tasks")).is_err());
    assert!(!root.join("packages/tasks").exists());
    assert_eq!(
        remove(&root, "tasks", &Idle),
        Err(RemoveError::NotInstalled)
    );
}

#[test]
fn a_running_program_is_not_removed_and_bad_names_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    assert_eq!(remove(&root, "tasks", &Busy), Err(RemoveError::Running));
    assert!(root.join("current/tasks/bin/prog").exists());
    assert_eq!(remove(&root, "../x", &Idle), Err(RemoveError::BadName));
}
