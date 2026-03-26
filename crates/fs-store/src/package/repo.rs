// repo.rs — RepoPackage: additional Store source repositories.
//
// A repo package points to a community or third-party Store repository.
// Installing it registers the remote as an additional package source.

use serde::{Deserialize, Serialize};

use crate::category::{PackageCategory, RepoCategory};
use crate::package::mod_prelude::*;

/// An additional Store source repository.
///
/// Repos are distributed via the official Store but point to independent
/// repositories that extend the package catalog. Installing one registers
/// it as an additional source for package discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Git URL of the remote repository, e.g.
    /// `"https://github.com/FreeSynergy/fs-store-community"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Branch to track, e.g. `"main"`.
    #[serde(default)]
    pub branch: String,

    /// `true` if the repository is verified by the `FreeSynergy` project.
    #[serde(default)]
    pub verified: bool,
}

impl Package for RepoPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: RepoCategory = RepoCategory;
        &CAT
    }
}

/// Repo-specific extension methods.
pub trait RepoExt: Package {
    /// Git URL of the repository.
    fn url(&self) -> Option<&str>;

    /// Branch being tracked.
    fn branch(&self) -> &str;

    /// `true` if verified by the `FreeSynergy` project.
    fn is_verified(&self) -> bool;
}

impl RepoExt for RepoPackage {
    fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    fn branch(&self) -> &str {
        &self.branch
    }
    fn is_verified(&self) -> bool {
        self.verified
    }
}
