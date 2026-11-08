use std::fs;
use std::path::{Path, PathBuf};

use fs_store_tuf::{TufError, read_target};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/tuf-reference-impl")
}

fn root() -> Vec<u8> {
    fs::read(data().join("metadata/1.root.json")).unwrap()
}

/// A scratch copy that a test may damage.
fn copy() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    for sub in ["metadata", "targets"] {
        fs::create_dir(tmp.path().join(sub)).unwrap();
        for e in fs::read_dir(data().join(sub)).unwrap() {
            let e = e.unwrap();
            fs::copy(e.path(), tmp.path().join(sub).join(e.file_name())).unwrap();
        }
    }
    let (m, t) = (tmp.path().join("metadata"), tmp.path().join("targets"));
    (tmp, m, t)
}

#[test]
fn a_listed_target_is_read_after_the_chain_is_verified() {
    let (_tmp, m, t) = copy();
    assert_eq!(
        read_target(&root(), &m, &t, "file1.txt").unwrap(),
        b"This is an example target file."
    );
}

#[test]
fn a_target_that_is_not_in_the_signed_metadata_is_refused() {
    let (_tmp, m, t) = copy();
    fs::write(t.join("evil.txt"), "x").unwrap();
    assert!(matches!(
        read_target(&root(), &m, &t, "evil.txt"),
        Err(TufError::NotListed(_))
    ));
}

#[test]
fn a_changed_target_file_is_refused() {
    let (_tmp, m, t) = copy();
    fs::write(t.join("file1.txt"), "This is an example target file!").unwrap();
    assert!(matches!(
        read_target(&root(), &m, &t, "file1.txt"),
        Err(TufError::Target(_))
    ));
}

#[test]
fn damaged_metadata_is_refused() {
    let (_tmp, m, t) = copy();
    let ts = fs::read_to_string(m.join("timestamp.json")).unwrap();
    fs::write(
        m.join("timestamp.json"),
        ts.replace("\"version\": 1", "\"version\": 2"),
    )
    .unwrap();
    assert!(matches!(
        read_target(&root(), &m, &t, "file1.txt"),
        Err(TufError::Metadata(_))
    ));
}

#[test]
fn another_root_does_not_trust_the_repository() {
    let (_tmp, m, t) = copy();
    let mut other = root();
    // a root with a different key set: flip one hex digit of a key id inside the trusted root
    let text = String::from_utf8(other.clone()).unwrap();
    let broken = text.replacen("\"keyids\": [", "\"keyids\": [\"00\",", 1);
    other = broken.into_bytes();
    assert!(read_target(&other, &m, &t, "file1.txt").is_err());
}
