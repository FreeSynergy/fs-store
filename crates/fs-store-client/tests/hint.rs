use fs_store_client::{UpdateHint, update_hint_in};
use fs_store_sources::{Findings, Settings, Update};

fn found(root: &std::path::Path) {
    Findings {
        checked: 1,
        update: vec![Update {
            name: "fs-tasks".into(),
            from: "0.0.2".into(),
            to: "0.0.3".into(),
        }],
        new: vec![],
        offered: vec![],
    }
    .save(&root.join("updates.toml"))
    .unwrap();
}

#[test]
fn a_program_with_an_update_gets_a_hint_and_others_do_not() {
    let tmp = tempfile::tempdir().unwrap();
    found(tmp.path());
    let hint = update_hint_in(tmp.path(), "fs-tasks").unwrap();
    assert_eq!(
        hint,
        UpdateHint {
            program: "fs-tasks".into(),
            from: "0.0.2".into(),
            to: "0.0.3".into()
        }
    );
    assert!(hint.sentence().contains("0.0.2 -> 0.0.3"));
    assert!(update_hint_in(tmp.path(), "fs-people").is_none());
}

#[test]
fn hints_off_or_nothing_checked_yet_means_no_hint_and_no_failure() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(update_hint_in(tmp.path(), "fs-tasks").is_none()); // no files at all
    found(tmp.path());
    Settings {
        notify: false,
        ..Settings::default()
    }
    .save(&tmp.path().join("settings.toml"))
    .unwrap();
    assert!(update_hint_in(tmp.path(), "fs-tasks").is_none());
    std::fs::write(tmp.path().join("updates.toml"), "[[update]] broken").unwrap();
    Settings {
        notify: true,
        ..Settings::default()
    }
    .save(&tmp.path().join("settings.toml"))
    .unwrap();
    assert!(update_hint_in(tmp.path(), "fs-tasks").is_none()); // unreadable: silent
}
