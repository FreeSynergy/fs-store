//! Tests of the view-model against a fake backend.
#![allow(clippy::assert_is_empty)]

use std::sync::{Arc, Mutex};

use fs_store_app::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase, SourceRow,
    StoreBackend,
};
use fs_store_app::model::{Action, Problem, StoreModel, View};
use fs_store_core::{Kind, SourceError, Version};

struct Fake {
    state: Mutex<State>,
}

struct State {
    sources: Vec<SourceRow>,
    offered: Vec<PackageRow>,
    installed: Vec<InstalledRow>,
    fail: Option<JobError>,
    remove_calls: u32,
    runs: Vec<(JobKind, String)>,
}

fn v(a: u64, b: u64, c: u64) -> Version {
    Version::new(a, b, c)
}

fn package(name: &str, version: Version) -> PackageRow {
    PackageRow {
        name: name.to_owned(),
        kind: Kind::Program,
        version,
        source: "freesynergy".to_owned(),
    }
}

fn fake(installed: Vec<InstalledRow>, fail: Option<JobError>) -> Arc<Fake> {
    Arc::new(Fake {
        state: Mutex::new(State {
            sources: vec![SourceRow {
                id: "freesynergy".into(),
                url: "https://example.org/ref".into(),
                enabled: true,
                reference: true,
            }],
            offered: vec![
                package("fs-tasks", v(0, 6, 1)),
                package("fs-store", v(0, 2, 0)),
                package("fs-theme", v(1, 0, 0)),
            ],
            installed,
            fail,
            remove_calls: 0,
            runs: Vec::new(),
        }),
    })
}

impl StoreBackend for Fake {
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
        Ok(self.state.lock().unwrap().sources.clone())
    }
    fn add_source(&self, url: &str) -> Result<SourceRow, BackendError> {
        let mut s = self.state.lock().unwrap();
        if s.sources.iter().any(|x| x.url == url) {
            return Err(BackendError::Source(SourceError::Duplicate));
        }
        let row = SourceRow {
            id: url.rsplit('/').next().unwrap_or("x").to_owned(),
            url: url.to_owned(),
            enabled: true,
            reference: false,
        };
        s.sources.push(row.clone());
        Ok(row)
    }
    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError> {
        let mut s = self.state.lock().unwrap();
        let row = s
            .sources
            .iter_mut()
            .find(|x| x.id == id)
            .ok_or(BackendError::Source(SourceError::NotFound))?;
        row.enabled = enabled;
        Ok(())
    }
    fn remove_source(&self, id: &str) -> Result<(), BackendError> {
        let mut s = self.state.lock().unwrap();
        s.remove_calls += 1;
        s.sources.retain(|x| x.id != id);
        Ok(())
    }
    fn available(&self) -> Result<Listing, BackendError> {
        let s = self.state.lock().unwrap();
        let on = s.sources.iter().any(|x| x.enabled);
        Ok(Listing {
            packages: if on { s.offered.clone() } else { Vec::new() },
            ..Listing::default()
        })
    }
    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        Ok(self.state.lock().unwrap().installed.clone())
    }
    fn run(
        &self,
        kind: JobKind,
        name: &str,
        observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError> {
        let mut s = self.state.lock().unwrap();
        s.runs.push((kind, name.to_owned()));
        observe(Phase::Verified);
        observe(Phase::Installing);
        if let Some(e) = s.fail.clone() {
            return Err(e);
        }
        let version = s
            .offered
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.version)
            .ok_or(JobError::NotOffered)?;
        s.installed.retain(|i| i.name != name);
        s.installed.push(InstalledRow {
            name: name.to_owned(),
            version,
        });
        Ok(version)
    }
}

fn model(fake: &Arc<Fake>) -> StoreModel {
    let mut m = StoreModel::new(fake.clone());
    m.refresh();
    m
}

fn action_of(m: &StoreModel, name: &str) -> Option<Action> {
    m.available()
        .iter()
        .find(|r| r.package.name == name)
        .map(|r| r.action)
}

#[test]
fn starts_on_available_and_lists_actions() {
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
        ],
        None,
    );
    let m = model(&f);
    assert_eq!(m.view(), View::Available);
    assert_eq!(action_of(&m, "fs-tasks"), Some(Action::Update));
    assert_eq!(action_of(&m, "fs-store"), Some(Action::UpToDate));
    assert_eq!(action_of(&m, "fs-theme"), Some(Action::Install));
    assert_eq!(m.installed().len(), 2);
}

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

#[test]
fn reference_source_cannot_be_removed_but_can_be_switched_off() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.remove_source("freesynergy");
    assert_eq!(
        m.problems(),
        [Problem::Source(SourceError::ReferenceSource)]
    );
    assert_eq!(f.state.lock().unwrap().remove_calls, 0);
    assert_eq!(m.sources().len(), 1);
    assert_eq!(
        m.problem_lines(),
        ["The FreeSynergy source cannot be removed. You can switch it off instead."]
    );
    m.set_enabled("freesynergy", false);
    assert!(!m.sources()[0].enabled);
    assert!(m.available().is_empty());
    assert!(m.problems().is_empty());
}

#[test]
fn add_and_remove_a_source() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.set_new_source("   ");
    m.add_source();
    assert_eq!(m.sources().len(), 1);
    m.set_new_source(" https://example.org/mine ");
    m.add_source();
    assert_eq!(m.sources().len(), 2);
    assert_eq!(m.new_source(), "");
    m.set_new_source("https://example.org/mine");
    m.add_source();
    assert_eq!(m.problems(), [Problem::Source(SourceError::Duplicate)]);
    m.remove_source("mine");
    assert_eq!(m.sources().len(), 1);
    assert_eq!(f.state.lock().unwrap().remove_calls, 1);
}

#[test]
fn unreadable_and_skipped_are_told() {
    struct Odd;
    impl StoreBackend for Odd {
        fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
            Ok(Vec::new())
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
            Ok(Listing {
                packages: Vec::new(),
                skipped: vec![("x".into(), "no signature".into())],
                unreadable: vec![("s".into(), "offline".into())],
            })
        }
        fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
            Err(BackendError::Other("disk".into()))
        }
        fn run(&self, _: JobKind, _: &str, _: &mut dyn FnMut(Phase)) -> Result<Version, JobError> {
            Err(JobError::NotOffered)
        }
    }
    let mut m = StoreModel::new(Arc::new(Odd));
    m.refresh();
    assert_eq!(
        m.problem_lines(),
        [
            "The store could not do this: disk",
            "The source s could not be read: offline",
            "x is not offered: no signature",
        ]
    );
}
