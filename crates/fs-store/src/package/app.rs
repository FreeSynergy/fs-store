// app.rs — AppPackage: native binaries distributed via GitHub Releases.

use serde::{Deserialize, Serialize};

use crate::category::{AppCategory, PackageCategory};
use crate::package::mod_prelude::*;

// ── AppPackage ────────────────────────────────────────────────────────────────

/// A native application distributed as a compiled binary.
///
/// Extends [`Package`] with the source repository URL. Download URLs are
/// embedded in each [`PackageRelease::distribution`] map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Source repository URL, e.g. `"https://github.com/FreeSynergy/fs-node"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

impl Package for AppPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: AppCategory = AppCategory;
        &CAT
    }
}

/// App-specific extension methods.
pub trait AppExt: Package {
    /// Source repository URL, if declared in the catalog.
    fn repo(&self) -> Option<&str>;
}

impl AppExt for AppPackage {
    fn repo(&self) -> Option<&str> {
        self.repo.as_deref()
    }
}
