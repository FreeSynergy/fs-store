mod common;

use std::fs;

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, link_programs, unlink_programs};

fn installed(tmp: &std::path::Path) -> std::path::PathBuf {
    let root = tmp.join("root");
    let a = archive(tmp, "one", "v1"); // bin/prog only: a directory, no executable at the top
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    // an executable at the top, as the release archives have
    let top = root.join("current/tasks/tasks-app");
    fs::write(&top, "#!/bin/sh\necho hi\n").unwrap();
    fs::set_permissions(&top, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    root
}

#[test]
fn executables_are_linked_through_current_and_removed_again() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let bin = tmp.path().join("bin");
    let (linked, skipped) = link_programs(&root, "tasks", &bin).unwrap();
    assert_eq!((linked, skipped), (vec!["tasks-app".to_owned()], vec![]));
    assert_eq!(
        fs::read_to_string(bin.join("tasks-app")).unwrap(),
        "#!/bin/sh\necho hi\n"
    );
    assert_eq!(unlink_programs(&root, "tasks", &bin).unwrap(), 1);
    assert!(fs::symlink_metadata(bin.join("tasks-app")).is_err());
}

#[test]
fn a_foreign_file_with_the_same_name_is_never_replaced() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let bin = tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    fs::write(bin.join("tasks-app"), "mine").unwrap();
    let (linked, skipped) = link_programs(&root, "tasks", &bin).unwrap();
    assert_eq!(linked.len(), 0);
    assert_eq!(skipped, vec!["tasks-app".to_owned()]);
    assert_eq!(fs::read_to_string(bin.join("tasks-app")).unwrap(), "mine");
    assert_eq!(unlink_programs(&root, "tasks", &bin).unwrap(), 0);
}
