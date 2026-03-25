// widget.rs — WidgetPackage: desktop widgets.

use serde::{Deserialize, Serialize};

use crate::category::{PackageCategory, WidgetCategory};
use crate::package::mod_prelude::*;

/// A desktop widget package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,
}

impl Package for WidgetPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: WidgetCategory = WidgetCategory;
        &CAT
    }
}
