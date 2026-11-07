//! Tests of the source list: the reference source can be disabled, never removed.
mod common;
use common::MemSources;
use fs_launcher_core::*;

#[test]
fn reference_can_be_disabled_but_not_removed() {
    let mut s = MemSources::new();
    let fs = SourceId("fs".into());
    s.set_enabled(&fs, false).unwrap();
    assert!(!s.list()[0].enabled);
    assert_eq!(s.remove(&fs), Err(SourceError::ReferenceSource));
    assert_eq!(s.list().len(), 1);
    s.set_enabled(&fs, true).unwrap();
    assert!(s.list()[0].enabled);
}

#[test]
fn own_sources_can_be_added_and_removed() {
    let mut s = MemSources::new();
    let id = SourceId("mine".into());
    s.add("https://example.org/pkgs.git", id.clone()).unwrap();
    assert_eq!(s.add("x", id.clone()), Err(SourceError::Duplicate));
    s.remove(&id).unwrap();
    assert_eq!(s.remove(&id), Err(SourceError::NotFound));
}

#[test]
fn version_and_name_are_valid_on_construction() {
    assert!(Version::parse("1.2.3").unwrap() < Version::parse("1.10.0").unwrap());
    assert_eq!(Version::parse("1.2"), None);
    assert_eq!(Version::parse("1.2.3.4"), None);
    assert_eq!(Version::parse("a.b.c"), None);
    assert!(PackageName::new("  ").is_none());
}
