//! The face of the window (View menu): the entries, the click through the real menu of the window, the saved choice, the override of the
//! command line, a broken or foreign settings file.

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::launcher::Launcher;
use fs_store_app::model::{Command, StoreModel};
use fs_store_app::updates::FileUpdates;
use fs_store_app::view::{menus, store_app};
use fs_store_app::words_front::Lang;
use fs_store_sources::{Face, Settings};

struct NoLauncher;
impl Launcher for NoLauncher {
    fn exists(&self, _: &str) -> bool {
        false
    }
    fn start(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn open_url(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
}

/// A folder of its own below the temporary folder, removed at the end of the test.
struct Dir(std::path::PathBuf);
impl Dir {
    fn new(tag: u32) -> Self {
        let p = std::env::temp_dir().join(format!("fs-store-face-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn model_in(dir: &std::path::Path) -> StoreModel {
    let f = common::fake(Vec::new(), None);
    common::model(&f).with_updates(Arc::new(FileUpdates::new(dir.to_path_buf())))
}

#[test]
fn the_view_menu_lists_every_face_once_and_marks_the_one_worn() {
    for lang in [Lang::En, Lang::De] {
        let all = menus(Face::Compact, lang);
        let view = &all[1];
        assert_eq!(view.entries.len(), Face::ALL.len());
        let ids: Vec<_> = view.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, ["face-storefront", "face-compact"]);
        assert!(view.entries.iter().all(|e| !e.disabled));
        assert!(view.entries[1].label.starts_with('\u{25cf}'));
        assert!(view.entries[0].label.starts_with('\u{25cb}'));
        assert!(view.entries.iter().all(|e| !e.label.contains("{mark}")));
    }
    assert_eq!(menus(Face::Storefront, Lang::En)[1].label, "View");
    assert_eq!(menus(Face::Storefront, Lang::De)[1].label, "Ansicht");
    // every id of the commands is unique, and a face command round-trips
    for c in Command::ALL {
        assert_eq!(Command::from_id(c.id()), Some(c));
    }
}

#[test]
fn a_click_on_a_face_in_the_window_changes_it_at_once_and_remembers_it() {
    let tmp = Dir::new(line!());
    let shared = Arc::new(Mutex::new(model_in(tmp.path())));
    let launcher: Arc<dyn Launcher> = Arc::new(NoLauncher);
    let app = store_app(&shared, &launcher).unwrap();
    // a new install: the storefront
    assert_eq!(shared.lock().unwrap().face(), Face::Storefront);
    assert!(app.choose("face-compact"));
    assert_eq!(shared.lock().unwrap().face(), Face::Compact);
    let saved = Settings::load(&tmp.path().join("settings.toml")).unwrap();
    assert_eq!(saved.face, Face::Compact);
    // the menu still works after it was put in again
    assert!(app.choose("face-storefront"));
    assert_eq!(shared.lock().unwrap().face(), Face::Storefront);
    assert!(!app.choose("face-holo"));
    // a new window (the next start) wears the saved choice
    assert!(app.choose("face-compact"));
    assert_eq!(model_in(tmp.path()).face(), Face::Compact);
}

#[test]
fn the_variant_of_the_command_line_wins_and_is_not_saved_and_a_click_ends_it() {
    let tmp = Dir::new(line!());
    Settings {
        face: Face::Compact,
        ..Settings::default()
    }
    .save(&tmp.path().join("settings.toml"))
    .unwrap();
    let mut m = model_in(tmp.path());
    assert_eq!(m.face(), Face::Compact);
    m.override_face(Some(Face::Storefront));
    assert_eq!(m.face(), Face::Storefront);
    let saved = Settings::load(&tmp.path().join("settings.toml")).unwrap();
    assert_eq!(saved.face, Face::Compact);
    m.set_face(Face::Compact);
    assert_eq!(m.face(), Face::Compact);
}

#[test]
fn a_broken_settings_file_gives_the_default_face_and_a_warning_and_no_crash() {
    let tmp = Dir::new(line!());
    std::fs::write(tmp.path().join("settings.toml"), "face = [[[ not toml").unwrap();
    let m = model_in(tmp.path());
    assert_eq!(m.face(), Face::Storefront);
    assert_ne!(m.problems(), []);
    // a face of a newer store: the default face, the other settings stay
    std::fs::write(
        tmp.path().join("settings.toml"),
        "face = 'holo'\ncheck = 'hourly'",
    )
    .unwrap();
    let m = model_in(tmp.path());
    assert_eq!(m.face(), Face::Storefront);
    assert_eq!(
        m.settings().check,
        fs_store_sources::CheckEvery::Hourly
    );
}
