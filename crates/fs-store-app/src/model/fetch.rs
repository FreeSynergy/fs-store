//! Pattern: none (a full read of the backend as plain data, made without the model so the window need not hold
//! its lock while the catalogs are fetched).

use std::collections::HashMap;

use crate::backend::{
    BackendError, InstalledDetails, InstalledRow, Listing, SourceRow, StoreBackend,
};

pub(super) type Installed =
    Result<(Vec<InstalledRow>, HashMap<String, InstalledDetails>), BackendError>;

/// What the backend said to a full read; made without the model, so the window need not hold its lock.
pub struct Fetched {
    pub(super) sources: Result<Vec<SourceRow>, BackendError>,
    pub(super) listing: Result<Listing, BackendError>,
    pub(super) installed: Installed,
}

/// Reads sources, offers and what is installed (slow: it fetches the catalogs). Hand the result to
/// [`StoreModel::apply_fetched`].
#[must_use]
pub fn fetch(backend: &dyn StoreBackend) -> Fetched {
    Fetched {
        sources: backend.sources(),
        listing: backend.available(),
        installed: fetch_installed(backend),
    }
}

pub(super) fn fetch_installed(backend: &dyn StoreBackend) -> Installed {
    let list = backend.installed()?;
    let mut rows: Vec<InstalledRow> = Vec::new();
    for row in list {
        match rows.iter_mut().find(|r| r.name == row.name) {
            Some(known) if known.version < row.version => known.version = row.version,
            Some(_) => {}
            None => rows.push(row),
        }
    }
    let details = rows
        .iter()
        .map(|r| (r.name.clone(), backend.details(&r.name)))
        .collect();
    Ok((rows, details))
}
