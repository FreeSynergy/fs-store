mod common;

use std::fs;

use common::{Idle, verified};
use fs_store_core::*;
use fs_store_engine::TarGz;

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
    assert!(matches!(r, Err(UpdateError::RolledBack(_))));
    assert!(!root.join("packages/tasks/escaped").exists());
    assert!(!root.join("packages/escaped").exists());
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
        assert!(matches!(r, Err(UpdateError::RolledBack(_))));
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
