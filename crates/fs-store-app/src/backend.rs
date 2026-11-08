//! Pattern: Strategy / Port. What the window needs from the store, shaped like `store.capnp`
//! (`sources`, `addSource`, `setEnabled`, `removeSource`, `available`, `installed`, `install`/`update` with
//! phases). The in-process engine implements it now; a Cap'n Proto client can do it later.

use fs_store_core::{InstallState, Kind, SourceError, Version};

/// The phase of a job; the same six as `JobPhase` in the contract.
pub type Phase = InstallState;

/// A source in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRow {
    /// Id.
    pub id: String,
    /// Git address.
    pub url: String,
    /// Whether its packages are offered.
    pub enabled: bool,
    /// The built-in FreeSynergy source.
    pub reference: bool,
}

/// A package a source offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRow {
    /// Name (`AppStream` component id).
    pub name: String,
    /// Kind.
    pub kind: Kind,
    /// Offered version.
    pub version: Version,
    /// Id of the source.
    pub source: String,
}

/// A package on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledRow {
    /// Name.
    pub name: String,
    /// Active version.
    pub version: Version,
}

/// How an installed package came onto this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    /// Installed by the system package manager (rpm).
    Rpm,
    /// Unpacked by the store below its own folder (tar.gz).
    Archive,
}

/// What is known about where an installed package is. Only facts that were found; nothing is guessed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstalledDetails {
    /// How it was installed (both, if both are found).
    pub kinds: Vec<InstallKind>,
    /// Commands that exist: (name, full path).
    pub commands: Vec<(String, String)>,
    /// The folder the store unpacked the active version into, if it did.
    pub folder: Option<String>,
    /// Git address of the source (from the register).
    pub source: Option<String>,
    /// Facet commands from the register.
    pub facets: Vec<String>,
}

/// What the enabled sources offer, and what they could not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Listing {
    /// Offered packages.
    pub packages: Vec<PackageRow>,
    /// Catalog entries the store refused to offer: (id, reason). Never dropped silently.
    pub skipped: Vec<(String, String)>,
    /// Sources that could not be read: (source id, detail).
    pub unreadable: Vec<(String, String)>,
}

/// What a job does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    /// Install a package that is not here yet.
    Install,
    /// Replace an installed package by a newer version.
    Update,
}

/// Why a call to the store failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// A rule of the source list.
    Source(SourceError),
    /// Anything else, as a developer text.
    Other(String),
}

/// Why a job failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// The program runs and was not replaced.
    Running,
    /// The signature of the catalog is not good.
    CatalogSignature,
    /// The signature of the release file is not good.
    ReleaseSignature,
    /// The checksum differs from the catalog.
    Checksum,
    /// The catalog offers an older version than the installed one.
    Downgrade,
    /// No enabled source offers the package.
    NotOffered,
    /// There is no installer for this format yet.
    Unsupported(String),
    /// The update failed and the previous version could not be restored.
    RollbackFailed,
    /// Anything else, as a developer text.
    Other(String),
}

/// The store, as the window sees it.
pub trait StoreBackend: Send + Sync {
    /// All sources, reference first.
    ///
    /// # Errors
    /// `BackendError` when the list cannot be read.
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError>;
    /// Adds a source by its Git address.
    ///
    /// # Errors
    /// `Source(Duplicate)` or `Other`.
    fn add_source(&self, url: &str) -> Result<SourceRow, BackendError>;
    /// Switches a source on or off; allowed for the reference source too.
    ///
    /// # Errors
    /// `Source(NotFound)` or `Other`.
    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError>;
    /// Removes a source.
    ///
    /// # Errors
    /// `Source(ReferenceSource)` for the built-in one, `Source(NotFound)` or `Other`.
    fn remove_source(&self, id: &str) -> Result<(), BackendError>;
    /// Packages of the enabled sources.
    ///
    /// # Errors
    /// `BackendError` when nothing can be listed at all.
    fn available(&self) -> Result<Listing, BackendError>;
    /// Packages on this machine.
    ///
    /// # Errors
    /// `BackendError` when the list cannot be read.
    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError>;
    /// What is known about where the installed package `name` is. Default: nothing is known.
    fn details(&self, _name: &str) -> InstalledDetails {
        InstalledDetails::default()
    }
    /// Runs one job and tells `observe` each phase passed (`Verified`, `Installing`). The ending is the
    /// return value: the new version (`Installed`), or the reason (`Failed`, then `RolledBack`).
    ///
    /// # Errors
    /// `JobError`.
    fn run(
        &self,
        kind: JobKind,
        name: &str,
        observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError>;
}
