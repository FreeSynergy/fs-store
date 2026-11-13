use fs_store_sources::{CheckEvery, Face, Findings, InstallMode, Novelty, Settings, Update};

#[test]
fn defaults_are_daily_ask_and_notify_and_survive_a_restart() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("settings.toml");
    let s = Settings::load(&path).unwrap();
    assert_eq!(
        (s.check, s.install, s.notify),
        (CheckEvery::Daily, InstallMode::Ask, true)
    );
    assert_eq!(s.face, Face::Storefront);
    let s = Settings {
        face: Face::Compact,
        check: CheckEvery::Hourly,
        install: InstallMode::Auto,
        notify: false,
    };
    s.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap(), s);
    // the face is written by name, and no temporary file is left behind
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("face = \"compact\"")
    );
    assert!(!tmp.path().join("settings.toml.new").exists());
}

#[test]
fn an_old_file_without_a_face_and_a_face_of_a_newer_store_give_the_default_face() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("settings.toml");
    std::fs::write(&path, "check = 'hourly'").unwrap();
    let s = Settings::load(&path).unwrap();
    assert_eq!((s.face, s.check), (Face::Storefront, CheckEvery::Hourly));
    // an unknown face does not cost the other settings
    std::fs::write(&path, "face = 'holo'\ncheck = 'hourly'").unwrap();
    let s = Settings::load(&path).unwrap();
    assert_eq!((s.face, s.check), (Face::Storefront, CheckEvery::Hourly));
    assert_eq!(Face::from_id("compact"), Some(Face::Compact));
    assert_eq!(Face::from_id("holo"), None);
}

#[test]
fn findings_are_kept_and_new_programs_stop_being_new_once_seen() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("updates.toml");
    assert_eq!(Findings::load(&path).unwrap().checked, 0);
    let mut f = Findings {
        checked: 5,
        update: vec![Update {
            name: "a".into(),
            from: "1.0.0".into(),
            to: "1.1.0".into(),
        }],
        new: vec![Novelty {
            name: "b".into(),
            version: "0.1.0".into(),
        }],
        offered: vec!["a".into(), "b".into()],
    };
    f.save(&path).unwrap();
    let mut g = Findings::load(&path).unwrap();
    assert_eq!(g, f);
    g.mark_seen();
    assert_eq!((g.new.len(), g.update.len()), (0, 1));
    f.update.clear();
    assert_eq!(f.update.len(), 0);
}

#[test]
fn a_broken_settings_file_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("settings.toml");
    std::fs::write(&path, "check = 'sometimes'").unwrap();
    assert!(Settings::load(&path).is_err());
}
