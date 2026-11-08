//! Tests of the admin page in the view-model: the timer, the per-program switches, success messages that go,
//! the logo (with fakes; no process, no file).
#![allow(clippy::assert_is_empty)]

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::backend::InstalledRow;
use fs_store_app::model::{StoreModel, View, placeholder};
use fs_store_app::updates::{Program, UpdateService};
use fs_store_sources::{Findings, Settings};

use common::{fake, model, v};

struct Admin {
    timer: Mutex<bool>,
    timer_error: Mutex<Option<String>>,
    output: Mutex<String>,
    programs: Mutex<Vec<Program>>,
    settings: Mutex<Settings>,
}

fn admin() -> Arc<Admin> {
    Arc::new(Admin {
        timer: Mutex::new(false),
        timer_error: Mutex::new(None),
        output: Mutex::new(String::new()),
        programs: Mutex::new(vec![
            Program {
                name: "fs-store".into(),
                auto_update: true,
            },
            Program {
                name: "fs-tasks".into(),
                auto_update: false,
            },
        ]),
        settings: Mutex::new(Settings::default()),
    })
}

impl UpdateService for Admin {
    fn settings(&self) -> Result<Settings, String> {
        Ok(*self.settings.lock().unwrap())
    }
    fn save_settings(&self, s: &Settings) -> Result<(), String> {
        *self.settings.lock().unwrap() = *s;
        Ok(())
    }
    fn findings(&self) -> Result<Findings, String> {
        Ok(Findings::default())
    }
    fn save_findings(&self, _: &Findings) -> Result<(), String> {
        Ok(())
    }
    fn check(&self) -> Result<(), String> {
        Ok(())
    }
    fn now(&self) -> u64 {
        0
    }
    fn timer_enabled(&self) -> bool {
        *self.timer.lock().unwrap()
    }
    fn timer_set(&self, on: bool) -> Result<String, String> {
        if let Some(e) = self.timer_error.lock().unwrap().clone() {
            return Err(e);
        }
        *self.timer.lock().unwrap() = on;
        Ok(self.output.lock().unwrap().clone())
    }
    fn programs(&self) -> Result<Vec<Program>, String> {
        Ok(self.programs.lock().unwrap().clone())
    }
    fn set_auto_update(&self, name: &str, on: bool) -> Result<(), String> {
        let mut list = self.programs.lock().unwrap();
        let p = list
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("{name} is not in the register"))?;
        p.auto_update = on;
        Ok(())
    }
}

fn with(svc: &Arc<Admin>) -> StoreModel {
    let f = fake(
        vec![InstalledRow {
            name: "fs-store".into(),
            version: v(0, 1, 0),
        }],
        None,
    );
    let mut m = StoreModel::new(f).with_updates(svc.clone());
    m.refresh();
    m
}

#[test]
fn the_page_reads_timer_and_programs() {
    let svc = admin();
    *svc.timer.lock().unwrap() = true;
    let m = with(&svc);
    assert!(m.timer_on());
    assert_eq!(m.programs().len(), 2);
    assert!(m.programs()[0].auto_update && !m.programs()[1].auto_update);
}

#[test]
fn the_timer_is_switched_and_its_output_is_a_note() {
    let svc = admin();
    *svc.output.lock().unwrap() = "Created symlink".into();
    let mut m = with(&svc);
    m.set_timer(true);
    assert!(m.timer_on());
    assert_eq!(m.admin_note(), Some("Created symlink"));
    assert!(m.problems().is_empty());
    m.set_timer(false);
    assert!(!m.timer_on());
}

#[test]
fn an_empty_output_still_says_something() {
    let svc = admin();
    let mut m = with(&svc);
    m.set_timer(true);
    assert!(m.admin_note().is_some_and(|n| !n.is_empty()));
}

#[test]
fn a_timer_error_is_a_problem_that_stays_after_a_view_change() {
    let svc = admin();
    *svc.timer_error.lock().unwrap() = Some("Failed to connect to bus".into());
    let mut m = with(&svc);
    m.set_timer(true);
    assert!(!m.timer_on());
    assert!(m.admin_note().is_none());
    assert!(m.problem_lines()[0].contains("Failed to connect to bus"));
    m.show(View::Settings);
    m.show(View::About);
    assert_eq!(m.problems().len(), 1, "errors stay until the next action");
}

#[test]
fn the_note_goes_when_the_view_changes() {
    let svc = admin();
    let mut m = with(&svc);
    m.show(View::Settings);
    m.set_timer(true);
    m.show(View::Settings);
    assert!(m.admin_note().is_some(), "same view, drawn again");
    m.show(View::Cards);
    assert!(m.admin_note().is_none());
}

#[test]
fn a_program_switch_is_saved_in_the_register() {
    let svc = admin();
    let mut m = with(&svc);
    m.set_program_auto("fs-tasks", true);
    assert!(svc.programs.lock().unwrap()[1].auto_update);
    assert!(m.programs()[1].auto_update);
    m.set_program_auto("nope", true);
    assert_eq!(m.problems().len(), 1);
}

#[test]
fn the_sources_live_on_the_admin_page_and_the_reference_stays() {
    let svc = admin();
    let mut m = with(&svc);
    m.set_new_source("https://example.org/more");
    m.add_source();
    assert_eq!(m.sources().len(), 2);
    m.set_enabled("more", false);
    assert!(!m.sources()[1].enabled);
    m.remove_source("freesynergy");
    assert_eq!(
        m.sources().len(),
        2,
        "the reference source cannot be removed"
    );
    m.remove_source("more");
    assert_eq!(m.sources().len(), 1);
}

#[test]
fn installed_message_goes_with_a_view_change_but_an_error_does_not() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.show(View::Cards);
    m.run_job("fs-tasks");
    assert!(m.job().is_some_and(|j| j.version.is_some()));
    m.show(View::Cards);
    assert!(m.job().is_some(), "same view: still told");
    m.show(View::Settings);
    assert!(
        m.job().is_none(),
        "success message gone after a view change"
    );

    let f = fake(Vec::new(), Some(fs_store_app::backend::JobError::Checksum));
    let mut m = model(&f);
    m.show(View::Cards);
    m.run_job("fs-tasks");
    m.show(View::Settings);
    assert!(m.job().is_some());
    assert_eq!(m.problems().len(), 1);
}

#[test]
fn the_placeholder_has_a_gradient_from_the_name() {
    let p = placeholder("fs-tasks");
    assert_ne!(p.rgb, p.rgb_end);
    assert_eq!(p, placeholder("fs-tasks"));
    assert_ne!(placeholder("fs-people").rgb, p.rgb);
}

#[test]
fn a_card_uses_the_stored_icon_only_when_there_is_one() {
    let f = fake(Vec::new(), None);
    f.state
        .lock()
        .unwrap()
        .icons
        .push(("fs-tasks".into(), "/icons/fs-tasks.png".into()));
    let m = model(&f);
    let cards = m.cards();
    let tasks = cards.iter().find(|c| c.name == "fs-tasks").unwrap();
    let theme = cards.iter().find(|c| c.name == "fs-theme").unwrap();
    assert_eq!(
        tasks.icon.as_deref().and_then(|p| p.to_str()),
        Some("/icons/fs-tasks.png")
    );
    assert!(theme.icon.is_none());
}
