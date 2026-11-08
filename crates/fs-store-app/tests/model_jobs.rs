//! Tests of the view-model: the phases of install, update and failed jobs, and one job at a time.
#![allow(clippy::assert_is_empty)]

mod common;

use fs_store_app::backend::{InstalledRow, JobError, JobKind, Phase};
use fs_store_app::model::{Action, Problem};

use common::{action_of, fake, model, v};

#[test]
fn install_flow_goes_through_all_phases() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.run_job("fs-theme");
    let job = m.job().unwrap();
    assert_eq!(
        job.history,
        [
            Phase::Pending,
            Phase::Verified,
            Phase::Installing,
            Phase::Installed
        ]
    );
    assert_eq!(job.version, Some(v(1, 0, 0)));
    assert!(!m.is_busy());
    assert_eq!(m.installed().len(), 1);
    assert_eq!(action_of(&m, "fs-theme"), Some(Action::UpToDate));
    assert_eq!(m.job_line(), "fs-theme 1.0.0 is installed.");
    assert_eq!(f.state.lock().unwrap().runs[0].0, JobKind::Install);
}

#[test]
fn update_flow_replaces_the_old_version() {
    let f = fake(
        vec![InstalledRow {
            name: "fs-tasks".into(),
            version: v(0, 6, 0),
        }],
        None,
    );
    let mut m = model(&f);
    m.run_job("fs-tasks");
    assert_eq!(f.state.lock().unwrap().runs[0].0, JobKind::Update);
    assert_eq!(m.installed()[0].version, v(0, 6, 1));
    assert_eq!(action_of(&m, "fs-tasks"), Some(Action::UpToDate));
}

#[test]
fn up_to_date_package_starts_no_job() {
    let f = fake(
        vec![InstalledRow {
            name: "fs-store".into(),
            version: v(0, 2, 0),
        }],
        None,
    );
    let mut m = model(&f);
    m.run_job("fs-store");
    assert!(m.job().is_none());
    assert!(f.state.lock().unwrap().runs.is_empty());
}

#[test]
fn failed_job_is_shown_as_failed_then_rolled_back() {
    let f = fake(Vec::new(), Some(JobError::ReleaseSignature));
    let mut m = model(&f);
    m.run_job("fs-theme");
    let job = m.job().unwrap();
    assert_eq!(
        job.history,
        [
            Phase::Pending,
            Phase::Verified,
            Phase::Installing,
            Phase::Failed,
            Phase::RolledBack
        ]
    );
    assert_eq!(job.phase, Phase::RolledBack);
    assert!(m.installed().is_empty());
    assert_eq!(m.job_line(), "fs-theme: the previous state is back.");
    assert_eq!(
        m.problem_lines(),
        ["fs-theme was not installed: its signature could not be verified."]
    );
    assert!(!m.is_busy());
}

#[test]
fn failed_rollback_stays_failed() {
    let f = fake(Vec::new(), Some(JobError::RollbackFailed));
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert_eq!(m.job().unwrap().phase, Phase::Failed);
    assert!(!m.is_busy());
}

#[test]
fn a_second_job_joins_the_queue_but_the_same_one_twice_is_refused() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.begin_job("fs-theme").is_some());
    assert!(m.is_busy());
    assert!(
        m.begin_job("fs-tasks").is_some(),
        "downloads run side by side"
    );
    assert!(m.begin_job("fs-theme").is_none());
    assert_eq!(m.problems(), [Problem::Busy("fs-theme".into())]);
    assert_eq!(m.jobs().len(), 2);
}

#[test]
fn installing_waits_until_every_download_is_ready_and_then_takes_all_at_once() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-theme").unwrap();
    m.begin_job("fs-tasks").unwrap();
    let backend = m.backend();
    m.prepared("fs-theme", backend.prepare(JobKind::Install, "fs-theme"));
    assert!(m.take_commit().is_none(), "fs-tasks still downloads");
    m.prepared("fs-tasks", backend.prepare(JobKind::Install, "fs-tasks"));
    let batch = m.take_commit().unwrap();
    assert_eq!(batch.len(), 2);
    assert!(m.jobs().iter().all(|j| j.phase == Phase::Installing));
    assert!(m.take_commit().is_none(), "the batch is handed out once");
    let results = backend.commit(batch, &mut |_, _| {});
    m.committed(results);
    assert!(m.jobs().iter().all(|j| j.phase == Phase::Installed));
}

#[test]
fn a_failed_download_stays_on_its_card_and_the_rest_is_installed() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-theme").unwrap();
    m.begin_job("fs-tasks").unwrap();
    let backend = m.backend();
    m.prepared("fs-theme", Err(JobError::Checksum));
    m.prepared("fs-tasks", backend.prepare(JobKind::Install, "fs-tasks"));
    let batch = m.take_commit().unwrap();
    assert_eq!(batch.len(), 1);
    m.committed(backend.commit(batch, &mut |_, _| {}));
    let cards = m.cards();
    let theme = cards.iter().find(|c| c.name == "fs-theme").unwrap();
    assert!(theme.failure.is_some());
    assert_eq!(
        m.jobs()
            .iter()
            .find(|j| j.name == "fs-tasks")
            .unwrap()
            .phase,
        Phase::Installed
    );
}

#[test]
fn update_all_queues_every_update_once_and_counts_them() {
    let row = |n: &str, ver| InstalledRow {
        name: n.to_owned(),
        version: ver,
    };
    let f = fake(
        vec![row("fs-tasks", v(0, 6, 0)), row("fs-store", v(0, 1, 0))],
        None,
    );
    let mut m = model(&f);
    m.refresh();
    let waiting = m.updates_waiting();
    assert_eq!(waiting, 2);
    let started = m.begin_all_updates();
    assert_eq!(started.len(), waiting);
    assert_eq!(m.updates_waiting(), 0);
    assert!(m.begin_all_updates().is_empty());
}
