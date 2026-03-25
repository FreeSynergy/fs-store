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

use std::sync::Arc;

use crate::init::BootableInstaller;
use crate::install_record::InstallRecord;
use crate::package::Package;
use crate::release::PackageRelease;
use crate::settings::StoreSettings;

// ── NamespaceMap ──────────────────────────────────────────────────────────────

/// All packages in the Store, grouped by namespace.
///
/// The structure mirrors the Store repository layout exactly:
/// `packages/apps/`, `packages/containers/`, `bundles/`, etc.
/// No mapping or translation — 1:1 with the Store.
#[derive(Default)]
pub struct NamespaceMap {
    pub apps: Vec<Arc<dyn Package>>,
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
    pub fn find_by_id(&self, id: &str) -> Option<&Arc<dyn Package>> {
        self.all().find(|p| p.id() == id)
    }

    /// Total number of packages across all namespaces.
    pub fn total_count(&self) -> usize {
        self.all().count()
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
    pub fn is_installed(&self) -> bool {
        !self.installed.is_empty()
    }

    /// The currently active install record, if any.
    pub fn active(&self) -> Option<&InstallRecord> {
        self.installed.iter().find(|r| r.is_active)
    }

    /// `true` when a newer version is available than the active installed one.
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
    pub fn latest_available(&self) -> Option<&PackageRelease> {
        self.available.first()
    }
}

// ── Inventory ─────────────────────────────────────────────────────────────────

/// The central domain object of the FreeSynergy Store.
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
    /// Create an empty inventory with the given settings.
    pub fn new(settings: StoreSettings) -> Self {
        Self {
            namespaces: NamespaceMap::default(),
            states: Vec::new(),
            settings,
            init: None,
        }
    }

    /// Find the `PackageState` for a package by id.
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
}
