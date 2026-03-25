// bundle.rs — BundlePackage: meta-packages that aggregate other packages.

use serde::{Deserialize, Serialize};

use crate::category::{BundleCategory, PackageCategory};
use crate::package::mod_prelude::*;

/// A reference to a component package within a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleComponent {
    /// Package id of the component, e.g. `"zentinel"`.
    pub id: String,

    /// Optional pinned version. If absent, the latest is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Whether this component is required (removing it removes the bundle).
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// A bundle that groups multiple packages for combined install/remove.
///
/// Bundles are the Store's equivalent of a "meta-package": installing a bundle
/// installs all of its required components. Optional components can be toggled
/// individually.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// The component packages this bundle includes.
    #[serde(default)]
    pub components: Vec<BundleComponent>,
}

impl Package for BundlePackage {
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: BundleCategory = BundleCategory;
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

/// Bundle-specific extension methods.
pub trait BundleExt: Package {
    /// Component packages in this bundle.
    fn components(&self) -> &[BundleComponent];

    /// The required components (removing these removes the bundle).
    fn required_components(&self) -> Vec<&BundleComponent> {
        self.components().iter().filter(|c| c.required).collect()
    }
}

impl BundleExt for BundlePackage {
    fn components(&self) -> &[BundleComponent] {
        &self.components
    }
}
