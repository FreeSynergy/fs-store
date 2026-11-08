use fs_store_sources::Registry;

#[test]
fn records_keeps_the_auto_update_choice_and_survives_a_restart() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("sub/installed.toml");
    let mut r = Registry::load(&path).unwrap();
    assert_eq!(r.list().len(), 0);
    r.record("tasks", "1.0.0", "https://x/cat");
    assert!(r.set_auto_update("tasks", false));
    assert!(!r.set_auto_update("nope", false));
    r.record("tasks", "1.1.0", "https://x/cat2");
    r.save().unwrap();
    let r = Registry::load(&path).unwrap();
    let t = r.get("tasks").unwrap();
    assert_eq!(
        (t.version.as_str(), t.source.as_str(), t.auto_update),
        ("1.1.0", "https://x/cat2", false)
    );
    let mut r = r;
    r.forget("tasks");
    assert!(r.get("tasks").is_none());
}

#[test]
fn a_broken_file_is_an_error_not_an_empty_register() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("installed.toml");
    std::fs::write(&path, "package = 5").unwrap();
    assert!(Registry::load(&path).is_err());
}

#[test]
fn auto_update_defaults_to_on() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("installed.toml");
    std::fs::write(
        &path,
        "[[package]]\nname='a'\nversion='1.0.0'\nsource='s'\n",
    )
    .unwrap();
    assert!(Registry::load(&path).unwrap().get("a").unwrap().auto_update);
}
