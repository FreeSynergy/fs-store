//! Tests of the page of cards: one card per name, its state and buttons, hover and selection, the repository
//! link and the uninstall that asks first.

mod common;

use std::sync::Mutex;

use fs_store_app::backend::{InstalledRow, JobError};
use fs_store_app::launcher::Launcher;
use fs_store_app::model::{
    CardState, OpenWidth, Problem, Working, buttons_for, columns, placeholder,
};

use common::{fake, model, package, v};

struct Opener {
    urls: Mutex<Vec<String>>,
    fail: bool,
}

impl Launcher for Opener {
    fn exists(&self, _: &str) -> bool {
        false
    }
    fn start(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn open_url(&self, url: &str) -> Result<(), String> {
        if self.fail {
            return Err("no opener".into());
        }
        self.urls.lock().unwrap().push(url.to_owned());
        Ok(())
    }
}

fn opener(fail: bool) -> Opener {
    Opener {
        urls: Mutex::new(Vec::new()),
        fail,
    }
}

fn installed(name: &str, version: fs_store_core::Version) -> InstalledRow {
    InstalledRow {
        name: name.into(),
        version,
    }
}

#[test]
fn one_card_per_name_even_when_offered_twice_and_installed() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    {
        let mut s = f.state.lock().unwrap();
        s.offered.push(package("fs-store", v(0, 1, 0)));
        s.installed.push(installed("fs-local-only", v(1, 0, 0)));
    }
    let m = model(&f);
    let names: Vec<String> = m.cards().into_iter().map(|c| c.name).collect();
    assert_eq!(names, ["fs-local-only", "fs-store", "fs-tasks", "fs-theme"]);
}

#[test]
fn card_states_and_button_layout() {
    let f = fake(
        vec![
            installed("fs-tasks", v(0, 6, 0)),
            installed("fs-store", v(0, 2, 0)),
        ],
        None,
    );
    let m = model(&f);
    let by = |n: &str| m.cards().into_iter().find(|c| c.name == n).unwrap();
    let theme = by("fs-theme");
    assert_eq!(theme.state, CardState::NotInstalled);
    assert!(theme.buttons.install && theme.buttons.open.is_none() && !theme.buttons.update);
    let store = by("fs-store");
    assert_eq!(store.state, CardState::Installed);
    assert_eq!(store.buttons.open, Some(OpenWidth::Full));
    assert!(!store.buttons.install && !store.buttons.update);
    let tasks = by("fs-tasks");
    assert_eq!(tasks.state, CardState::UpdateAvailable);
    assert_eq!(tasks.buttons.open, Some(OpenWidth::Half));
    assert!(tasks.buttons.update && !tasks.buttons.install);
    assert_eq!(tasks.version_line(), "0.6.0 \u{2192} 0.6.1");
    assert_eq!(buttons_for(CardState::Installed), store.buttons);
}

#[test]
fn installed_but_not_offered_is_still_a_card() {
    let f = fake(vec![installed("fs-local-only", v(1, 0, 0))], None);
    let m = model(&f);
    let card = m.cards().into_iter().find(|c| c.name == "fs-local-only");
    let card = card.unwrap();
    assert_eq!(card.state, CardState::Installed);
    assert_eq!(card.description_text(), fs_store_app::words::NO_DESCRIPTION);
}

#[test]
fn hover_marks_one_card_and_reports_changes() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.hover("fs-tasks"));
    assert!(!m.hover("fs-tasks"), "nothing changed: no new drawing");
    assert!(
        m.cards()
            .iter()
            .all(|c| c.hovered == (c.name == "fs-tasks"))
    );
    assert!(m.hover("fs-theme"));
    assert!(
        !m.unhover("fs-tasks"),
        "late leave of the old card changes nothing"
    );
    assert!(m.cards().iter().any(|c| c.name == "fs-theme" && c.hovered));
    assert!(m.unhover("fs-theme"));
    assert!(m.cards().iter().all(|c| !c.hovered));
}

#[test]
fn selecting_shows_the_description_or_says_there_is_none() {
    let f = fake(Vec::new(), None);
    f.state.lock().unwrap().offered[0].description = "  Tasks for people.  ".into();
    let mut m = model(&f);
    assert!(m.selected().is_none());
    m.select("fs-tasks");
    let card = m.selected().unwrap();
    assert!(card.selected);
    assert_eq!(card.description_text(), "Tasks for people.");
    m.select("fs-theme");
    assert_eq!(m.selected().unwrap().name, "fs-theme");
    assert_eq!(
        m.selected().unwrap().description_text(),
        fs_store_app::words::NO_DESCRIPTION
    );
    m.select("fs-theme");
    assert!(m.selected().is_none(), "second click hides it");
    m.select("fs-theme");
    m.close_description();
    assert!(m.selected().is_none());
}

