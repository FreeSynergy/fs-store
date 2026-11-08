//! Tests of the view-model: a failure is always a visible message; installed is never offered as installable;
//! the menu; opening programs.

#![allow(clippy::assert_is_empty)]

mod common;

use std::sync::{Arc, Mutex};

use fs_store_app::backend::{InstallKind, InstalledDetails, InstalledRow, JobError, PackageRow};
use fs_store_app::launcher::Launcher;
use fs_store_app::model::{Action, Command, Effect, Problem, StoreModel, View, is_quit_key};

use common::{action_of, fake, model, package, v};

fn installed(name: &str, version: fs_store_core::Version) -> InstalledRow {
    InstalledRow {
        name: name.into(),
        version,
    }
}

#[test]
fn failed_job_gives_a_message_with_the_reason() {
    let f = fake(Vec::new(), Some(JobError::Other("disk full".into())));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert_eq!(m.problem_lines(), ["fs-theme was not installed: disk full"]);
    assert!(matches!(m.problems()[0], Problem::JobFailed { .. }));
}

#[test]
fn unsupported_format_names_the_format() {
    let f = fake(Vec::new(), Some(JobError::Unsupported("Flatpak".into())));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert!(m.problem_lines()[0].contains("Flatpak"));
}

#[test]
fn button_without_action_says_so() {
    let f = fake(vec![installed("fs-store", v(0, 2, 0))], None);
    let mut m = model(&f);
    m.run_job("fs-store");
    m.run_job("not-there");
    assert_eq!(m.problems().len(), 1);
    assert!(matches!(m.problems()[0], Problem::NothingToDo(_)));
    assert!(!m.problem_lines()[0].is_empty());
}

#[test]
fn empty_address_is_told() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.add_source();
    assert_eq!(m.problems(), [Problem::EmptyAddress]);
}

#[test]
fn duplicate_source_is_told() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.set_new_source("https://example.org/ref");
    m.add_source();
    assert_eq!(m.problems().len(), 1);
}

#[test]
fn installed_program_is_not_installable_and_appears_once() {
    let f = fake(
        vec![
            installed("fs-tasks", v(0, 6, 0)),
            installed("fs-tasks", v(0, 6, 1)),
            installed("fs-store", v(0, 2, 0)),
        ],
        None,
    );
    {
        let mut s = f.state.lock().unwrap();
        s.offered.push(package("fs-tasks", v(0, 5, 0)));
    }
    let m = model(&f);
    assert_eq!(m.installed().len(), 2);
    let tasks: Vec<_> = m
        .available()
        .iter()
        .filter(|r| r.package.name == "fs-tasks")
        .collect();
    assert_eq!(tasks.len(), 1);
    assert_eq!(action_of(&m, "fs-tasks"), Some(Action::UpToDate));
    assert_eq!(action_of(&m, "fs-store"), Some(Action::UpToDate));
    assert!(
        m.available()
            .iter()
            .all(|r| r.installed.is_none() == (r.action == Action::Install))
    );
}

#[test]
fn newest_offer_wins_and_offers_update() {
    let f = fake(vec![installed("fs-tasks", v(0, 5, 0))], None);
    let p: PackageRow = package("fs-tasks", v(0, 9, 0));
    f.state.lock().unwrap().offered.push(p);
    let m = model(&f);
    let row = m
        .available()
        .iter()
        .find(|r| r.package.name == "fs-tasks")
        .unwrap();
    assert_eq!(row.package.version, v(0, 9, 0));
    assert_eq!(row.action, Action::Update);
}

#[test]
fn menu_commands() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert_eq!(m.command(Command::Cards), Effect::Show(View::Cards));
    assert_eq!(m.command(Command::Sources), Effect::Show(View::Sources));
    assert_eq!(m.command(Command::Settings), Effect::Show(View::Settings));
    assert_eq!(m.command(Command::About), Effect::Show(View::About));
    assert_eq!(m.command(Command::Help), Effect::Help);
    assert_eq!(m.command(Command::Quit), Effect::Quit);
    assert_eq!(Command::from_id("quit"), Some(Command::Quit));
    assert_eq!(Command::from_id("x"), None);
    for c in Command::ALL {
        assert_eq!(Command::from_id(c.id()), Some(c));
    }
}

#[test]
fn quit_is_refused_while_a_job_runs() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.begin_job("fs-theme").is_some());
    assert_eq!(m.command(Command::Quit), Effect::Nothing);
    assert_eq!(m.problems(), [Problem::Busy("fs-theme".into())]);
}

