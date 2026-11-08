//! Pattern: Aggregate root (the state of the window; `sources`, `jobs` and `text` add commands and sentences).

use std::collections::HashMap;
use std::sync::Arc;

use crate::backend::{
    BackendError, InstalledDetails, InstalledRow, PackageRow, SourceRow, StoreBackend,
};

use crate::updates::{NoUpdates, UpdateService};
use fs_store_sources::{Findings, Settings};

use super::cards_ui::CardsUi;
use super::jobs::JobView;
use super::types::{Action, AvailableRow, Problem, View};
use super::updates::CheckState;

/// State of the window.
pub struct StoreModel {
    pub(super) backend: Arc<dyn StoreBackend>,
    pub(super) view: View,
    pub(super) sources: Vec<SourceRow>,
    pub(super) available: Vec<AvailableRow>,
    pub(super) installed: Vec<InstalledRow>,
    pub(super) offers: Vec<PackageRow>,
    pub(super) details: HashMap<String, InstalledDetails>,
    pub(super) job: Option<JobView>,
    pub(super) problems: Vec<Problem>,
    pub(super) new_source: String,
    pub(super) updates: Arc<dyn UpdateService>,
    pub(super) settings: Settings,
    pub(super) findings: Findings,
    pub(super) check: CheckState,
    pub(super) shown: Option<View>,
    pub(super) fresh: Vec<String>,
    pub(super) ui: CardsUi,
}

impl StoreModel {
    /// A model over `backend`, showing the page of cards; nothing is read yet (see [`StoreModel::refresh`]).
    #[must_use]
    pub fn new(backend: Arc<dyn StoreBackend>) -> Self {
        Self {
            backend,
            view: View::Cards,
            sources: Vec::new(),
            available: Vec::new(),
            installed: Vec::new(),
            offers: Vec::new(),
            details: HashMap::new(),
            job: None,
            problems: Vec::new(),
            new_source: String::new(),
            updates: Arc::new(NoUpdates),
            settings: Settings {
                notify: true,
                ..Settings::default()
            },
            findings: Findings::default(),
            check: CheckState::Idle,
            shown: None,
            fresh: Vec::new(),
            ui: CardsUi::default(),
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

    /// Show a view. Entering the page of cards (not before) marks the new programs as seen; they stay
    /// marked on their cards until the page is left.
    pub fn show(&mut self, view: View) {
        self.view = view;
        if self.shown == Some(view) {
            return;
        }
        self.shown = Some(view);
        if view == View::Cards {
            self.enter_available();
        } else {
            self.fresh.clear();
        }
    }

    /// The sources, reference first.
    #[must_use]
    pub fn sources(&self) -> &[SourceRow] {
        &self.sources
    }

    /// What the sources offer, one row per name, with the action it allows.
    #[must_use]
    pub fn available(&self) -> &[AvailableRow] {
        &self.available
    }

    /// What is installed, one row per name.
    #[must_use]
    pub fn installed(&self) -> &[InstalledRow] {
        &self.installed
    }

    /// Where the installed package `name` is (empty if nothing was found).
    #[must_use]
    pub fn details(&self, name: &str) -> InstalledDetails {
        self.details.get(name).cloned().unwrap_or_default()
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
        self.read_available();
        self.read_installed();
        self.merge();
        self.load_updates();
    }

    fn read_sources(&mut self) {
        match self.backend.sources() {
            Ok(list) => self.sources = list,
            Err(e) => self.report(&e),
        }
    }

    /// Reads what is installed; one row per name (the newest version if a backend names it twice).
    pub(super) fn read_installed(&mut self) {
        match self.backend.installed() {
            Ok(list) => {
                let mut rows: Vec<InstalledRow> = Vec::new();
                for row in list {
                    match rows.iter_mut().find(|r| r.name == row.name) {
                        Some(known) if known.version < row.version => known.version = row.version,
                        Some(_) => {}
                        None => rows.push(row),
                    }
                }
                self.details = rows
                    .iter()
                    .map(|r| (r.name.clone(), self.backend.details(&r.name)))
                    .collect();
                self.installed = rows;
            }
            Err(e) => self.report(&e),
        }
    }

    /// Reads the offers of the enabled sources; what could not be read or is not offered becomes problems.
    pub(super) fn read_available(&mut self) {
        let listing = match self.backend.available() {
            Ok(listing) => listing,
            Err(e) => {
                self.offers.clear();
                self.report(&e);
                return;
            }
        };
        self.offers = listing.packages;
        for (source, detail) in listing.unreadable {
            self.problems.push(Problem::Unreadable { source, detail });
        }
        for (id, reason) in listing.skipped {
            self.problems.push(Problem::Skipped { id, reason });
        }
    }

    /// Joins offers and installed: one row per name (the newest offer), with the action it allows. A program
    /// that is installed is never offered as installable.
    pub(super) fn merge(&mut self) {
        let mut best: Vec<&PackageRow> = Vec::new();
        for offer in &self.offers {
            match best.iter_mut().find(|p| p.name == offer.name) {
                Some(known) if known.version < offer.version => *known = offer,
                Some(_) => {}
                None => best.push(offer),
            }
        }
        self.available = best
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
                    package: package.clone(),
                    installed,
                    action,
                }
            })
            .collect();
    }

    pub(super) fn report(&mut self, error: &BackendError) {
        self.problems.push(match error {
            BackendError::Source(e) => Problem::Source(e.clone()),
            BackendError::Other(text) => Problem::Backend(text.clone()),
        });
    }
}
