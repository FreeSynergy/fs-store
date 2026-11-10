//! The store of this machine: `sources.toml` and `installed.toml` below the store's root.
//!
//! Pattern: Adapter — the store's file-backed crates behind [`StoreView`].

use std::path::{Path, PathBuf};

use fs_store_core::{Format, Kind, SourceRepository};
use fs_store_sources::{FileSources, Registry};

use crate::catalogs;
use crate::view::{Offers, PackageInfo, SourceInfo, StoreView};

/// Version of a package as the system package manager knows it.
type RpmLookup = fn(&str) -> Option<String>;

/// The format the package manager of this system installs (the system's own answer was found, so it is one of them).
fn native_format() -> Format {
    match fs_store_engine::detect_system() {
        fs_store_engine::System::Deb => Format::Deb,
        _ => Format::Rpm,
    }
}

fn rpm_version(name: &str) -> Option<String> {
    fs_store_engine::native_installed_version(name).map(|v| v.to_string())
}

/// The root of the store: `$FS_STORE_ROOT`, else `~/.local/share/fs-store` (as the commands use it).
#[must_use]
pub fn default_root() -> PathBuf {
    std::env::var_os("FS_STORE_ROOT").map_or_else(
        || {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/share/fs-store")
        },
        PathBuf::from,
    )
}

/// The store as the files of this machine tell it.
pub struct LocalStore {
    root: PathBuf,
    rpm: RpmLookup,
    /// What the system package manager is called in answers (deb on Debian and Ubuntu, else rpm).
    native: Format,
}

impl LocalStore {
    /// The store below `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            rpm: rpm_version,
            native: native_format(),
        }
    }

    /// The store below the default root.
    #[must_use]
    pub fn open_default() -> Self {
        Self::new(default_root())
    }

    /// Ask `lookup` instead of `rpm` which version the system has installed (a test).
    #[must_use]
    pub fn with_rpm_lookup(mut self, lookup: RpmLookup) -> Self {
        self.rpm = lookup;
        self.native = Format::Rpm; // the lookup stands in for rpm
        self
    }

    /// The root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn source_list(&self) -> Result<Vec<fs_store_core::Source>, String> {
        let file = FileSources::load(&self.root.join("sources.toml"))
            .map_err(|e| format!("sources.toml: {e:?}"))?;
        Ok(file.list())
    }
}

impl StoreView for LocalStore {
    fn sources(&self) -> Result<Vec<SourceInfo>, String> {
        Ok(self
            .source_list()?
            .into_iter()
            .map(|s| SourceInfo {
                id: s.id.0,
                url: s.url,
                enabled: s.enabled,
                reference: s.reference,
            })
            .collect())
    }

    fn available(&self) -> Result<Offers, String> {
        let enabled: Vec<_> = self
            .source_list()?
            .into_iter()
            .filter(|s| s.enabled)
            .collect();
        Ok(catalogs::read_all(&self.root, &enabled))
    }

    fn installed(&self) -> Result<Vec<PackageInfo>, String> {
        let registry = Registry::load(&self.root.join("installed.toml"))
            .map_err(|e| format!("installed.toml: {e:?}"))?;
        let sources = self.source_list().unwrap_or_default();
        Ok(registry
            .list()
            .iter()
            .map(|e| {
                // The system's own answer wins: an rpm updated by dnf is newer than the register says.
                let by_rpm = (self.rpm)(&e.name);
                PackageInfo {
                    name: e.name.clone(),
                    kind: Kind::Program,
                    format: if by_rpm.is_some() {
                        self.native
                    } else {
                        Format::TarGz
                    },
                    version: by_rpm.unwrap_or_else(|| e.version.clone()),
                    source: sources
                        .iter()
                        .find(|s| s.url == e.source)
                        .map_or_else(|| e.source.clone(), |s| s.id.0.clone()),
                    facets: e.facets.clone(),
                }
            })
            .collect())
    }
}
