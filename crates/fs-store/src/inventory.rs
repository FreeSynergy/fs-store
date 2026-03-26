// inventory.rs — Inventory: the central hub object.
//
// The Inventory is the single source of truth for the local system state.
// It combines:
//   - All packages available in the Store (read via StoreReader)
//   - All packages currently installed locally (InstallRecord list)
//   - Settings
//   - The BootableInstaller reference (init/ package)
//
// The Inventory is what the Bus, GUI, CLI, API, and Node all talk to.
// Nobody talks to StoreReader or InstallRecord directly in normal operation.
//
// Persistence:
//   Install records are stored at `settings.storage.data_dir/records.toml`.
//   Call `save_records()` after any install/remove mutation.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::init::BootableInstaller;
use crate::install_record::InstallRecord;
use crate::package::Package;
use crate::reader::StoreReader;
use crate::release::PackageRelease;
use crate::settings::StoreSettings;

// ── RecordsFile ───────────────────────────────────────────────────────────────

/// On-disk format for `records.toml` — a flat list of all install records.
#[derive(Debug, Default, Serialize, Deserialize)]
struct RecordsFile {
    #[serde(default)]
    records: Vec<InstallRecord>,
}

// ── NamespaceMap ──────────────────────────────────────────────────────────────

/// All packages in the Store, grouped by namespace.
///
/// The structure mirrors the Store repository layout exactly:
/// `packages/apps/`, `packages/containers/`, `bundles/`, etc.
/// No mapping or translation — 1:1 with the Store.
#[derive(Default)]
pub struct NamespaceMap {
    pub apps: Vec<Arc<dyn Package>>,
    pub managers: Vec<Arc<dyn Package>>,
    pub containers: Vec<Arc<dyn Package>>,
    pub themes: Vec<Arc<dyn Package>>,
    pub widgets: Vec<Arc<dyn Package>>,
    pub tasks: Vec<Arc<dyn Package>>,
    pub languages: Vec<Arc<dyn Package>>,
    pub icons: Vec<Arc<dyn Package>>,
    pub bundles: Vec<Arc<dyn Package>>,
    pub externals: Vec<Arc<dyn Package>>,
    pub repos: Vec<Arc<dyn Package>>,
}

impl NamespaceMap {
    /// All packages across all namespaces as a flat iterator.
    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn Package>> {
        self.apps
            .iter()
            .chain(&self.managers)
            .chain(&self.containers)
            .chain(&self.themes)
            .chain(&self.widgets)
            .chain(&self.tasks)
            .chain(&self.languages)
            .chain(&self.icons)
            .chain(&self.bundles)
            .chain(&self.externals)
            .chain(&self.repos)
    }

    /// Find a package by id across all namespaces.
    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<&Arc<dyn Package>> {
        self.all().find(|p| p.id() == id)
    }

    /// Total number of packages across all namespaces.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.all().count()
    }

    /// Return the namespace name for a package id, e.g. `"containers"`.
    ///
    /// Returns `None` if the id is not found in any namespace.
    #[must_use]
    pub fn namespace_of(&self, id: &str) -> Option<&'static str> {
        if self.apps.iter().any(|p| p.id() == id) {
            return Some("apps");
        }
        if self.managers.iter().any(|p| p.id() == id) {
            return Some("managers");
        }
        if self.containers.iter().any(|p| p.id() == id) {
            return Some("containers");
        }
        if self.themes.iter().any(|p| p.id() == id) {
            return Some("themes");
        }
        if self.widgets.iter().any(|p| p.id() == id) {
            return Some("widgets");
        }
        if self.tasks.iter().any(|p| p.id() == id) {
            return Some("tasks");
        }
        if self.languages.iter().any(|p| p.id() == id) {
            return Some("languages");
        }
        if self.icons.iter().any(|p| p.id() == id) {
            return Some("icons");
        }
        if self.bundles.iter().any(|p| p.id() == id) {
            return Some("bundles");
        }
        if self.externals.iter().any(|p| p.id() == id) {
            return Some("externals");
        }
        if self.repos.iter().any(|p| p.id() == id) {
            return Some("repos");
        }
        None
    }
}

// ── PackageState ──────────────────────────────────────────────────────────────

/// The combined view of one package: what the Store offers vs. what is installed.
///
/// This is the primary object the GUI, CLI, and API work with.
/// It answers questions like "is this installed?", "is an update available?",
/// "which version is active?".
pub struct PackageState {
    /// The package descriptor from the Store catalog.
    pub package: Arc<dyn Package>,

    /// Available releases from the Store (newest first).
    pub available: Vec<PackageRelease>,

    /// Locally installed instances (may be multiple versions).
    pub installed: Vec<InstallRecord>,
}

impl PackageState {
    /// `true` when at least one version of this package is installed.
    #[must_use]
    pub fn is_installed(&self) -> bool {
        !self.installed.is_empty()
    }

    /// The currently active install record, if any.
    #[must_use]
    pub fn active(&self) -> Option<&InstallRecord> {
        self.installed.iter().find(|r| r.is_active)
    }

    /// `true` when a newer version is available than the active installed one.
    #[must_use]
    pub fn has_update(&self) -> bool {
        let Some(active) = self.active() else {
            return false;
        };
        let Some(latest) = self.available.first() else {
            return false;
        };
        latest.version != active.version
    }

