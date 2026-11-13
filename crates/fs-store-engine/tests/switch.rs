//! A package that was unpacked by the store and becomes an rpm: the commands follow the active form, what the store
//! unpacked is taken away (only that), and `check_links` sees a command that starts another version than the register.

mod common;

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, check_links, record_install, settle_install_with};
use fs_store_sources::Installed;

struct Setup {
    root: PathBuf,
    bin: PathBuf,
    apps: PathBuf,
    system: PathBuf,
}

fn executable(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "#!/bin/sh\n").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// `people` 0.0.2 unpacked by the store, with its commands; an rpm program (not yet installed by the store) in `system`.
fn unpacked(tmp: &Path) -> Setup {
    let s = Setup {
        root: tmp.join("root"),
        bin: tmp.join("bin"),
        apps: tmp.join("apps"),
        system: tmp.join("usr-bin"),
    };
    let a = archive(tmp, "one", "v1");
    run_update(
        &Update {
            package: &verified("people", Version::new(0, 0, 2)),
            previous: None,
        },
        &mut TarGz::new(&s.root, &a),
        &Idle,
    )
    .unwrap();
    executable(&s.root.join("current/people/people-app"));
    executable(&s.root.join("current/people/people-tool"));
    record_install(&s.root, "people", "0.0.2", "src", "sig").unwrap();
    let facets = vec!["contacts".to_owned()];
    let made = settle_install_with(
        &s.root, "people", "0.0.2", &facets, &s.bin, &s.apps, &s.system,
    );
    assert!(made.problems.is_empty(), "{:?}", made.problems);
    s
}

fn entry(version: &str) -> Installed {
    Installed {
        name: "people".to_owned(),
        version: version.to_owned(),
        source: "src".to_owned(),
        auto_update: false,
        facets: vec!["contacts".to_owned()],
    }
}

#[test]
fn facets_follow_the_rpm_after_a_tar_install_and_the_old_files_go() {
    let tmp = tempfile::tempdir().unwrap();
    let s = unpacked(tmp.path());
    assert_eq!(
        fs::read_link(s.bin.join("contacts")).unwrap(),
        s.root.join("current/people/people-app")
    );
    // another package of the store, and a command of somebody else: both stay
    fs::create_dir_all(s.root.join("packages/other/1.0.0")).unwrap();
    fs::write(s.root.join("packages/other/1.0.0/f"), "x").unwrap();
    symlink("/somewhere/else", s.bin.join("mine")).unwrap();

    // the rpm 0.0.7 is installed now
    executable(&s.system.join("people-app"));
    let r = settle_install_with(
        &s.root,
        "people",
        "0.0.7",
        &["contacts".to_owned()],
        &s.bin,
        &s.apps,
        &s.system,
    );
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    assert_eq!(
        fs::read_link(s.bin.join("contacts")).unwrap(),
        s.system.join("people-app")
    );
    assert_eq!(
        fs::read_link(s.bin.join("people")).unwrap(),
        s.system.join("people-app")
    );
    assert!(fs::symlink_metadata(s.bin.join("people-tool")).is_err());
    assert!(fs::symlink_metadata(s.root.join("current/people")).is_err());
    assert!(!s.root.join("packages/people").exists());
    assert!(!s.root.join("manifests/people.toml").exists());
    // the counter-test: what is not ours is untouched
    assert!(s.root.join("packages/other/1.0.0/f").is_file());
    assert_eq!(
        fs::read_link(s.bin.join("mine")).unwrap(),
        Path::new("/somewhere/else")
    );
}

#[test]
fn a_file_of_somebody_else_in_the_old_folder_is_kept_and_said() {
    let tmp = tempfile::tempdir().unwrap();
    let s = unpacked(tmp.path());
    let stranger = s.root.join("packages/people/0.0.2/notes.txt");
    fs::write(&stranger, "mine").unwrap();
    executable(&s.system.join("people-app"));
    let r = settle_install_with(&s.root, "people", "0.0.7", &[], &s.bin, &s.apps, &s.system);
    assert_eq!(r.problems.len(), 1, "{:?}", r.problems);
    assert!(r.problems[0].contains("not ours"));
    assert_eq!(fs::read_to_string(&stranger).unwrap(), "mine");
    // the folder stays as it is (with the list), so nothing is lost and the next try finds the same state
    assert!(s.root.join("manifests/people.toml").is_file());
}

#[test]
fn check_links_sees_a_command_that_starts_the_old_version_and_the_repair_clears_it() {
    let tmp = tempfile::tempdir().unwrap();
    let s = unpacked(tmp.path());
    executable(&s.system.join("people-app"));
    // healthy unpacked install: nothing to say
    let none = check_links(&s.root, &s.bin, &[entry("0.0.2")], &s.system);
    assert_eq!(none.len(), 0, "{none:?}");
    // the register says rpm 0.0.7, the commands still start the unpacked 0.0.2 (the bug as the person saw it)
    let bad = check_links(&s.root, &s.bin, &[entry("0.0.7")], &s.system);
    assert_eq!(bad.len(), 2, "{bad:?}");
    assert!(bad[0].contains("starts version 0.0.2") && bad[0].contains("people 0.0.7"));
    assert!(bad[0].contains("Repair: fs-store relink people"), "{bad:?}");
    // the repair
    let facets = vec!["contacts".to_owned()];
    let r = settle_install_with(
        &s.root, "people", "0.0.7", &facets, &s.bin, &s.apps, &s.system,
    );
    assert_eq!(r.problems.len(), 0, "{:?}", r.problems);
    let none = check_links(&s.root, &s.bin, &[entry("0.0.7")], &s.system);
    assert_eq!(none.len(), 0, "{none:?}");
}

#[test]
fn check_links_sees_a_link_into_the_system_while_the_unpacked_form_is_active() {
    let tmp = tempfile::tempdir().unwrap();
    let s = unpacked(tmp.path());
    executable(&s.system.join("people-app"));
    fs::remove_file(s.bin.join("contacts")).unwrap();
    symlink(s.system.join("people-app"), s.bin.join("contacts")).unwrap();
    let bad = check_links(&s.root, &s.bin, &[entry("0.0.2")], &s.system);
    assert_eq!(bad.len(), 1, "{bad:?}");
    assert!(bad[0].contains("contacts") && bad[0].contains("system program"));
}

#[test]
fn check_links_sees_a_link_to_a_file_that_is_gone() {
    let tmp = tempfile::tempdir().unwrap();
    let s = unpacked(tmp.path());
    fs::remove_file(s.root.join("current/people/people-app")).unwrap();
    let bad = check_links(&s.root, &s.bin, &[entry("0.0.2")], &s.system);
    assert_eq!(bad.len(), 2, "{bad:?}");
    assert!(bad[0].contains("which is gone"));
}
