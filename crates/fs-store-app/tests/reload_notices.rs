//! 0.0.16: reload (the real worker flow: ticket, read, finish), the messages with a clock that is handed in, and the question
//! before an install that is the one confirmation.

use std::sync::{Arc, Mutex};

use fs_store_app::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase, SourceRow,
    StoreBackend,
};
use fs_store_app::model::{Severity, StoreModel};
use fs_store_app::words_front::Lang;
use fs_store_app::words_msg as m;
use fs_store_core::{Format, Kind, Version};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Fine,
    SourceDown,
    ListFails,
    Panics,
}

struct Shop {
    mode: Mutex<Mode>,
    offers: Mutex<Vec<PackageRow>>,
    format: Format,
}

fn pkg(name: &str, minor: u64, native: Vec<Format>) -> PackageRow {
    PackageRow {
        name: name.to_owned(),
        title: String::new(),
        kind: Kind::Program,
        version: Version::new(1, minor, 0),
        source: "s".to_owned(),
        description: String::new(),
        homepage: String::new(),
        details: fs_store_core::Details::default(),
        native,
        locked: None,
    }
}

fn shop(format: Format, native: Vec<Format>) -> Arc<Shop> {
    Arc::new(Shop {
        mode: Mutex::new(Mode::Fine),
        offers: Mutex::new(vec![pkg("fs-a", 0, native.clone()), pkg("fs-b", 0, native)]),
        format,
    })
}

impl StoreBackend for Shop {
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
        Ok(vec![SourceRow {
            id: "s".into(),
            url: "https://example.org/s".into(),
            enabled: true,
            reference: true,
        }])
    }
    fn add_source(&self, _: &str) -> Result<SourceRow, BackendError> {
        Err(BackendError::Other("no".into()))
    }
    fn set_enabled(&self, _: &str, _: bool) -> Result<(), BackendError> {
        Ok(())
    }
    fn remove_source(&self, _: &str) -> Result<(), BackendError> {
        Ok(())
    }
    fn available(&self) -> Result<Listing, BackendError> {
        let mode = *self.mode.lock().unwrap();
        match mode {
            Mode::Fine => Ok(Listing {
                packages: self.offers.lock().unwrap().clone(),
                ..Listing::default()
            }),
            Mode::SourceDown => Ok(Listing {
                unreadable: vec![("s".into(), "offline".into())],
                ..Listing::default()
            }),
            Mode::ListFails => Err(BackendError::Other("no route".into())),
            Mode::Panics => panic!("the backend broke"),
        }
    }
    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        Ok(Vec::new())
    }
    fn system_format(&self) -> Format {
        self.format
    }
    fn uninstall(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn run(&self, _: JobKind, _: &str, _: &mut dyn FnMut(Phase)) -> Result<Version, JobError> {
        Ok(Version::new(1, 0, 0))
    }
}

/// The window's way: a ticket, the slow read outside the model, then the result is handed back.
fn reload(model: &mut StoreModel) {
    let ticket = model.begin_read().expect("no read runs");
    assert!(model.is_refreshing());
    assert!(model.begin_read().is_none(), "not twice at once");
    let result = ticket.read();
    assert!(model.finish_read(result));
    assert!(!model.is_refreshing());
}

fn names(model: &StoreModel) -> Vec<String> {
    let mut n: Vec<String> = model
        .available()
        .iter()
        .map(|r| r.package.name.clone())
        .collect();
    n.sort();
    n
}

#[test]
fn a_changed_catalog_shows_after_reload() {
    let s = shop(Format::TarGz, Vec::new());
    let mut model = StoreModel::new(s.clone());
    model.refresh();
    assert_eq!(names(&model), ["fs-a", "fs-b"]);
    // the catalog changes: one program new, one with a newer version
    {
        let mut o = s.offers.lock().unwrap();
        o.push(pkg("fs-c", 0, Vec::new()));
        o[0] = pkg("fs-a", 3, Vec::new());
    }
    reload(&mut model);
    assert_eq!(names(&model), ["fs-a", "fs-b", "fs-c"]);
    let a = model.available().iter().find(|r| r.package.name == "fs-a");
    assert_eq!(a.unwrap().package.version, Version::new(1, 3, 0));
    // counter-proof: without a reload the old catalog stays on screen
    s.offers.lock().unwrap().pop();
    assert_eq!(names(&model).len(), 3);
}

