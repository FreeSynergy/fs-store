//! The window really builds and draws (the shell with the new strip of messages, the history page, the question and the start
//! overview), to a picture file in the temp folder; no window is opened on a desktop, no network, no file of the person.

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::launcher::Launcher;
use fs_store_app::model::{Section, View};
use fs_store_app::view::store_app;

use common::{fake, model};

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

fn draw(m: fs_store_app::model::StoreModel, name: &str) {
    let launcher: Arc<dyn Launcher> = Arc::new(NoLauncher);
    let app = store_app(&Arc::new(Mutex::new(m)), &launcher).unwrap();
    let path =
        std::env::temp_dir().join(format!("fs-store-0016-{}-{name}.png", std::process::id()));
    app.picture(900, 640, &path).unwrap();
    let size = std::fs::metadata(&path).unwrap().len();
    let _ = std::fs::remove_file(&path);
    assert!(size > 1000, "{name}: the picture is empty");
}

#[test]
fn the_start_the_question_the_strip_and_the_history_all_draw() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    // the start overview
    m.open_section(Section::Start);
    // a message on the strip, and the question
    m.report_failure("the catalog could not be read");
    m.ask_install("fs-tasks");
    draw(m, "start");
    // the history page
    let mut m = model(&f);
    m.report_failure("something failed");
    m.sync_notices(1);
    m.show(View::Messages);
    draw(m, "history");
}
