//! Pattern: State (each job moves along one fixed course of phases) and Unit of Work (the jobs of the page form
//! the batch: downloads run at once, installing starts when none is `Pending` any more). Job course: `Pending`, `Verified`,
//! `Installing`, `Installed`; on an error `Failed` and then `RolledBack` (nothing is left half done;
//! `RollbackFailed` stays at `Failed`).

use fs_store_core::Version;

use crate::backend::{JobError, JobKind, Phase, Prepared};

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

/// One program with an update waiting: a line of the "Updates" block on the start page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRow {
    /// Package name (the key).
    pub name: String,
    /// The name a person reads.
    pub title: String,
    /// The installed version.
    pub from: Version,
    /// The offered version.
    pub to: Version,
    /// Its update is queued or running.
    pub queued: bool,
}

impl StoreModel {
    /// The name a person reads for the package `id`: the catalog's own name, else derived (the one function
    /// [`fs_store_core::display_name`]). Every sentence and window uses this, never the id.
    #[must_use]
    pub fn title(&self, id: &str) -> String {
        let given = self
            .offers
            .iter()
            .find(|p| p.name == id)
            .map(|p| p.title.as_str());
        fs_store_core::display_name(id, given)
    }

    /// The programs with an update waiting, in the order of the cards: the "Updates" block of the start page.
    #[must_use]
    pub fn update_rows(&self) -> Vec<UpdateRow> {
        let mut rows: Vec<UpdateRow> = self
            .available
            .iter()
            .filter(|r| r.action == Action::Update)
            .filter_map(|r| {
                Some(UpdateRow {
                    name: r.package.name.clone(),
                    title: self.title(&r.package.name),
                    from: r.installed?,
                    to: r.package.version,
                    queued: self
                        .jobs
                        .iter()
                        .any(|j| j.name == r.package.name && !j.is_finished()),
                })
            })
            .collect();
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        rows
    }

    /// Takes the finished jobs out of the job list (what is queued or running stays).
    pub fn clear_finished_jobs(&mut self) {
        self.jobs.retain(|j| !j.is_finished());
    }

    /// True while a job or an uninstall runs.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.busy_name().is_some()
    }

    /// Puts the job the button of `name` stands for into the queue: `Pending` (its download starts at once).
    /// `None` (and a problem) if there is nothing to start, the program is being removed, or it is queued already.
    /// Run it with [`StoreBackend::prepare`], tell [`StoreModel::prepared`], then [`StoreModel::take_commit`].
    pub fn begin_job(&mut self, name: &str) -> Option<Ticket<'_>> {
        if self.ui.removing.is_some() {
            self.problems = vec![Problem::Busy(self.ui.removing.clone().unwrap_or_default())];
            return None;
        }
        if self.jobs.iter().any(|j| j.name == name && !j.is_finished()) {
            self.problems = vec![Problem::Busy(name.to_owned())];
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
        self.jobs.retain(|j| j.name != name);
        self.jobs.push(JobView {
            kind,
            name: name.to_owned(),
            phase: Phase::Pending,
            history: vec![Phase::Pending],
            version: None,
            error: None,
        });
        let job = self.jobs.last()?;
        Some(Ticket {
            kind,
            name: &job.name,
        })
    }

    /// Every program that has an update and is not queued yet goes into the queue, in the order of the cards.
    /// Returns what to start.
    pub fn begin_all_updates(&mut self) -> Vec<(JobKind, String)> {
        let names: Vec<String> = self
            .available
            .iter()
            .filter(|r| r.action == Action::Update)
            .map(|r| r.package.name.clone())
            .collect();
        names
            .into_iter()
            .filter_map(|n| self.begin_job(&n).map(|t| (t.kind, t.name.to_owned())))
            .collect()
    }

    /// How many programs have an update waiting (the number on "Update all").
    #[must_use]
    pub fn updates_waiting(&self) -> usize {
        self.available
            .iter()
            .filter(|r| r.action == Action::Update)
            .filter(|r| {
                !self
                    .jobs
                    .iter()
                    .any(|j| j.name == r.package.name && !j.is_finished())
            })
            .count()
    }

    /// The backend reports a phase of the job `name`.
    pub fn observe_job(&mut self, name: &str, phase: Phase) {
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| j.name == name && !j.is_finished() && !j.history.contains(&phase))
        {
            job.phase = phase;
            job.history.push(phase);
        }
    }

    /// The backend reports a phase of the last job (one job at a time: tests, and programs without a window).
    pub fn observe(&mut self, phase: Phase) {
        if let Some(name) = self.jobs.last().map(|j| j.name.clone()) {
            self.observe_job(&name, phase);
        }
    }

    /// The download of `name` is done: `Verified` (it waits for the others), or the job failed.
    pub fn prepared(&mut self, name: &str, result: Result<Prepared, JobError>) {
        match result {
            Ok(p) => {
                self.stash.insert(name.to_owned(), p);
                self.observe_job(name, Phase::Verified);
            }
            Err(error) => self.fail_job(name, error),
        }
    }

    /// When no download is running any more and nothing installs, the names that are ready now go to the
    /// installer ALL AT ONCE; they are `Installing` from here. Otherwise `None`.
    pub fn take_commit(&mut self) -> Option<Vec<(String, Prepared)>> {
        let waiting = |p: Phase| self.jobs.iter().any(|j| j.phase == p);
        if waiting(Phase::Pending) || waiting(Phase::Installing) || !waiting(Phase::Verified) {
            return None;
        }
        let names: Vec<String> = self
            .jobs
            .iter()
            .filter(|j| j.phase == Phase::Verified)
            .map(|j| j.name.clone())
            .collect();
        let mut batch = Vec::new();
        for n in names {
            self.observe_job(&n, Phase::Installing);
            if let Some(p) = self.stash.remove(&n) {
                batch.push((n, p));
            }
        }
        Some(batch)
    }

    /// The installer is done with the batch: one result per name.
    pub fn committed(&mut self, results: Vec<(String, Result<Version, JobError>)>) {
        for (name, result) in results {
            self.end_job(&name, result);
        }
        self.read_installed();
        self.merge();
    }

    /// The backend is done with the last job: `Installed`, or `Failed` and then `RolledBack`.
    pub fn finish(&mut self, result: Result<Version, JobError>) {
        if let Some(name) = self.jobs.last().map(|j| j.name.clone()) {
            self.end_job(&name, result);
        }
        self.read_installed();
        self.merge();
    }

    fn fail_job(&mut self, name: &str, error: JobError) {
        let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) else {
            return;
        };
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

    fn end_job(&mut self, name: &str, result: Result<Version, JobError>) {
        match result {
            Ok(version) => {
                if let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) {
                    job.version = Some(version);
                    job.phase = Phase::Installed;
                    job.history.push(Phase::Installed);
                }
                self.updated(name);
            }
            Err(error) => self.fail_job(name, error),
        }
    }

    /// Runs the job of `name` to its end on this thread (tests, and programs without a window).
    pub fn run_job(&mut self, name: &str) {
        let Some(ticket) = self.begin_job(name) else {
            return;
        };
        let (kind, name) = (ticket.kind, ticket.name.to_owned());
        let backend = self.backend();
        let result = backend.prepare(kind, &name);
        self.prepared(&name, result);
        let Some(batch) = self.take_commit() else {
            self.read_installed();
            self.merge();
            return;
        };
        let mut seen = Vec::new();
        let results = backend.commit(batch, &mut |n, ph| seen.push((n.to_owned(), ph)));
        for (n, ph) in seen {
            self.observe_job(&n, ph);
        }
        self.committed(results);
    }
}
