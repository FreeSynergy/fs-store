//! Tests of the update part of the view-model: badge, new marks, settings, the check (with a fake service).
#![allow(clippy::assert_is_empty)]

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::backend::InstalledRow;
use fs_store_app::model::{CheckState, StoreModel, View, check_due, format_time};
use fs_store_app::updates::UpdateService;
use fs_store_sources::{CheckEvery, Findings, InstallMode, Novelty, Settings, Update};

use common::{fake, v};

struct FakeUpdates {
    settings: Mutex<Settings>,
    findings: Mutex<Findings>,
    after_check: Mutex<Option<Findings>>,
    check_result: Mutex<Result<(), String>>,
    checks: Mutex<u32>,
    saved_findings: Mutex<u32>,
    now: u64,
}

fn service(findings: Findings, now: u64) -> Arc<FakeUpdates> {
    Arc::new(FakeUpdates {
        settings: Mutex::new(Settings {
            notify: true,
            ..Settings::default()
        }),
        findings: Mutex::new(findings),
        after_check: Mutex::new(None),
        check_result: Mutex::new(Ok(())),
        checks: Mutex::new(0),
        saved_findings: Mutex::new(0),
        now,
    })
}

impl UpdateService for FakeUpdates {
    fn settings(&self) -> Result<Settings, String> {
        Ok(*self.settings.lock().unwrap())
    }
    fn save_settings(&self, s: &Settings) -> Result<(), String> {
        *self.settings.lock().unwrap() = *s;
        Ok(())
    }
    fn findings(&self) -> Result<Findings, String> {
        Ok(self.findings.lock().unwrap().clone())
    }
    fn save_findings(&self, f: &Findings) -> Result<(), String> {
        *self.findings.lock().unwrap() = f.clone();
        *self.saved_findings.lock().unwrap() += 1;
        Ok(())
    }
    fn check(&self) -> Result<(), String> {
        *self.checks.lock().unwrap() += 1;
        if let Some(f) = self.after_check.lock().unwrap().take() {
            *self.findings.lock().unwrap() = f;
        }
        self.check_result.lock().unwrap().clone()
    }
    fn now(&self) -> u64 {
        self.now
    }
}

fn update(name: &str, from: &str, to: &str) -> Update {
    Update {
        name: name.into(),
        from: from.into(),
        to: to.into(),
    }
}

fn findings() -> Findings {
    Findings {
        checked: 1_000,
        update: vec![update("fs-store", "0.1.0", "0.2.0")],
        new: vec![Novelty {
            name: "fs-theme".into(),
            version: "1.0.0".into(),
        }],
        offered: Vec::new(),
    }
}

