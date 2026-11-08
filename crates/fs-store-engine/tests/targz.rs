use std::fs;
use std::path::Path;

use fs_store_core::*;
use fs_store_engine::{ProcRunningCheck, TarGz};

fn archive(dir: &Path, file: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(format!("{file}.tar.gz"));
    let enc = flate2::write::GzEncoder::new(
        fs::File::create(&path).unwrap(),
        flate2::Compression::fast(),
    );
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(content.len() as u64);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", content.as_bytes())
        .unwrap();
    b.into_inner().unwrap().finish().unwrap();
    path
}

fn verified(name: &str, v: Version) -> VerifiedPackage {
    let p = Package {
        name: PackageName::new(name).unwrap(),
        kind: Kind::Program,
        version: v,
        format: Format::TarGz,
        source: SourceId(REFERENCE_ID.to_owned()),
    };
    SignaturePolicy::default()
        .admit(
            p,
            SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
        )
        .unwrap()
}

struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

#[test]
fn install_then_update_keeps_old_version_and_switches_current() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let a1 = archive(tmp.path(), "one", "v1");
    let a2 = archive(tmp.path(), "two", "v2");
    let p1 = verified("tasks", Version::new(1, 0, 0));
    let mut s = TarGz::new(&root, &a1);
    run_update(
        &Update {
            package: &p1,
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(root.join("current/tasks/bin/prog")).unwrap(),
        "v1"
    );
    let p2 = verified("tasks", Version::new(1, 1, 0));
    let mut s = TarGz::new(&root, &a2);
    run_update(
        &Update {
            package: &p2,
            previous: Some(Version::new(1, 0, 0)),
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(root.join("current/tasks/bin/prog")).unwrap(),
        "v2"
    );
    assert!(root.join("packages/tasks/1.0.0/bin/prog").exists());
}

#[test]
fn broken_archive_rolls_back_to_previous() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let a1 = archive(tmp.path(), "one", "v1");
    let mut s = TarGz::new(&root, &a1);
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    let bad = tmp.path().join("bad.tar.gz");
    fs::write(&bad, b"not a tarball").unwrap();
    let mut s = TarGz::new(&root, &bad);
    let r = run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 1, 0)),
            previous: Some(Version::new(1, 0, 0)),
        },
        &mut s,
        &Idle,
    );
    assert_eq!(r, Err(UpdateError::RolledBack));
    assert_eq!(
        fs::read_to_string(root.join("current/tasks/bin/prog")).unwrap(),
        "v1"
    );
    assert!(!root.join("packages/tasks/1.1.0").exists());
}

#[test]
fn failed_first_install_leaves_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let bad = tmp.path().join("bad.tar.gz");
    fs::write(&bad, b"nope").unwrap();
    let mut s = TarGz::new(&root, &bad);
    let r = run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    );
    assert_eq!(r, Err(UpdateError::RolledBack));
    assert!(!root.join("current/tasks").exists());
}

#[test]
fn archive_cannot_escape_its_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let path = tmp.path().join("evil.tar.gz");
    let mut raw = Vec::new();
    {
        let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
        let mut b = tar::Builder::new(enc);
        let mut h = tar::Header::new_gnu();
        h.set_size(1);
        h.set_mode(0o644);
        // write the name bytes directly, the builder API refuses `..`
        h.as_old_mut().name[..11].copy_from_slice(b"../escaped\0");
        h.set_cksum();
        b.append(&h, &b"x"[..]).unwrap();
        b.into_inner().unwrap().finish().unwrap();
    }
    fs::write(&path, raw).unwrap();
    let mut s = TarGz::new(&root, &path);
    let r = run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    );
    assert_eq!(r, Err(UpdateError::RolledBack));
    assert!(!root.join("packages/tasks/escaped").exists());
    assert!(!root.join("packages/escaped").exists());
}

#[test]
fn proc_check_sees_no_program_that_is_not_there() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!ProcRunningCheck::new(tmp.path()).is_running("tasks"));
}

