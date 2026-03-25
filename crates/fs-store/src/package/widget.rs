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
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: WidgetCategory = WidgetCategory;
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
