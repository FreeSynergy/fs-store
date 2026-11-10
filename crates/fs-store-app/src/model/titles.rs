//! Pattern: none (the names a person reads, and the update list of the start page: pure functions of the state).

use fs_store_core::Version;

use crate::words;

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
        // The offer the cards show (the newest version of the name), else any offer of it.
        let given = self
            .available
            .iter()
            .map(|r| &r.package)
            .chain(self.offers.iter())
            .find(|p| p.name == id)
            .map(|p| p.title.as_str());
        fs_store_core::display_name(id, given)
    }

    /// One line per update for the "Updates" block, from [`StoreModel::update_rows`]: "name: from -> to", and
    /// that it is queued when it is.
    #[must_use]
    pub fn update_lines(&self) -> Vec<String> {
        self.update_rows()
            .iter()
            .map(|row| {
                let line = words::fill(
                    words::UPDATE_LINE,
                    &[
                        ("name", &row.title),
                        ("from", &row.from.to_string()),
                        ("to", &row.to.to_string()),
                    ],
                );
                if row.queued {
                    format!("{line} ({})", words::UPDATE_QUEUED)
                } else {
                    line
                }
            })
            .collect()
    }

    /// The programs with an update waiting, sorted by name: the "Updates" block of the start page.
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
