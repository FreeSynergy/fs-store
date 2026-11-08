//! Pattern: View-model (state plus commands, no GUI types). The window only draws this and calls its commands.
//! Job course: `Pending`, `Verified`, `Installing`, `Installed`; on an error `Failed` and then `RolledBack`
//! (nothing is left half done; `RollbackFailed` stays at `Failed`).

use std::sync::Arc;

use fs_store_core::{SourceError, Version};

use crate::backend::{
    BackendError, InstalledRow, JobError, JobKind, PackageRow, Phase, SourceRow, StoreBackend,
};
use crate::words;

/// The three views of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The list of sources.
    Sources,
    /// Packages of the enabled sources.
    Available,
    /// Packages on this machine.
    Installed,
}

/// What the button of an available package does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Not installed: Install.
    Install,
    /// A newer version is offered: Update.
    Update,
    /// Nothing to do.
    UpToDate,
}

/// A row of the available view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableRow {
    /// The offer.
    pub package: PackageRow,
    /// The installed version, if any.
    pub installed: Option<Version>,
    /// What the button does.
    pub action: Action,
}

/// Something to tell the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A rule of the source list.
    Source(SourceError),
    /// A call failed; developer text.
    Backend(String),
    /// A source could not be read.
    Unreadable {
        /// Source id.
        source: String,
        /// Detail.
        detail: String,
    },
    /// A catalog entry is not offered.
    Skipped {
        /// Entry id.
        id: String,
        /// Reason.
        reason: String,
    },
    /// A job is running already.
    Busy(String),
}

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

/// State of the window.
pub struct StoreModel {
    backend: Arc<dyn StoreBackend>,
    view: View,
    sources: Vec<SourceRow>,
    available: Vec<AvailableRow>,
    installed: Vec<InstalledRow>,
    job: Option<JobView>,
    problems: Vec<Problem>,
    new_source: String,
}

impl StoreModel {
    /// A model over `backend`, showing the available view; nothing is read yet (see [`StoreModel::refresh`]).
    #[must_use]
    pub fn new(backend: Arc<dyn StoreBackend>) -> Self {
        Self {
            backend,
            view: View::Available,
            sources: Vec::new(),
            available: Vec::new(),
            installed: Vec::new(),
            job: None,
            problems: Vec::new(),
            new_source: String::new(),
        }
    }

    /// The backend, for a worker thread.
    #[must_use]
    pub fn backend(&self) -> Arc<dyn StoreBackend> {
        Arc::clone(&self.backend)
    }

    /// The view shown.
    #[must_use]
    pub fn view(&self) -> View {
        self.view
    }

    /// Show a view.
    pub fn show(&mut self, view: View) {
        self.view = view;
    }

    /// The sources, reference first.
    #[must_use]
    pub fn sources(&self) -> &[SourceRow] {
        &self.sources
    }

    /// Rows of the available view.
    #[must_use]
    pub fn available(&self) -> &[AvailableRow] {
        &self.available
    }

    /// Rows of the installed view.
    #[must_use]
    pub fn installed(&self) -> &[InstalledRow] {
        &self.installed
    }

    /// The running or last job.
    #[must_use]
    pub fn job(&self) -> Option<&JobView> {
        self.job.as_ref()
    }

    /// Things to tell the person, oldest first.
    #[must_use]
    pub fn problems(&self) -> &[Problem] {
        &self.problems
    }

    /// The text of the field for a new source.
    #[must_use]
    pub fn new_source(&self) -> &str {
        &self.new_source
    }

    /// The person typed in the field for a new source.
    pub fn set_new_source(&mut self, text: &str) {
        text.clone_into(&mut self.new_source);
    }