#[test]
fn an_unreachable_source_tells_it_and_keeps_the_old_state() {
    let s = shop(Format::TarGz, Vec::new());
    let mut model = StoreModel::new(s.clone());
    model.refresh();
    *s.mode.lock().unwrap() = Mode::SourceDown;
    reload(&mut model);
    assert_eq!(names(&model), ["fs-a", "fs-b"]);
    assert_eq!(
        model.problem_lines(),
        ["The source s could not be read: offline"]
    );
    // the same with a catalog read that fails as a whole
    *s.mode.lock().unwrap() = Mode::ListFails;
    reload(&mut model);
    assert_eq!(names(&model), ["fs-a", "fs-b"]);
    assert_eq!(
        model.problem_lines(),
        ["The store could not do this: no route"]
    );
    // counter-proof: when the source is back, the message is gone and the news show
    *s.mode.lock().unwrap() = Mode::Fine;
    s.offers.lock().unwrap().push(pkg("fs-c", 0, Vec::new()));
    reload(&mut model);
    assert!(model.problem_lines().is_empty());
    assert_eq!(names(&model).len(), 3);
}

#[test]
fn a_broken_read_does_not_leave_the_reload_dead() {
    let s = shop(Format::TarGz, Vec::new());
    let mut model = StoreModel::new(s.clone());
    model.refresh();
    *s.mode.lock().unwrap() = Mode::Panics;
    reload(&mut model);
    assert_eq!(names(&model), ["fs-a", "fs-b"]);
    assert_eq!(model.problem_lines().len(), 1);
    // counter-proof: the next reload works
    *s.mode.lock().unwrap() = Mode::Fine;
    reload(&mut model);
    assert!(model.problem_lines().is_empty());
}

#[test]
fn messages_fade_by_the_clock_can_be_closed_and_stay_in_the_history() {
    let s = shop(Format::TarGz, Vec::new());
    let mut model = StoreModel::new(s.clone());
    model.refresh();
    *s.mode.lock().unwrap() = Mode::ListFails;
    reload(&mut model);
    model.sync_notices(1000);
    let shown = model.visible_notices(1000);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].severity, Severity::Error);
    // an error stays longer than a hint, and goes by itself
    assert_eq!(model.visible_notices(1000 + 19).len(), 1);
    assert!(model.visible_notices(1000 + 20).is_empty());
    assert_eq!(model.notice_history().len(), 1);
    // a hint goes after a few seconds
    model.ask_uninstall("nothing-here");
    model.sync_notices(2000);
    assert!(
        model
            .visible_notices(2000)
            .iter()
            .any(|n| n.severity == Severity::Info)
    );
    assert!(
        model
            .visible_notices(2000 + 6)
            .iter()
            .all(|n| n.severity != Severity::Info)
    );
    // every message can be closed; the history keeps it
    model.sync_notices(2001);
    let id = model.visible_notices(2001)[0].id;
    model.close_notice(id);
    assert!(model.visible_notices(2001).iter().all(|n| n.id != id));
    assert!(model.notice_history().iter().any(|n| n.id == id));
    // the same failure again after another reload is a new message (and shows again)
    let before = model.notice_history().len();
    *s.mode.lock().unwrap() = Mode::ListFails;
    reload(&mut model);
    model.sync_notices(3000);
    assert_eq!(model.notice_history().len(), before + 1);
    assert_eq!(model.visible_notices(3000).len(), 1);
    // counter-proof: without a new problem nothing new appears
    model.sync_notices(3001);
    assert_eq!(model.notice_history().len(), before + 1);
}

#[test]
fn the_question_is_the_one_confirmation_and_says_where_how_big_and_which_rights() {
    let lang = Lang::current();
    let s = shop(Format::TarGz, Vec::new());
    let mut model = StoreModel::new(s);
    model.refresh();
    model.ask_install("fs-a");
    let p = model.preview().expect("the question");
    assert!(p.facts.contains("https://example.org/s"), "{}", p.facts);
    assert!(p.facts.contains(m::rights_own(lang)));
    assert!(p.notice.is_none(), "no password for an install in the own folder");
    assert_eq!(p.confirm_label(), m::install_now(lang));
    // one "yes" starts the job at once: no second question, no card waiting for an answer
    let _ = model.take_preview();
    assert!(model.begin_job("fs-a").is_some());
    assert!(model.preview().is_none());
    assert!(model.problem_lines().is_empty());
    assert!(model.cards().iter().all(|c| !c.is_confirming()));
}

#[test]
fn a_system_install_announces_the_password_in_the_question() {
    let lang = Lang::current();
    let s = shop(Format::Rpm, vec![Format::Rpm]);
    let mut model = StoreModel::new(s);
    model.refresh();
    model.ask_install("fs-a");
    let p = model.preview().expect("the question");
    assert_eq!(p.notice.as_deref(), Some(m::password_notice(lang)));
    assert!(p.facts.contains(m::rights_system(lang)));
    // counter-proof: a package that comes in no native form has no such announcement, even on an rpm system
    let s = shop(Format::Rpm, Vec::new());
    let mut model = StoreModel::new(s);
    model.refresh();
    model.ask_install("fs-a");
    assert!(model.preview().unwrap().notice.is_none());
}
