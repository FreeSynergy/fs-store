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
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: ThemeCategory = ThemeCategory;
        &CAT
    }
}
