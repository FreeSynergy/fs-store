// context.rs — StoreContext: the central shared state for the GUI.
//
// Design: Provider Pattern (OOP rule).
//
//   StoreContext is provided once at the App root via `provide_context`.
//   Every component reads it with `use_context::<Signal<StoreContext>>()`.
//   Business logic (loading, filtering) lives here — components only render.
//
//   StoreContext wraps a Signal<StoreState> which holds the view model:
//   a flat, cloneable list of PackageRow objects derived from the Inventory.

use dioxus::prelude::*;
use fs_store::{Inventory, StoreReader, StoreSettings, StoreSource};

// ── PackageRow ────────────────────────────────────────────────────────────────

/// View model for one package — cloneable, no Arc, no trait objects.
///
/// Built from `PackageState` during load; safe to store in a Signal.
#[derive(Clone, PartialEq)]
pub struct PackageRow {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub version: String,
    pub namespace: &'static str,
    pub icon_path: Option<String>,
    pub tags: Vec<String>,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub has_update: bool,
}

// ── StoreState ────────────────────────────────────────────────────────────────

/// The complete reactive state of the Store GUI.
#[derive(Clone, Default)]
pub struct StoreState {
    pub rows: Vec<PackageRow>,
    pub loading: bool,
    pub error: Option<String>,
    /// Id of the currently selected (detail-view) package.
    pub selected_id: Option<String>,
    /// Active search query.
    pub search: String,
    /// Active namespace filter; `None` = show all.
    pub namespace_filter: Option<&'static str>,
}

impl StoreState {
    pub fn loading() -> Self {
        Self {
            loading: true,
            ..Default::default()
        }
    }

    /// Packages matching the current search + namespace filter.
    pub fn filtered(&self) -> impl Iterator<Item = &PackageRow> {
        let search = self.search.to_lowercase();
        let ns = self.namespace_filter;
        self.rows.iter().filter(move |r| {
            let ns_ok = ns.is_none_or(|n| r.namespace == n);
            let q_ok = search.is_empty()
                || r.id.to_lowercase().contains(&search)
                || r.name.to_lowercase().contains(&search)
                || r.summary.to_lowercase().contains(&search);
            ns_ok && q_ok
        })
    }

    pub fn selected(&self) -> Option<&PackageRow> {
        self.selected_id
            .as_deref()
            .and_then(|id| self.rows.iter().find(|r| r.id == id))
    }

    pub fn installed(&self) -> impl Iterator<Item = &PackageRow> {
        self.rows.iter().filter(|r| r.is_installed)
    }
}

// ── Context helpers ───────────────────────────────────────────────────────────

/// Type alias — used by every component that needs the store state.
pub type StoreSignal = Signal<StoreState>;

/// Provide the Store context at the root and trigger async loading.
///
/// Call once in the App component. Returns the signal so the root
/// can also read state without a second `use_context` call.
pub fn use_store_context() -> StoreSignal {
    let mut state = use_signal(StoreState::loading);
    provide_context(state);

    use_future(move || async move {
        let reader = make_reader();
        let mut inv = Inventory::new(StoreSettings::default());
        match inv.load(&reader).await {
            Ok(()) => {
                let rows = build_rows(&inv);
                state.write().rows = rows;
                state.write().loading = false;
            }
            Err(e) => {
                state.write().loading = false;
                state.write().error = Some(e.to_string());
            }
        }
    });

    state
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn make_reader() -> StoreReader {
    match std::env::var("FS_STORE_LOCAL") {
        Ok(path) => StoreReader::new(StoreSource::Local(path.into())),
        Err(_) => StoreReader::official(),
    }
}

fn build_rows(inv: &Inventory) -> Vec<PackageRow> {
    inv.states
        .iter()
        .map(|s| {
            let ns = inv
                .namespaces
                .namespace_of(s.package.id())
                .unwrap_or("unknown");
            PackageRow {
                id: s.package.id().to_owned(),
                name: s.package.name().to_owned(),
                summary: s.package.summary().to_owned(),
                description: s.package.description().to_owned(),
                version: s.package.latest_version().unwrap_or("-").to_owned(),
                namespace: ns,
                icon_path: s.package.icon_path().map(str::to_owned),
                tags: s.package.tags().to_vec(),
                is_installed: s.is_installed(),
                installed_version: s.active().map(|r| r.version.clone()),
                has_update: s.has_update(),
            }
        })
        .collect()
}
