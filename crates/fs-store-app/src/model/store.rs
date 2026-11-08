//! Pattern: Aggregate root (the state of the window; `sources`, `jobs` and `text` add commands and sentences).

use std::sync::Arc;

use crate::backend::{BackendError, InstalledRow, SourceRow, StoreBackend};

use super::jobs::JobView;
use super::types::{Action, AvailableRow, Problem, View};

/// State of the window.
pub struct StoreModel {
    pub(super) backend: Arc<dyn StoreBackend>,
    pub(super) view: View,
    pub(super) sources: Vec<SourceRow>,
    pub(super) available: Vec<AvailableRow>,
    pub(super) installed: Vec<InstalledRow>,
    pub(super) job: Option<JobView>,
    pub(super) problems: Vec<Problem>,
    pub(super) new_source: String,
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

    pub(super) fn read_installed(&mut self) {
        match self.backend.installed() {
            Ok(list) => self.installed = list,
            Err(e) => self.report(&e),
        }
    }

    pub(super) fn read_available(&mut self) {
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

    pub(super) fn report(&mut self, error: &BackendError) {
        self.problems.push(match error {
            BackendError::Source(e) => Problem::Source(e.clone()),
            BackendError::Other(text) => Problem::Backend(text.clone()),
        });
    }
}
