// theme.rs — ThemePackage: visual themes (colors, fonts, window chrome).

use serde::{Deserialize, Serialize};

use crate::category::{PackageCategory, ThemeCategory};
use crate::package::mod_prelude::*;

/// A visual theme package.
///
/// Themes contain CSS variable overrides, font sets, and window chrome styles.
/// They have no additional domain fields beyond the common [`PackageData`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,
}

impl Package for ThemePackage {
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: ThemeCategory = ThemeCategory;
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
