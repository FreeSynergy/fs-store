//! Tests of the detection of installed packages: register, `current/` links and rpm together.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use fs_store_app::backend::InstallKind;
use fs_store_app::installed::{details, detect};
use fs_store_core::Version;
use fs_store_sources::Registry;

fn v(a: u64, b: u64, c: u64) -> Version {
    Version::new(a, b, c)
}

fn temp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-store-app-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn no_rpm(_: &str) -> Option<Version> {
    None
}

#[test]
fn all_three_places_are_combined_once_per_name() {
    let root = temp("detect");
    // linked only
    fs::create_dir_all(root.join("packages/linked/1.0.0")).unwrap();
    fs::create_dir_all(root.join("current")).unwrap();
    symlink(
        root.join("packages/linked/1.0.0"),
        root.join("current/linked"),
    )
    .unwrap();
    // registered only, and also linked with an older version
    let mut reg = Registry::load(&root.join("installed.toml")).unwrap();
    reg.record("both", "2.0.0", "https://example.org/s");
    reg.record("regonly", "0.3.0", "https://example.org/s");
    reg.save().unwrap();
    fs::create_dir_all(root.join("packages/both/1.5.0")).unwrap();
    symlink(root.join("packages/both/1.5.0"), root.join("current/both")).unwrap();
    let rpm = |n: &str| (n == "fromrpm").then(|| v(3, 0, 0));
    let rows = detect(
        &root,
        &rpm,
        &["fromrpm".into(), "nothing".into(), "both".into()],
    );
    let got: Vec<_> = rows.iter().map(|r| (r.name.as_str(), r.version)).collect();
    assert_eq!(
        got,
        [
            ("both", v(2, 0, 0)),
            ("fromrpm", v(3, 0, 0)),
            ("linked", v(1, 0, 0)),
            ("regonly", v(0, 3, 0)),
        ]
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn details_list_only_what_exists() {
    let root = temp("details");
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(root.join("packages/tasks/0.6.0")).unwrap();
    fs::create_dir_all(root.join("current")).unwrap();
    symlink(
        root.join("packages/tasks/0.6.0"),
        root.join("current/tasks"),
    )
    .unwrap();
    fs::write(bin.join("tasks"), "").unwrap();
    let mut reg = Registry::load(&root.join("installed.toml")).unwrap();
    reg.record("tasks", "0.6.0", "https://example.org/s");
    reg.set_facets("tasks", vec!["tasks-board".into(), "tasks-gone".into()]);
    reg.save().unwrap();
    fs::write(bin.join("tasks-board"), "").unwrap();
    let d = details(&root, &bin, &no_rpm, InstallKind::Rpm, "tasks");
    assert_eq!(d.kinds, [InstallKind::Archive]);
    assert_eq!(d.source.as_deref(), Some("https://example.org/s"));
    assert_eq!(d.facets, ["tasks-board", "tasks-gone"]);
    let names: Vec<_> = d.commands.iter().map(|c| c.0.as_str()).collect();
    assert_eq!(names, ["tasks", "tasks-board"]);
    assert!(d.folder.unwrap().ends_with("packages/tasks/0.6.0"));
    let rpm_only = details(
        &root,
        &bin,
        &|_| Some(v(1, 0, 0)),
        InstallKind::Rpm,
        "unknown",
    );
    assert_eq!(rpm_only.kinds, [InstallKind::Rpm]);
    assert!(rpm_only.folder.is_none());
    let _ = fs::remove_dir_all(&root);
}