    /// The newest available release, if any.
    #[must_use]
    pub fn latest_available(&self) -> Option<&PackageRelease> {
        self.available.first()
    }
}

// ── Inventory ─────────────────────────────────────────────────────────────────

/// The central domain object of the `FreeSynergy` Store.
///
/// All consumers (Bus, GUI, CLI, API, Node) interact through the Inventory.
/// It owns the namespace map, the install records, and the settings.
///
/// # Usage
///
/// ```ignore
/// let mut inventory = Inventory::new(StoreSettings::default());
/// inventory.load(&mut reader).await?;
/// let state = inventory.package_state("forgejo");
/// ```
pub struct Inventory {
    /// All packages available in the Store, by namespace.
    pub namespaces: NamespaceMap,

    /// Combined package state (available + installed) for every known package.
    pub states: Vec<PackageState>,

    /// Store configuration.
    pub settings: StoreSettings,

    /// The bootable USB installer, if available in the Store.
    pub init: Option<Arc<dyn BootableInstaller>>,
}

impl Inventory {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create an empty inventory with the given settings.
    #[must_use]
    pub fn new(settings: StoreSettings) -> Self {
        Self {
            namespaces: NamespaceMap::default(),
            states: Vec::new(),
            settings,
            init: None,
        }
    }

    // ── Loading ───────────────────────────────────────────────────────────────

    /// Fetch the Store catalog and merge it with local install records.
    ///
    /// After this call, `namespaces` contains all Store packages and `states`
    /// contains a [`PackageState`] for each package, with install records
    /// attached for any that are locally installed.
    ///
    /// # Errors
    ///
    /// Returns an error if loading the catalog or reading install records fails.
    pub async fn load(&mut self, reader: &StoreReader) -> Result<()> {
        self.namespaces = reader.load_all().await?;
        let records = self.read_records()?;
        self.states = self.build_states(&records);
        info!(
            "Inventory: {} packages, {} installed",
            self.states.len(),
            self.installed().count()
        );
        Ok(())
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Find the [`PackageState`] for a package by id.
    #[must_use]
    pub fn package_state(&self, id: &str) -> Option<&PackageState> {
        self.states.iter().find(|s| s.package.id() == id)
    }

    /// All packages that are currently installed.
    pub fn installed(&self) -> impl Iterator<Item = &PackageState> {
        self.states.iter().filter(|s| s.is_installed())
    }

    /// All packages that have an update available.
    pub fn with_updates(&self) -> impl Iterator<Item = &PackageState> {
        self.states.iter().filter(|s| s.has_update())
    }

    // ── Record mutations ──────────────────────────────────────────────────────

    /// Register a newly installed package version.
    ///
    /// Any previously active record for the same package is deactivated.
    /// Call [`save_records`] afterwards to persist.
    ///
    /// [`save_records`]: Inventory::save_records
    pub fn record_installed(&mut self, record: InstallRecord) {
        for state in &mut self.states {
            if state.package.id() == record.package_id {
                for r in &mut state.installed {
                    r.is_active = false;
                }
                state.installed.push(record);
                return;
            }
        }
    }

    /// Remove the install record for a specific package version.
    ///
    /// If no active record remains, the newest remaining version is activated.
    /// Call [`save_records`] afterwards to persist.
    ///
    /// [`save_records`]: Inventory::save_records
    pub fn record_removed(&mut self, package_id: &str, version: &str) {
        for state in &mut self.states {
            if state.package.id() == package_id {
                state.installed.retain(|r| r.version != version);
                // Re-activate the newest remaining record if needed.
                if !state.installed.iter().any(|r| r.is_active) {
                    if let Some(last) = state.installed.last_mut() {
                        last.is_active = true;
                    }
                }
                return;
            }
        }
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Write all install records to disk.
    ///
    /// Records are stored at `settings.storage.data_dir/records.toml`.
    /// The directory is created if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if creating the directory or writing the file fails.
    pub fn save_records(&self) -> Result<()> {
        let path = self.records_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory '{}'", parent.display()))?;
        }
        let records: Vec<InstallRecord> = self
            .states
            .iter()
            .flat_map(|s| s.installed.iter().cloned())
            .collect();
        let file = RecordsFile { records };
        let text = toml::to_string_pretty(&file).context("serializing install records")?;
        std::fs::write(&path, &text)
            .with_context(|| format!("writing install records '{}'", path.display()))?;
        Ok(())
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn records_path(&self) -> std::path::PathBuf {
        self.settings.storage.data_dir.join("records.toml")
    }

    fn read_records(&self) -> Result<Vec<InstallRecord>> {
        let path = self.records_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading install records '{}'", path.display()))?;
        let file: RecordsFile = toml::from_str(&text)
            .with_context(|| format!("parsing install records '{}'", path.display()))?;
        Ok(file.records)
    }

    fn build_states(&self, records: &[InstallRecord]) -> Vec<PackageState> {
        self.namespaces
            .all()
            .map(|pkg| {
                let installed = records
                    .iter()
                    .filter(|r| r.package_id == pkg.id())
                    .cloned()
                    .collect();
                PackageState {
                    package: Arc::clone(pkg),
                    available: pkg.releases().to_vec(),
                    installed,
                }
            })
            .collect()
    }
}
