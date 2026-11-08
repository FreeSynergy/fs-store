//! Pattern: State (the job moves along one fixed course of phases). Job course: `Pending`, `Verified`,
//! `Installing`, `Installed`; on an error `Failed` and then `RolledBack` (nothing is left half done;
//! `RollbackFailed` stays at `Failed`).

use fs_store_core::Version;

use crate::backend::{JobError, JobKind, Phase};

use super::store::StoreModel;
use super::types::{Action, Problem};

/// The running or last job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobView {
    /// Install or update.
    pub kind: JobKind,
    /// Package name.
    pub name: String,
    /// Where it is.
    pub phase: Phase,
    /// Phases passed, in order, ending with `phase`.
    pub history: Vec<Phase>,
    /// The new version, once installed.
    pub version: Option<Version>,
    /// Why it failed.
    pub error: Option<JobError>,
}

impl JobView {
    /// True once nothing more will happen.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(
            self.phase,
            Phase::Installed | Phase::RolledBack | Phase::Failed
        )
    }
}

/// What a worker needs to run the job it was given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket<'a> {
    /// Install or update.
    pub kind: JobKind,
    /// Package name.
    pub name: &'a str,
}

impl StoreModel {
    /// True while a job runs: only one at a time.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.job.as_ref().is_some_and(|j| !j.is_finished())
    }

    /// Starts the job the button of `name` stands for: `Pending`. `None` (and a problem when busy) if there is
    /// nothing to start. Run the job with the backend, tell [`StoreModel::observe`], then [`StoreModel::finish`].
    pub fn begin_job(&mut self, name: &str) -> Option<Ticket<'_>> {
        if let Some(job) = self.job.as_ref().filter(|j| !j.is_finished()) {
            let busy = Problem::Busy(job.name.clone());
            self.problems = vec![busy];
            return None;
        }
        self.problems.clear();
        let action = self
            .available
            .iter()
            .find(|r| r.package.name == name)
            .map(|r| r.action);
        let kind = match action {
            Some(Action::Install) => JobKind::Install,
            Some(Action::Update) => JobKind::Update,
            Some(Action::UpToDate) | None => {
                self.problems.push(Problem::NothingToDo(name.to_owned()));
                return None;
            }
        };
        self.job = Some(JobView {
            kind,
            name: name.to_owned(),
            phase: Phase::Pending,
            history: vec![Phase::Pending],
            version: None,
            error: None,
        });
        let job = self.job.as_ref()?;
        Some(Ticket {
            kind,
            name: &job.name,
        })
    }

    /// The backend reports a phase.
    pub fn observe(&mut self, phase: Phase) {
        if let Some(job) = self.job.as_mut().filter(|j| !j.is_finished()) {
            job.phase = phase;
            job.history.push(phase);
        }
    }

    /// The backend is done: `Installed`, or `Failed` and then `RolledBack`. Reads the installed list again; a failure is told as a problem.
    pub fn finish(&mut self, result: Result<Version, JobError>) {
        if let Some(job) = self.job.as_mut() {
            match result {
                Ok(version) => {
                    job.version = Some(version);
                    job.phase = Phase::Installed;
                    job.history.push(Phase::Installed);
                }
                Err(error) => {
                    job.history.push(Phase::Failed);
                    job.phase = Phase::Failed;
                    if error != JobError::RollbackFailed {
                        job.history.push(Phase::RolledBack);
                        job.phase = Phase::RolledBack;
                    }
                    self.problems.push(Problem::JobFailed {
                        name: job.name.clone(),
                        error: error.clone(),
                    });
                    job.error = Some(error);
                }
            }
        }
        self.read_installed();
        self.merge();
    }

    /// Runs the job of `name` to its end on this thread (tests, and programs without a window).
    pub fn run_job(&mut self, name: &str) {
        let Some(ticket) = self.begin_job(name) else {
            return;
        };
        let (kind, name) = (ticket.kind, ticket.name.to_owned());
        let backend = self.backend();
        let mut seen = Vec::new();
        let result = backend.run(kind, &name, &mut |p| seen.push(p));
        for phase in seen {
            self.observe(phase);
        }
        self.finish(result);
    }
}
