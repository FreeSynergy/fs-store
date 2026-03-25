// language.rs — LanguagePackage: i18n language packs.

use serde::{Deserialize, Serialize};

use crate::category::{LanguageCategory, PackageCategory};
use crate::package::mod_prelude::*;

/// A language pack that provides translations for all installed programs.
///
/// Installing a language pack fetches `.ftl` files for every installed package
/// that declares support for that locale. Help files are only installed for
/// languages the user has explicitly installed (offline-first principle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// BCP-47 locale code this pack covers, e.g. `"de"`, `"ar"`, `"zh-Hant"`.
    pub locale: String,

    /// `true` for right-to-left languages (Arabic, Farsi, Urdu, Pashto).
    #[serde(default)]
    pub rtl: bool,
}

impl Package for LanguagePackage {
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: LanguageCategory = LanguageCategory;
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

/// Language-specific extension methods.
pub trait LanguageExt: Package {
    /// BCP-47 locale code, e.g. `"de"`.
    fn locale(&self) -> &str;

    /// `true` for right-to-left languages.
    fn is_rtl(&self) -> bool;
}

impl LanguageExt for LanguagePackage {
    fn locale(&self) -> &str {
        &self.locale
    }
    fn is_rtl(&self) -> bool {
        self.rtl
    }
}
