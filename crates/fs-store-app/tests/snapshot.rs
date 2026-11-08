//! Draws the page of cards to a PNG with a fake backend (no window, no network). Runs only when
//! `FS_STORE_SNAPSHOT=/path/out.png` is set; `FS_STORE_SELECT=<name>` also selects a card.

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::backend::InstalledRow;
use fs_store_app::launcher::Launcher;
use fs_store_app::model::View;
use fs_store_app::view::store_app;

use common::{fake, model, package, v};

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

#[test]
fn picture_of_the_cards() {
    let Some(path) = std::env::var_os("FS_STORE_SNAPSHOT") else {
        return;
    };
    let f = fake(
        vec![
            InstalledRow {
                name: "fs-tasks".into(),
                version: v(0, 6, 0),
            },
            InstalledRow {
                name: "fs-store".into(),
                version: v(0, 2, 0),
            },
            InstalledRow {
                name: "fs-wiki".into(),
                version: v(0, 1, 0),
            },
        ],
        None,
    );
    {
        let mut s = f.state.lock().unwrap();
        for (name, text) in [
            (
                "fs-calendar",
                "Appointments and reminders for you and your people.",
            ),
            ("fs-people", "Contacts, groups and who knows whom."),
            ("fs-wiki", "A small wiki for notes that belong together."),
        ] {
            let mut p = package(name, v(0, 3, 0));
            p.description = text.to_owned();
            p.homepage = format!("https://github.com/FreeSynergy/{name}");
            s.offered.push(p);
        }
    }
    if let Some(icon) = std::env::var_os("FS_STORE_ICON") {
        f.state
            .lock()
            .unwrap()
            .icons
            .push(("fs-wiki".into(), icon.into()));
    }
    let mut m = model(&f);
    if std::env::var_os("FS_STORE_VIEW").is_some_and(|v| v == "settings") {
        m.show(View::Settings);
    }
    if let Some(name) = std::env::var_os("FS_STORE_SELECT") {
        m.select(&name.to_string_lossy());
    }
    let launcher: Arc<dyn Launcher> = Arc::new(NoLauncher);
    let app = store_app(&Arc::new(Mutex::new(m)), &launcher).unwrap();
    app.picture(1000, 700, std::path::Path::new(&path)).unwrap();
}
