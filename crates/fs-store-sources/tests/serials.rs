use fs_store_sources::{Serials, Stale};

#[test]
fn an_older_serial_is_a_rollback_and_an_expired_catalog_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("serials.toml");
    let mut s = Serials::load(&path).unwrap();
    assert_eq!(s.check("a", Some(5), Some(100), 50), Ok(()));
    s.save().unwrap();
    let mut s = Serials::load(&path).unwrap(); // remembered across runs
    assert_eq!(
        s.check("a", Some(4), Some(100), 50),
        Err(Stale::RolledBack { seen: 5 })
    );
    assert_eq!(s.check("a", Some(5), Some(100), 50), Ok(())); // same serial again is fine
    assert_eq!(s.check("a", Some(6), Some(100), 101), Err(Stale::Expired));
    assert_eq!(s.check("b", Some(1), None, 1_000_000), Ok(())); // sources are separate
    assert_eq!(s.check("c", None, None, 0), Ok(())); // no serial: accepted
}
