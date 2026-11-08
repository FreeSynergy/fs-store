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
    assert_eq!(
        m.job_line(),
        "fs-theme: the previous state is back. fs-theme was not installed: its signature could not be verified."
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
fn only_one_job_at_a_time() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.begin_job("fs-theme").is_some());
    assert!(m.is_busy());
    assert!(m.begin_job("fs-tasks").is_none());
    assert_eq!(m.problems(), [Problem::Busy("fs-theme".into())]);
    m.observe(Phase::Verified);
    assert_eq!(m.job().unwrap().phase, Phase::Verified);
    m.finish(Ok(v(1, 0, 0)));
    assert!(!m.is_busy());
}
