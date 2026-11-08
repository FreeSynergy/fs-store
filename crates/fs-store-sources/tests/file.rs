//! Tests of the file-backed source list.
use fs_store_core::{REFERENCE_ID, SourceError, SourceId, SourceRepository};
use fs_store_sources::FileSources;

fn reference() -> SourceId {
    SourceId(REFERENCE_ID.into())
}

#[test]
fn missing_file_gives_the_reference_source() {
    let dir = tempfile::tempdir().unwrap();
    let s = FileSources::load(&dir.path().join("sources.toml")).unwrap();
    assert_eq!(s.list().len(), 1);
    assert!(s.list()[0].reference && s.list()[0].enabled);
}

#[test]
fn changes_survive_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sources.toml");
    let mut s = FileSources::load(&path).unwrap();
    s.add("https://example.org/p.git", SourceId("mine".into()))
        .unwrap();
    s.set_enabled(&reference(), false).unwrap();
    s.save().unwrap();
    let again = FileSources::load(&path).unwrap();
    assert_eq!(again.list(), s.list());
    assert!(!again.list()[0].enabled);
}

#[test]
fn reference_deleted_from_the_file_comes_back_with_its_switch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sources.toml");
    let mut s = FileSources::load(&path).unwrap();
    s.set_enabled(&reference(), false).unwrap();
    s.save().unwrap();
    // Someone removes the reference entry but leaves its disabled mark in place.
    std::fs::write(
        &path,
        "disabled = [\"freesynergy\"]\n\n[[source]]\nid = \"mine\"\nurl = \"u\"\n",
    )
    .unwrap();
    let again = FileSources::load(&path).unwrap();
    assert_eq!(again.list()[0].id, reference());
    assert!(!again.list()[0].enabled);
    // Someone empties the file: the reference is back, enabled.
    std::fs::write(&path, "").unwrap();
    let empty = FileSources::load(&path).unwrap();
    assert!(empty.list()[0].reference && empty.list()[0].enabled);
}

#[test]
fn reference_cannot_be_removed_and_cannot_be_faked() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sources.toml");
    let mut s = FileSources::load(&path).unwrap();
    assert_eq!(s.remove(&reference()), Err(SourceError::ReferenceSource));
    assert_eq!(s.add("evil", reference()), Err(SourceError::Duplicate));
    // A file entry with the reference id cannot change the built-in URL or become a second one.
    std::fs::write(
        &path,
        "[[source]]\nid = \"freesynergy\"\nurl = \"https://evil\"\n",
    )
    .unwrap();
    let again = FileSources::load(&path).unwrap();
    assert_eq!(again.list().len(), 1);
    assert_ne!(again.list()[0].url, "https://evil");
}

#[test]
fn broken_file_is_an_error_not_a_silent_reset() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sources.toml");
    std::fs::write(&path, "[[source]\n").unwrap();
    assert!(FileSources::load(&path).is_err());
}
