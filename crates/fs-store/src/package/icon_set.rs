// icon_set.rs — IconSetPackage: SVG icon sets.

use serde::{Deserialize, Serialize};

use crate::category::{IconSetCategory, PackageCategory};
use crate::package::mod_prelude::*;

/// An SVG icon set package.
///
/// Icon sets can originate from community sources (Homarr, Simple Icons)
/// or be custom-built. They are stored locally once installed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSetPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Number of icons in this set, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_count: Option<u32>,

    /// Original upstream source URL (for attribution), if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_url: Option<String>,
}

impl Package for IconSetPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: IconSetCategory = IconSetCategory;
        &CAT
    }
}
