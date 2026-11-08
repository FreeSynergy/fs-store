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