    /// True while a job runs: only one at a time.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.job.as_ref().is_some_and(|j| !j.is_finished())
    }

    /// Reads sources, installed and available packages again. Problems of the last read are replaced.
    pub fn refresh(&mut self) {
        self.problems.clear();
        self.read_sources();
        self.read_installed();
        self.read_available();
    }

    fn read_sources(&mut self) {
        match self.backend.sources() {
            Ok(list) => self.sources = list,
            Err(e) => self.report(&e),
        }
    }

    fn read_installed(&mut self) {
        match self.backend.installed() {
            Ok(list) => self.installed = list,
            Err(e) => self.report(&e),
        }
    }

    fn read_available(&mut self) {
        let listing = match self.backend.available() {
            Ok(listing) => listing,
            Err(e) => {
                self.available.clear();
                self.report(&e);
                return;
            }
        };
        self.available = listing
            .packages
            .into_iter()
            .map(|package| {
                let installed = self
                    .installed
                    .iter()
                    .find(|i| i.name == package.name)
                    .map(|i| i.version);
                let action = match installed {
                    None => Action::Install,
                    Some(v) if package.version > v => Action::Update,
                    Some(_) => Action::UpToDate,
                };
                AvailableRow {
                    package,
                    installed,
                    action,
                }
            })
            .collect();
        for (source, detail) in listing.unreadable {
            self.problems.push(Problem::Unreadable { source, detail });
        }
        for (id, reason) in listing.skipped {
            self.problems.push(Problem::Skipped { id, reason });
        }
    }

    fn report(&mut self, error: &BackendError) {
        self.problems.push(match error {
            BackendError::Source(e) => Problem::Source(e.clone()),
            BackendError::Other(text) => Problem::Backend(text.clone()),
        });
    }

    /// Adds the source typed in the field. An empty field does nothing.
    pub fn add_source(&mut self) {
        let url = self.new_source.trim().to_owned();
        if url.is_empty() {
            return;
        }
        self.problems.clear();
        match self.backend.add_source(&url) {
            Ok(_) => {
                self.new_source.clear();
                self.refresh();
            }
            Err(e) => self.report(&e),
        }
    }

    /// Switches a source on or off. The reference source may be switched off too.
    pub fn set_enabled(&mut self, id: &str, enabled: bool) {
        self.problems.clear();
        match self.backend.set_enabled(id, enabled) {
            Ok(()) => self.refresh(),
            Err(e) => self.report(&e),
        }
    }

    /// Removes a source. The reference source is refused here, before the backend is asked.
    pub fn remove_source(&mut self, id: &str) {
        self.problems.clear();
        if self.sources.iter().any(|s| s.id == id && s.reference) {
            self.problems
                .push(Problem::Source(SourceError::ReferenceSource));
            return;
        }
        match self.backend.remove_source(id) {
            Ok(()) => self.refresh(),
            Err(e) => self.report(&e),
        }
    }

    /// Starts the job the button of `name` stands for: `Pending`. `None` (and a problem when busy) if there is
    /// nothing to start. Run the job with the backend, tell [`StoreModel::observe`], then [`StoreModel::finish`].
    pub fn begin_job(&mut self, name: &str) -> Option<Ticket<'_>> {
        if let Some(job) = self.job.as_ref().filter(|j| !j.is_finished()) {
            let busy = Problem::Busy(job.name.clone());
            self.problems = vec![busy];
            return None;
        }
        let kind = match self
            .available
            .iter()
            .find(|r| r.package.name == name)?
            .action
        {
            Action::Install => JobKind::Install,
            Action::Update => JobKind::Update,
            Action::UpToDate => return None,
        };
        self.problems.clear();
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

    /// The backend is done: `Installed`, or `Failed` and then `RolledBack`. Reads the lists again.
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
                    job.error = Some(error);
                }
            }
        }
        self.read_installed();
        self.read_available();
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

    /// One sentence about the job, or about there being none.
    #[must_use]
    pub fn job_line(&self) -> String {
        let Some(job) = &self.job else {
            return words::NO_JOB.to_owned();
        };
        let version = job.version.map(|v| v.to_string()).unwrap_or_default();
        let values = [("name", job.name.as_str()), ("version", version.as_str())];
        let mut line = words::fill(words::job_phase(job.phase), &values);
        if let Some(error) = &job.error {
            let detail = match error {
                JobError::Other(text) => text.as_str(),
                _ => "",
            };
            line.push(' ');
            line.push_str(&words::fill(
                words::job_problem(error),
                &[("name", job.name.as_str()), ("detail", detail)],
            ));
        }
        line
    }

    /// The sentences for [`StoreModel::problems`].
    #[must_use]
    pub fn problem_lines(&self) -> Vec<String> {
        self.problems
            .iter()
            .map(|p| match p {
                Problem::Source(e) => words::source_problem(e).to_owned(),
                Problem::Backend(detail) => {
                    words::fill(words::STORE_PROBLEM, &[("detail", detail)])
                }
                Problem::Unreadable { source, detail } => words::fill(
                    words::SOURCE_UNREADABLE,
                    &[("source", source), ("detail", detail)],
                ),
                Problem::Skipped { id, reason } => {
                    words::fill(words::ENTRY_SKIPPED, &[("id", id), ("reason", reason)])
                }
                Problem::Busy(name) => words::fill(words::JOB_BUSY, &[("name", name)]),
            })
            .collect()
    }
}
