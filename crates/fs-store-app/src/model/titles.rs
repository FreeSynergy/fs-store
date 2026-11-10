//! Pattern: none (the names a person reads, and the update list of the start page: pure functions of the state).

use fs_store_core::Version;

use super::store::StoreModel;
use super::types::Action;

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
}