#[test]
fn proc_check_sees_a_running_program_and_ignores_the_caller() {
    use std::process::Command;
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("packages/tasks/1.0.0");
    fs::create_dir_all(&dir).unwrap();
    let prog = dir.join("sleeper");
    fs::copy("/usr/bin/sleep", &prog).unwrap();
    let mut child = Command::new(&prog).arg("30").spawn().unwrap();
    assert!(ProcRunningCheck::new(tmp.path()).is_running("tasks"));
    assert!(!ProcRunningCheck::ignoring(tmp.path(), child.id()).is_running("tasks"));
    assert!(!ProcRunningCheck::new(tmp.path()).is_running("other"));
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn a_wrapper_directory_named_after_the_package_is_stripped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let path = tmp.path().join("w.tar.gz");
    let enc = flate2::write::GzEncoder::new(
        fs::File::create(&path).unwrap(),
        flate2::Compression::fast(),
    );
    let mut b = tar::Builder::new(enc);
    for name in ["tasks-1.0.0-linux/", "tasks-1.0.0-linux/tasks"] {
        let mut h = tar::Header::new_gnu();
        if name.ends_with('/') {
            h.set_entry_type(tar::EntryType::Directory);
            h.set_size(0);
            h.set_mode(0o755);
            h.set_cksum();
            b.append_data(&mut h, name, std::io::empty()).unwrap();
        } else {
            h.set_size(1);
            h.set_mode(0o755);
            h.set_cksum();
            b.append_data(&mut h, name, &b"x"[..]).unwrap();
        }
    }
    b.into_inner().unwrap().finish().unwrap();
    let mut s = TarGz::new(&root, &path);
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    assert!(root.join("current/tasks/tasks").exists());
}

fn raw_tar(entries: &[(&str, tar::EntryType, &str, &[u8])]) -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    for (name, kind, link, data) in entries {
        let mut h = tar::Header::new_gnu();
        h.set_entry_type(*kind);
        h.set_size(data.len() as u64);
        h.set_mode(0o755);
        if !link.is_empty() {
            h.set_link_name(link).unwrap();
        }
        h.set_cksum();
        b.append_data(&mut h, name, *data).unwrap();
    }
    b.into_inner().unwrap().finish().unwrap();
    raw
}

#[test]
fn symlink_and_hard_link_entries_are_refused() {
    for kind in [tar::EntryType::Symlink, tar::EntryType::Link] {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tmp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let bytes = raw_tar(&[
            ("lnk", kind, outside.to_str().unwrap(), b""),
            ("lnk/pwned", tar::EntryType::Regular, "", b"x"),
        ]);
        let mut s = TarGz::from_bytes(&tmp.path().join("root"), bytes);
        let r = run_update(
            &Update {
                package: &verified("tasks", Version::new(1, 0, 0)),
                previous: None,
            },
            &mut s,
            &Idle,
        );
        assert_eq!(r, Err(UpdateError::RolledBack));
        assert!(!outside.join("pwned").exists());
    }
}

#[test]
fn archive_made_with_tar_dot_is_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    let bytes = raw_tar(&[
        ("./", tar::EntryType::Directory, "", b""),
        ("./bin/x", tar::EntryType::Regular, "", b"x"),
    ]);
    let root = tmp.path().join("root");
    let mut s = TarGz::from_bytes(&root, bytes);
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut s,
        &Idle,
    )
    .unwrap();
    assert!(root.join("current/tasks/bin/x").exists());
}

#[test]
fn reinstalling_the_same_version_with_a_broken_archive_keeps_the_working_install() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let a1 = archive(tmp.path(), "one", "v1");
    let v = Version::new(1, 0, 0);
    run_update(
        &Update {
            package: &verified("tasks", v),
            previous: None,
        },
        &mut TarGz::new(&root, &a1),
        &Idle,
    )
    .unwrap();
    let bad = tmp.path().join("bad.tar.gz");
    fs::write(&bad, b"nope").unwrap();
    // the version is already there: nothing is unpacked, nothing is deleted
    run_update(
        &Update {
            package: &verified("tasks", v),
            previous: Some(v),
        },
        &mut TarGz::new(&root, &bad),
        &Idle,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(root.join("current/tasks/bin/prog")).unwrap(),
        "v1"
    );
}

#[test]
fn a_relative_root_still_gives_working_links() {
    let tmp = tempfile::tempdir().unwrap();
    let a1 = archive(tmp.path(), "one", "v1");
    let link = tmp.path().join("root");
    let rel = pathdiff(&link);
    run_update(
        &Update {
            package: &verified("tasks", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut TarGz::new(&rel, &a1),
        &Idle,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(link.join("current/tasks/bin/prog")).unwrap(),
        "v1"
    );
}

fn pathdiff(abs: &Path) -> std::path::PathBuf {
    // relative to the current directory of the test process
    let cwd = std::env::current_dir().unwrap();
    let mut up = std::path::PathBuf::new();
    for _ in cwd.components().skip(1) {
        up.push("..");
    }
    up.join(abs.strip_prefix("/").unwrap())
}