#[test]
fn quit_key_is_ctrl_q() {
    assert!(is_quit_key(81, true));
    assert!(!is_quit_key(81, false));
    assert!(!is_quit_key(65, true));
}

#[test]
fn about_names_version_licence_and_address() {
    let lines = StoreModel::about_lines();
    assert_eq!(
        lines[0],
        format!("FreeSynergy.Store {}", env!("CARGO_PKG_VERSION"))
    );
    assert!(lines[1].contains("MIT OR Apache-2.0"));
    assert!(lines[2].contains("https://github.com/FreeSynergy/fs-store"));
}

struct FakeLauncher {
    present: Vec<String>,
    started: Mutex<Vec<String>>,
    fail: bool,
}

impl Launcher for FakeLauncher {
    fn exists(&self, command: &str) -> bool {
        self.present.iter().any(|c| c == command)
    }
    fn start(&self, command: &str) -> Result<(), String> {
        if self.fail {
            return Err("permission denied".into());
        }
        self.started.lock().unwrap().push(command.to_owned());
        Ok(())
    }
    fn open_url(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
}

fn launcher(present: &[&str], fail: bool) -> FakeLauncher {
    FakeLauncher {
        present: present.iter().map(|s| (*s).to_owned()).collect(),
        started: Mutex::new(Vec::new()),
        fail,
    }
}

fn with_details() -> Arc<common::Fake> {
    let f = fake(vec![installed("tasks", v(0, 6, 0))], None);
    f.state.lock().unwrap().details.push((
        "tasks".into(),
        InstalledDetails {
            kinds: vec![InstallKind::Archive],
            commands: vec![("tasks-cli".into(), "/home/u/.local/bin/tasks-cli".into())],
            folder: Some("/home/u/.local/share/fs-store/packages/tasks/0.6.0".into()),
            source: Some("https://example.org/ref".into()),
            facets: vec!["tasks-board".into()],
        },
    ));
    f
}

#[test]
fn details_become_lines_and_nothing_is_invented() {
    let m = model(&with_details());
    assert_eq!(
        m.detail_lines("tasks"),
        [
            "Installed as: tar.gz",
            "Command tasks-cli: /home/u/.local/bin/tasks-cli",
            "Folder: /home/u/.local/share/fs-store/packages/tasks/0.6.0",
            "Source: https://example.org/ref",
        ]
    );
    assert!(m.detail_lines("unknown").is_empty());
}

#[test]
fn open_prefers_fs_name_then_name_app_then_package_command() {
    let mut m = model(&with_details());
    let l = launcher(&["fs-tasks", "tasks-app", "tasks-cli"], false);
    m.open("tasks", None, &l);
    let l2 = launcher(&["tasks-app", "tasks-cli"], false);
    m.open("tasks", None, &l2);
    let l3 = launcher(&["tasks-cli"], false);
    m.open("tasks", None, &l3);
    assert_eq!(*l.started.lock().unwrap(), ["fs-tasks"]);
    assert_eq!(*l2.started.lock().unwrap(), ["tasks-app"]);
    assert_eq!(*l3.started.lock().unwrap(), ["tasks-cli"]);
    assert!(m.problems().is_empty());
}

#[test]
fn open_facet_starts_the_facet_command() {
    let mut m = model(&with_details());
    let l = launcher(&[], false);
    m.open("tasks", Some("tasks-board"), &l);
    assert_eq!(*l.started.lock().unwrap(), ["tasks-board"]);
}

#[test]
fn open_failure_and_missing_command_are_visible() {
    let mut m = model(&with_details());
    m.open("tasks", Some("tasks-board"), &launcher(&[], true));
    assert_eq!(
        m.problem_lines(),
        ["tasks-board could not be opened: permission denied"]
    );
    m.open("tasks", None, &launcher(&[], false));
    assert_eq!(m.problems().len(), 1);
    assert!(m.problem_lines()[0].contains("tasks"));
}

#[test]
fn help_goes_to_the_page_of_the_current_view_in_the_persons_language() {
    use fs_store_app::model::{help_url, is_help_key};
    assert_eq!(
        help_url(View::Cards, "en_US.UTF-8"),
        "https://freesynergy.github.io/fs-documentation/help/store/programs/"
    );
    assert_eq!(
        help_url(View::Settings, "de_DE.UTF-8"),
        "https://freesynergy.github.io/fs-documentation/de/help/store/settings/"
    );
    assert!(help_url(View::Sources, "").ends_with("/help/store/sources/"));
    assert!(help_url(View::About, "fr").ends_with("/help/store/about/"));
    assert!(is_help_key(112) && !is_help_key(81));
}
