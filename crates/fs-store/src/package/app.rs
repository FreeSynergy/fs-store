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
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: AppCategory = AppCategory;
        &CAT
    }
    fn summary(&self) -> &str {
        &self.data.summary
    }
    fn icon_path(&self) -> Option<&str> {
        self.data.icon_path.as_deref()
    }
    fn tags(&self) -> &[String] {
        &self.data.tags
    }
    fn releases(&self) -> &[PackageRelease] {
        &self.data.releases
    }
    fn help(&self) -> &PackageHelp {
        &self.data.help
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
