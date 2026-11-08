//! Pattern: Command (each method is one action of the sources view; the backend does the work, then the model reads again).

use fs_store_core::SourceError;

use super::store::StoreModel;
use super::types::Problem;

impl StoreModel {
    /// Adds the source typed in the field. An empty field is told to the person.
    pub fn add_source(&mut self) {
        let url = self.new_source.trim().to_owned();
        self.problems.clear();
        if url.is_empty() {
            self.problems.push(Problem::EmptyAddress);
            return;
        }
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
}
