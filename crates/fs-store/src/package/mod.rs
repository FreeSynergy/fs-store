// package/mod.rs — Package trait and all concrete package types.
//
// Design: Trait composition instead of inheritance (Rust has no inheritance).
//
//   Package          — base trait, every package type implements this
//   AppPackage       — extends Package with binary distribution metadata
//   ContainerPackage — extends Package with ports, variables, features
//   ThemePackage     — extends Package (visual themes, no extra methods needed yet)
//   WidgetPackage    — extends Package with widget-specific metadata
//   TaskPackage      — extends Package with automation template metadata
//   LanguagePackage  — extends Package with locale list
//   IconSetPackage   — extends Package with icon set metadata
//   BundlePackage    — extends Package with component package references
//   ExternalPackage  — extends Package with external link metadata
//
// Each concrete struct holds a `PackageData` for the common fields and adds
// its own type-specific fields. This avoids code duplication without inheritance.

mod mod_prelude;

pub mod app;
pub mod bundle;
pub mod container;
pub mod external;
pub mod icon_set;
pub mod language;
pub mod repo;
pub mod task;
pub mod theme;
pub mod widget;

pub use app::AppPackage;
pub use bundle::BundlePackage;
pub use container::ContainerPackage;
pub use external::ExternalPackage;
pub use icon_set::IconSetPackage;
pub use language::LanguagePackage;
pub use repo::RepoPackage;
pub use task::TaskPackage;
pub use theme::ThemePackage;
pub use widget::WidgetPackage;

use crate::category::PackageCategory;
use crate::release::PackageRelease;
use serde::{Deserialize, Serialize};

// ── PackageHelp ───────────────────────────────────────────────────────────────

/// Metadata about a package's help files stored in the Store.
///
/// The actual FTL content is fetched on demand by [`StoreReader`].
/// Only the paths and available locales are stored here.
///
/// [`StoreReader`]: crate::reader::StoreReader
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageHelp {
    /// Store-relative base path for help files, e.g. `"packages/containers/forgejo"`.
    pub base_path: String,

    /// BCP-47 locale codes for which help files exist, e.g. `["en", "de"]`.
    pub available_locales: Vec<String>,
}

// ── PackageData ───────────────────────────────────────────────────────────────

/// Common fields shared by all package types.
///
/// Each concrete package struct embeds this and delegates the [`Package`] trait
/// methods to it. This is the Rust equivalent of a shared base class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageData {
    /// Unique slug within the namespace, e.g. `"forgejo"`.
    pub id: String,

    /// Human-readable name, e.g. `"Forgejo"`.
    pub name: String,

    /// ≤255 character summary for store listings.
    pub summary: String,

    /// Medium-length description for the detail view.
    #[serde(default)]
    pub description: String,

    /// Store-relative path to the package SVG icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>,

    /// Search and filter tags.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Available releases, newest first.
    #[serde(default)]
    pub releases: Vec<PackageRelease>,

    /// Help file metadata for this package.
    #[serde(default)]
    pub help: PackageHelp,
}

// ── Package trait ─────────────────────────────────────────────────────────────

/// The base interface for every package in the FreeSynergy Store.
///
/// All concrete package types implement this trait. Callers that do not need
/// type-specific behaviour work exclusively through `dyn Package`, enabling
/// polymorphic collections (`Vec<Arc<dyn Package>>`).
///
/// # Rust note
///
/// Rust has no inheritance. Specialisation is expressed through sub-traits:
/// `ContainerPackage: Package`, `AppPackage: Package`, etc.  A type can
/// implement multiple sub-traits (multiple "parents").
pub trait Package: Send + Sync {
    /// Unique slug within the namespace, e.g. `"forgejo"`.
    fn id(&self) -> &str;

    /// Human-readable name, e.g. `"Forgejo"`.
    fn name(&self) -> &str;

    /// The category this package belongs to.
    ///
    /// Returns a `'static` reference because concrete category types are
    /// zero-sized structs — no heap allocation needed.
    fn category(&self) -> &'static dyn PackageCategory;

    /// Short summary (≤255 chars) for store listings.
    fn summary(&self) -> &str;

    /// Store-relative path to this package's SVG icon.
    fn icon_path(&self) -> Option<&str>;

    /// Search/filter tags.
    fn tags(&self) -> &[String];

    /// All available releases for this package, newest first.
    fn releases(&self) -> &[PackageRelease];

    /// Help file metadata (paths and available locales).
    fn help(&self) -> &PackageHelp;

    /// The latest available release, if any.
    fn latest_release(&self) -> Option<&PackageRelease> {
        self.releases().first()
    }

    /// The latest version string, if any.
    fn latest_version(&self) -> Option<&str> {
        self.latest_release().map(|r| r.version.as_str())
    }
}
