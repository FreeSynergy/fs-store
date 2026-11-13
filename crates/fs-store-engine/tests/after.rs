//! The follow-up of an install and of a removal says every problem and does the same in the window and the commands.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, settle_install, settle_remove};

fn installed(tmp: &Path) -> PathBuf {
    let root = tmp.join("root");
    let a = archive(tmp, "one", "v1");
    run_update(
        &Update {
            package: &verified("people", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    fs::write(root.join("current/people/people-app"), "#!/bin/sh\n").unwrap();
    let tool = root.join("current/people/people-tool");
    fs::write(&tool, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).unwrap();
    root
}

fn dirs(tmp: &Path) -> (PathBuf, PathBuf) {
    (tmp.join("bin"), tmp.join("apps"))
}

#[test]
fn a_good_install_reports_what_it_made_and_no_problem() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = dirs(tmp.path());
    let s = settle_install(
        &root,
        "people",
        "1.0.0",
        &["contacts".to_owned()],
        &bin,
        &apps,
    );
    assert_eq!(s.made, ["people", "contacts"]);
    assert_eq!(s.linked, ["people-tool"]);
    assert_eq!(s.problems.len(), 0, "{:?}", s.problems);
}

#[test]
fn a_bin_folder_that_cannot_be_made_is_said_twice_not_swallowed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = dirs(tmp.path());
    fs::write(&bin, "a file where the folder should be").unwrap();
    let s = settle_install(&root, "people", "1.0.0", &[], &bin, &apps);
    assert!(s.made.is_empty() && s.linked.is_empty());
    assert_eq!(s.problems.len(), 2, "{:?}", s.problems);
    assert!(s.problems[0].contains("facet commands"));
    assert!(s.problems[1].contains("commands not linked"));
}

#[test]
fn a_foreign_file_in_the_way_is_said() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = dirs(tmp.path());
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("people-tool"), "mine").unwrap();
    let s = settle_install(&root, "people", "1.0.0", &[], &bin, &apps);
    assert_eq!(s.linked.len(), 0);
    assert_eq!(s.problems.len(), 1);
    assert!(s.problems[0].contains("people-tool") && s.problems[0].contains("in the way"));
    assert_eq!(fs::read_to_string(bin.join("people-tool")).unwrap(), "mine");
}

#[test]
fn a_package_of_the_package_manager_gets_no_program_links_and_no_false_problem() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (bin, apps) = dirs(tmp.path());
    let s = settle_install(&root, "people", "1.0.0", &[], &bin, &apps);
    assert_eq!(s.linked.len(), 0);
    assert_eq!(s.problems.len(), 0, "{:?}", s.problems);
}

#[test]
fn a_removal_that_cannot_take_a_link_away_says_so() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = dirs(tmp.path());
    let facets = vec!["contacts".to_owned()];
    let s = settle_install(&root, "people", "1.0.0", &facets, &bin, &apps);
    assert_eq!(s.problems.len(), 0);
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o555)).unwrap();
    let problems = settle_remove(&root, "people", &facets, &bin, &apps);
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        !problems.is_empty(),
        "removing from a read-only folder must be reported"
    );
    assert!(problems.iter().all(|p| p.starts_with("people:")));
    let again = settle_remove(&root, "people", &facets, &bin, &apps);
    assert_eq!(again.len(), 0, "{again:?}");
}
