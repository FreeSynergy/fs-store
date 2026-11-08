use std::fs;
use std::path::Path;
use std::time::Duration;

use fs_store_tuf_publish::{Expiries, generate_keys, publish};

fn setup(catalog: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    generate_keys(tmp.path(), 3, 2, Duration::from_secs(3600)).unwrap();
    fs::write(tmp.path().join("catalog.xml"), catalog).unwrap();
    tmp
}

fn read(dir: &Path) -> Result<Vec<u8>, fs_store_tuf::TufError> {
    let root = fs::read(dir.join("root.json")).unwrap();
    fs_store_tuf::read_target(
        &root,
        &dir.join("tuf/metadata"),
        &dir.join("tuf/targets"),
        "catalog.xml",
    )
}

#[test]
fn published_catalog_is_read_back_through_the_client() {
    let tmp = setup("<catalog/>");
    let v = publish(tmp.path(), &Expiries::default()).unwrap();
    assert_eq!((v.targets, v.snapshot, v.timestamp), (1, 1, 1));
    assert_eq!(read(tmp.path()).unwrap(), b"<catalog/>");
}

#[test]
fn an_expired_timestamp_is_refused() {
    let tmp = setup("<catalog/>");
    let e = Expiries {
        timestamp: Duration::from_secs(1),
        ..Expiries::default()
    };
    publish(tmp.path(), &e).unwrap();
    std::thread::sleep(Duration::from_secs(3));
    assert!(matches!(
        read(tmp.path()),
        Err(fs_store_tuf::TufError::Metadata(_))
    ));
}

#[test]
fn a_catalog_changed_after_publishing_is_refused() {
    let tmp = setup("<catalog/>");
    publish(tmp.path(), &Expiries::default()).unwrap();
    fs::write(tmp.path().join("tuf/targets/catalog.xml"), "<evil/>").unwrap();
    assert!(read(tmp.path()).is_err());
}

#[test]
fn a_republish_bumps_versions_and_carries_the_new_catalog() {
    let tmp = setup("<catalog a/>");
    publish(tmp.path(), &Expiries::default()).unwrap();
    fs::write(tmp.path().join("catalog.xml"), "<catalog b/>").unwrap();
    let v = publish(tmp.path(), &Expiries::default()).unwrap();
    assert_eq!((v.targets, v.snapshot, v.timestamp), (2, 2, 2));
    assert_eq!(read(tmp.path()).unwrap(), b"<catalog b/>");
}

#[test]
fn missing_inputs_are_errors_not_panics() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(publish(tmp.path(), &Expiries::default()).is_err());
}

#[test]
fn a_foreign_root_does_not_trust_the_published_repository() {
    let a = setup("<catalog/>");
    let b = setup("<catalog/>");
    publish(a.path(), &Expiries::default()).unwrap();
    let root_b = fs::read(b.path().join("root.json")).unwrap();
    let r = fs_store_tuf::read_target(
        &root_b,
        &a.path().join("tuf/metadata"),
        &a.path().join("tuf/targets"),
        "catalog.xml",
    );
    assert!(r.is_err());
}
