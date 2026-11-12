//! Tests of the start page model: updates on the start page, the job list with several jobs, display names.

mod common;

use fs_store_app::backend::{InstalledRow, Phase};
use fs_store_app::model::Problem;

use common::{fake, model, v};

fn installed(name: &str, version: fs_store_core::Version) -> InstalledRow {
    InstalledRow {
        name: name.into(),
        version,
    }
}

#[test]
fn programs_are_shown_as_freesynergy_name() {
    let f = fake(Vec::new(), None);
    let m = model(&f);
    assert_eq!(m.title("fs-tasks"), "FreeSynergy.Tasks");
    for card in m.cards() {
        assert!(card.title.starts_with("FreeSynergy."), "{}", card.title);
    }
}

#[test]
fn no_sentence_shows_an_fs_id() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert_eq!(m.job_line(), "FreeSynergy.Theme 1.0.0 is installed.");
    m.run_job("fs-theme");
    for line in m.problem_lines().into_iter().chain(m.job_lines()) {
        assert!(!line.contains("fs-"), "{line}");
    }
}

#[test]
fn updates_are_listed_for_the_start_page() {
    let f = fake(vec![installed("fs-tasks", v(0, 6, 0))], None);
    let mut m = model(&f);
    let rows = m.update_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "FreeSynergy.Tasks");
    assert_eq!((rows[0].from, rows[0].to), (v(0, 6, 0), v(0, 6, 1)));
    assert!(!rows[0].queued);
    m.begin_job("fs-tasks");
    assert!(m.update_rows()[0].queued);
}

#[test]
fn several_jobs_are_in_one_list_and_nothing_blocks() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.begin_job("fs-theme").is_some());
    assert!(m.begin_job("fs-tasks").is_some());
    assert_eq!(m.jobs().len(), 2);
    assert_eq!(m.job_lines().len(), 2);
    assert_eq!(
        m.job_lines()[0],
        "Waiting: FreeSynergy.Theme: waiting to start."
    );
    assert!(m.problems().iter().all(|p| !matches!(p, Problem::Busy(_))));
    // the same program twice is refused, the others stay possible
    assert!(m.begin_job("fs-theme").is_none());
    m.observe_job("fs-tasks", Phase::Verified);
    assert_eq!(m.jobs()[1].phase, Phase::Verified);
    assert_eq!(m.jobs()[0].phase, Phase::Pending);
}

#[test]
fn finished_jobs_can_be_cleared_and_running_ones_stay() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.run_job("fs-theme");
    m.begin_job("fs-tasks");
    m.clear_finished_jobs();
    assert_eq!(m.jobs().len(), 1);
    assert_eq!(m.jobs()[0].name, "fs-tasks");
}

#[test]
fn the_catalog_title_is_used_for_the_name() {
    let f = fake(Vec::new(), None);
    f.state
        .lock()
        .unwrap()
        .offered
        .push(fs_store_app::backend::PackageRow {
            title: "FreeSynergy.Spezial".into(),
            ..common::package("fs-special", v(1, 0, 0))
        });
    let m = model(&f);
    assert_eq!(m.title("fs-special"), "FreeSynergy.Spezial");
    assert_eq!(
        m.cards()
            .iter()
            .find(|c| c.name == "fs-special")
            .unwrap()
            .title,
        "FreeSynergy.Spezial"
    );
}

#[test]
fn update_lines_say_the_display_name_and_the_queue() {
    let f = fake(vec![installed("fs-tasks", v(0, 6, 0))], None);
    let mut m = model(&f);
    assert_eq!(
        m.update_lines(),
        ["FreeSynergy.Tasks: 0.6.0 \u{2192} 0.6.1"]
    );
    m.begin_job("fs-tasks");
    assert_eq!(
        m.update_lines(),
        ["FreeSynergy.Tasks: 0.6.0 \u{2192} 0.6.1 (in the queue)"]
    );
}

#[test]
fn refresh_during_jobs_keeps_jobs_and_failed_messages() {
    let f = fake(Vec::new(), Some(fs_store_app::backend::JobError::Checksum));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert!(
        m.problems()
            .iter()
            .any(|p| matches!(p, Problem::JobFailed { .. }))
    );
    let fetched = fs_store_app::model::fetch(m.backend().as_ref());
    m.apply_fetched(fetched);
    assert!(
        m.problems()
            .iter()
            .any(|p| matches!(p, Problem::JobFailed { .. }))
    );
    m.begin_job("fs-tasks");
    m.apply_fetched(fs_store_app::model::fetch(m.backend().as_ref()));
    assert!(
        m.jobs()
            .iter()
            .any(|j| j.name == "fs-tasks" && !j.is_finished())
    );
}

#[test]
fn job_states_are_waiting_running_done_failed() {
    let f = fake(Vec::new(), Some(fs_store_app::backend::JobError::Checksum));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert!(m.job_lines()[0].starts_with("Failed: FreeSynergy.Theme was not installed"));
    f.state.lock().unwrap().fail = None;
    m.begin_job("fs-tasks");
    m.observe_job("fs-tasks", Phase::Verified);
    assert!(m.job_lines()[1].starts_with("Waiting: "));
    m.observe_job("fs-tasks", Phase::Installing);
    assert!(m.job_lines()[1].starts_with("Running: "));
}

#[test]
fn clearing_finished_jobs_while_a_batch_runs_loses_nothing() {
    let f = fake(Vec::new(), Some(fs_store_app::backend::JobError::Checksum));
    let mut m = model(&f);
    m.run_job("fs-theme");
    f.state.lock().unwrap().fail = None;
    m.begin_job("fs-tasks");
    m.observe_job("fs-tasks", Phase::Verified);
    m.observe_job("fs-tasks", Phase::Installing);
    m.clear_finished_jobs();
    assert_eq!(m.jobs().len(), 1);
    m.committed(vec![("fs-tasks".into(), Ok(v(0, 6, 1)))]);
    assert_eq!(m.jobs()[0].phase, Phase::Installed);
}

#[test]
fn an_old_read_does_not_undo_an_install() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.defer_refresh();
    let ticket = m.begin_read().unwrap();
    let old = ticket.read(); // read before the install: theme is not installed in it
    m.run_job("fs-theme");
    assert_eq!(m.installed().len(), 1);
    assert!(!m.finish_read(old), "an old result is thrown away");
    assert_eq!(m.installed().len(), 1);
    assert!(!m.is_refreshing());
}

#[test]
fn only_one_read_at_a_time_and_the_button_knows() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    let ticket = m.begin_read().unwrap();
    assert!(m.is_refreshing());
    assert!(m.begin_read().is_none());
    assert!(m.finish_read(ticket.read()));
    assert!(!m.is_refreshing());
    assert!(m.begin_read().is_some());
}

#[test]
fn source_commands_of_the_window_only_ask_for_a_read() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.refresh();
    m.defer_refresh();
    m.set_enabled("freesynergy", false);
    assert!(m.take_read_wanted());
    assert!(!m.take_read_wanted());
    // an older read is now stale
    let ticket = m.begin_read().unwrap();
    m.set_enabled("freesynergy", true);
    assert!(!m.finish_read(ticket.read()));
}

#[test]
fn a_silent_installer_gets_its_own_text() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-tasks");
    m.observe_job("fs-tasks", Phase::Verified);
    m.observe_job("fs-tasks", Phase::Installing);
    m.committed(Vec::new());
    let line = &m.job_lines()[0];
    assert!(line.contains("gave no answer"), "{line}");
    assert!(!line.contains("broke down"));
}