#[test]
fn repository_opens_the_homepage_and_failures_are_visible() {
    let f = fake(Vec::new(), None);
    f.state.lock().unwrap().offered[0].homepage = "https://github.com/FreeSynergy/fs-tasks".into();
    let mut m = model(&f);
    let ok = opener(false);
    m.open_repository("fs-tasks", &ok);
    assert_eq!(m.problems(), []);
    assert_eq!(
        *ok.urls.lock().unwrap(),
        ["https://github.com/FreeSynergy/fs-tasks"]
    );
    m.open_repository("fs-theme", &ok);
    assert_eq!(m.problems(), [Problem::NoHomepage("fs-theme".into())]);
    assert!(m.problem_lines()[0].contains("FreeSynergy.Theme"));
    m.open_repository("fs-tasks", &opener(true));
    assert!(matches!(m.problems(), [Problem::RepositoryFailed { .. }]));
    assert!(m.problem_lines()[0].contains("no opener"));
}

#[test]
fn an_address_that_is_not_http_is_not_opened() {
    let f = fake(Vec::new(), None);
    f.state.lock().unwrap().offered[0].homepage = "--help".into();
    let mut m = model(&f);
    let ok = opener(false);
    m.open_repository("fs-tasks", &ok);
    assert!(ok.urls.lock().unwrap().is_empty());
    assert_eq!(m.problems().len(), 1);
}

#[test]
fn uninstall_asks_first_and_cancel_changes_nothing() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    let mut m = model(&f);
    m.ask_uninstall("fs-store");
    assert!(
        m.cards()
            .iter()
            .any(|c| c.name == "fs-store" && c.is_confirming())
    );
    assert!(f.state.lock().unwrap().uninstalls.is_empty(), "not yet");
    m.cancel_uninstall();
    assert!(m.cards().iter().all(|c| !c.is_confirming()));
    m.run_uninstall();
    assert_eq!(f.state.lock().unwrap().uninstalls, Vec::<String>::new());
    assert_eq!(m.installed().len(), 1);
}

#[test]
fn confirmed_uninstall_removes_and_the_card_goes_back_to_install() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    let mut m = model(&f);
    m.ask_uninstall("fs-store");
    let name = m.begin_uninstall().unwrap();
    assert_eq!(name, "fs-store");
    assert!(m.is_busy());
    let working = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-store")
        .unwrap()
        .working;
    assert_eq!(working, Some(Working::Removing));
    let result = m.backend().uninstall(&name);
    m.finish_uninstall(result);
    assert!(!m.is_busy());
    assert_eq!(f.state.lock().unwrap().uninstalls, ["fs-store"]);
    let card = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-store")
        .unwrap();
    assert_eq!(card.state, CardState::NotInstalled);
    assert_eq!(m.problems(), []);
}

#[test]
fn failed_uninstall_is_told_and_the_program_stays() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    f.state.lock().unwrap().uninstall_fails = Some("is running".into());
    let mut m = model(&f);
    m.ask_uninstall("fs-store");
    m.run_uninstall();
    assert!(matches!(m.problems(), [Problem::UninstallFailed { .. }]));
    assert!(m.problem_lines()[0].contains("is running"));
    let card = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-store")
        .unwrap();
    assert_eq!(card.state, CardState::Installed);
}

#[test]
fn nothing_to_uninstall_for_a_program_that_is_not_here() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.ask_uninstall("fs-theme");
    assert!(m.cards().iter().all(|c| !c.is_confirming()));
    assert!(matches!(m.problems(), [Problem::NothingToDo(_)]));
    assert!(m.begin_uninstall().is_none());
}

#[test]
fn nothing_is_asked_or_started_while_a_job_runs() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    let mut m = model(&f);
    m.begin_job("fs-theme").unwrap();
    m.ask_uninstall("fs-store");
    assert!(matches!(m.problems(), [Problem::Busy(_)]));
    let working = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-theme")
        .unwrap()
        .working;
    assert!(matches!(working, Some(Working::Job(..))));
}

#[test]
fn failed_job_stays_visible_as_a_problem_on_the_page() {
    let f = fake(Vec::new(), Some(JobError::Checksum));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert!(matches!(m.problems(), [Problem::JobFailed { .. }]));
    let card = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-theme")
        .unwrap();
    assert_eq!(card.state, CardState::NotInstalled, "back as before");
    assert!(card.working.is_none());
}

#[test]
fn placeholder_is_stable_and_differs_by_name() {
    assert_eq!(placeholder("fs-tasks"), placeholder("fs-tasks"));
    assert_eq!(placeholder("fs-tasks").letter, 'T');
    assert_eq!(placeholder("wiki").letter, 'W');
    assert_ne!(placeholder("fs-tasks").rgb, placeholder("fs-people").rgb);
    assert_eq!(placeholder("").letter, '?');
}

#[test]
fn columns_fit_the_width_and_never_drop_below_one() {
    assert_eq!(columns(1000.0, 240.0, 12.0), 4);
    assert_eq!(columns(504.0, 240.0, 12.0), 2);
    assert_eq!(columns(10.0, 240.0, 12.0), 1);
}
