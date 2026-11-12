//! `--demo` is sealed: no catalog over the network, no `updates.toml`, no process. The parts it chooses answer every
//! such call with an error that says "demo mode", and a check at the start is not wanted.

use std::sync::Arc;

use fs_store_app::backend::{JobKind, StoreBackend};
use fs_store_app::demo::parts;
use fs_store_app::model::StoreModel;

#[test]
fn demo_looks_for_no_updates_and_starts_nothing() {
    let p = parts(true);
    assert!(!p.check_at_start_allowed);
    let e = p.updates.check().unwrap_err();
    assert!(e.contains("demo mode"), "{e}");
    assert!(p.updates.timer_set(true).is_err());
    assert!(!p.updates.timer_enabled());
    assert!(!p.launcher.exists("fs-tasks"));
    assert!(
        p.launcher
            .start("fs-tasks")
            .unwrap_err()
            .contains("demo mode")
    );
    assert!(
        p.launcher
            .open_url("https://example.org")
            .unwrap_err()
            .contains("demo mode")
    );
}

#[test]
fn demo_settings_live_in_memory_only() {
    let p = parts(true);
    let mut settings = p.updates.settings().unwrap();
    settings.check = fs_store_sources::CheckEvery::Hourly;
    p.updates.save_settings(&settings).unwrap();
    assert_eq!(p.updates.settings().unwrap(), settings);
    // a second demo starts from nothing: it shared no file with the first
    assert_ne!(
        parts(true).updates.settings().unwrap().check,
        fs_store_sources::CheckEvery::Hourly
    );
}

#[test]
fn demo_installs_nothing_and_the_model_never_wants_a_check_at_start() {
    let p = parts(true);
    let backend: Arc<dyn StoreBackend> = Arc::clone(&p.backend);
    let e = backend
        .run(JobKind::Install, "fs-wiki", &mut |_| {})
        .unwrap_err();
    assert!(format!("{e:?}").contains("demo mode"), "{e:?}");
    assert!(backend.uninstall("fs-tasks").is_err());
    let mut model = StoreModel::new(p.backend).with_updates(p.updates);
    model.refresh();
    // the window starts a check only when `check_at_start_allowed`; it is false, whatever the settings say
    assert!(!p.check_at_start_allowed);
    // the same three as before the run: nothing was installed or removed
    let names: Vec<&str> = model.installed().iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["fs-tasks", "fs-people", "kanidm"]);
}