fn model_with(svc: &Arc<FakeUpdates>) -> StoreModel {
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
fn label_and_title_carry_the_count() {
    let svc = service(findings(), 2_000);
    let m = model_with(&svc);
    assert_eq!(m.window_title(), "FreeSynergy.Store (1)");
    assert_eq!(m.update_list().len(), 1);
}

#[test]
fn no_updates_means_plain_label_and_title() {
    let svc = service(Findings::default(), 2_000);
    let m = model_with(&svc);
    assert_eq!(m.window_title(), "FreeSynergy.Store");
}

#[test]
fn notify_off_hides_the_badge_but_not_the_list() {
    let svc = service(findings(), 2_000);
    let mut m = model_with(&svc);
    m.set_notify(false);
    assert_eq!(m.window_title(), "FreeSynergy.Store");
    assert_eq!(m.update_list().len(), 1);
    assert!(!svc.settings.lock().unwrap().notify);
}

#[test]
fn new_marks_show_on_first_look_and_are_gone_after() {
    let svc = service(findings(), 2_000);
    let mut m = model_with(&svc);
    assert!(!m.is_fresh("fs-theme"), "not looked at yet");
    m.show(View::Cards);
    assert!(m.is_fresh("fs-theme"));
    assert!(
        m.cards().iter().any(|c| c.name == "fs-theme" && c.is_new),
        "badge on the card"
    );
    assert!(svc.findings.lock().unwrap().new.is_empty(), "saved as seen");
    m.show(View::Cards); // drawn again: still marked
    assert!(m.is_fresh("fs-theme"));
    m.show(View::Sources);
    m.show(View::Cards);
    assert!(!m.is_fresh("fs-theme"), "new only until first looked at");
}

#[test]
fn nothing_is_marked_before_the_list_is_opened() {
    let svc = service(findings(), 2_000);
    let mut m = model_with(&svc);
    m.show(View::Settings);
    assert_eq!(m.new_programs().len(), 1);
    assert_eq!(*svc.saved_findings.lock().unwrap(), 0);
}

#[test]
fn settings_are_saved() {
    let svc = service(Findings::default(), 0);
    let mut m = model_with(&svc);
    m.set_check_every(CheckEvery::Hourly);
    m.set_install_mode(InstallMode::Auto);
    let saved = *svc.settings.lock().unwrap();
    assert_eq!(saved.check, CheckEvery::Hourly);
    assert_eq!(saved.install, InstallMode::Auto);
    assert_eq!(m.settings().check, CheckEvery::Hourly);
}

#[test]
fn due_rules() {
    assert!(!check_due(CheckEvery::Off, 0, 5));
    assert!(check_due(CheckEvery::Start, 4, 5));
    assert!(check_due(CheckEvery::Daily, 0, 5));
    assert!(!check_due(CheckEvery::Daily, 1_000, 1_000 + 86_399));
    assert!(check_due(CheckEvery::Daily, 1_000, 1_000 + 86_400));
    assert!(!check_due(CheckEvery::Hourly, 1_000, 1_000 + 3_599));
    assert!(check_due(CheckEvery::Hourly, 1_000, 1_000 + 3_600));
}

#[test]
fn start_check_follows_settings_and_age() {
    let svc = service(findings(), 1_000 + 7_200);
    let mut m = model_with(&svc);
    m.set_check_every(CheckEvery::Hourly);
    assert!(m.check_wanted_at_start());
    m.set_check_every(CheckEvery::Daily);
    assert!(!m.check_wanted_at_start());
    m.set_check_every(CheckEvery::Off);
    assert!(!m.check_wanted_at_start());
    m.set_check_every(CheckEvery::Start);
    assert!(m.check_wanted_at_start());
}

#[test]
fn check_runs_once_and_rereads_findings() {
    let svc = service(findings(), 2_000);
    *svc.after_check.lock().unwrap() = Some(Findings {
        checked: 5_000,
        ..Findings::default()
    });
    let mut m = model_with(&svc);
    assert!(m.begin_check());
    assert!(!m.begin_check(), "already running");
    assert_eq!(m.check_line().unwrap(), m.check_line().unwrap());
    assert_eq!(*m.check_state(), CheckState::Running);
    let result = m.update_service().check();
    m.finish_check(result);
    assert_eq!(*m.check_state(), CheckState::Done);
    assert_eq!(m.update_list().len(), 0);
    assert_eq!(m.last_checked(), 5_000);
    assert_eq!(*svc.checks.lock().unwrap(), 1);
}

#[test]
fn failed_check_is_told() {
    let svc = service(Findings::default(), 2_000);
    *svc.check_result.lock().unwrap() = Err("no network".into());
    let mut m = model_with(&svc);
    assert!(m.begin_check());
    let result = m.update_service().check();
    m.finish_check(result);
    assert!(m.check_line().unwrap().contains("no network"));
    assert!(m.begin_check(), "can be tried again");
}

#[test]
fn update_job_removes_the_entry() {
    let svc = service(findings(), 2_000);
    let mut m = model_with(&svc);
    m.run_job("fs-store");
    assert_eq!(m.update_list().len(), 0);
    assert!(svc.findings.lock().unwrap().update.is_empty());
    assert_eq!(m.window_title(), "FreeSynergy.Store");
}

#[test]
fn failed_update_job_keeps_the_entry() {
    let svc = service(findings(), 2_000);
    let f = fake(
        vec![InstalledRow {
            name: "fs-store".into(),
            version: v(0, 1, 0),
        }],
        Some(fs_store_app::backend::JobError::Checksum),
    );
    let mut m = StoreModel::new(f).with_updates(svc.clone());
    m.refresh();
    m.run_job("fs-store");
    assert_eq!(m.update_list().len(), 1);
}

#[test]
fn last_check_wording() {
    let svc = service(Findings::default(), 0);
    let m = model_with(&svc);
    assert_eq!(m.last_check_line(), "Last look for updates: never");
    assert_eq!(format_time(0), "1970-01-01 00:00 UTC");
    assert_eq!(format_time(1_794_000_000), "2026-11-06 21:20 UTC");
}
