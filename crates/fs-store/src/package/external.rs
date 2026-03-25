// external.rs — ExternalPackage: non-self-hostable external services.
//
// External packages are services like Telegram or Discord that cannot be
// self-hosted. They appear in the Store so the Bus can route messages to them
// via their API capabilities, and so the UI can display their role in the
// ecosystem. They have no installation — only links.

use serde::{Deserialize, Serialize};

use crate::category::{ExternalCategory, PackageCategory};
use crate::package::mod_prelude::*;

/// An external service that cannot be self-hosted.
///
/// Extends [`Package`] with link metadata. The `releases` list is always
/// empty for external packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Main website URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Download or sign-up URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download: Option<String>,

    /// API / developer documentation URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

impl Package for ExternalPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: ExternalCategory = ExternalCategory;
        &CAT
    }
}
