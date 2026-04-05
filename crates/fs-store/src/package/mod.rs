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
pub use container::{ApiEndpoint, ContainerPackage, StoragePaths};
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

    /// Optional secondary (badge) icon path — shown overlapping the primary icon
    /// to indicate that this is one of multiple running instances of the same program.
    ///
    /// Maps to `secondary_icon = "badge.svg"` in `package.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_icon_path: Option<String>,

    /// How far the secondary icon overlaps the primary (0.0 = outside, 1.0 = centred on top).
    ///
    /// Clamped to `[0.0, 1.0]` at read time.  Defaults to `0.3`.
    /// Maps to `overlap_factor = 0.3` in `package.toml`.
    #[serde(default = "default_overlap_factor")]
    pub overlap_factor: f32,

    /// Optional user-defined display name for an installed instance of this package.
    ///
    /// Mirrors `InstalledResource::caption` in `fs-inventory`.  Stored in `package.toml`
    /// so the Store catalog can provide a suggested caption for well-known multi-instance
    /// packages (e.g. `"wiki.team-a"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,

    /// Search and filter tags.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Available releases, newest first.
    #[serde(default)]
    pub releases: Vec<PackageRelease>,

    /// Help file metadata for this package.
    #[serde(default)]
    pub help: PackageHelp,

    /// SPDX license identifier, e.g. `"MIT"`.
    #[serde(default)]
    pub license: String,

    /// Upstream homepage URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Store-relative paths to screenshot images (PNG/JPEG).
    #[serde(default)]
    pub screenshots: Vec<String>,

    /// URL to the changelog / release notes for this package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changelog_url: Option<String>,
}

fn default_overlap_factor() -> f32 {
    0.3
}

// ── Package trait ─────────────────────────────────────────────────────────────

/// The base interface for every package in the `FreeSynergy` Store.
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

    /// Medium-length description for the detail view.
    fn description(&self) -> &str;

    /// Store-relative path to this package's SVG icon.
    fn icon_path(&self) -> Option<&str>;

    /// Search/filter tags.
    fn tags(&self) -> &[String];

    /// All available releases for this package, newest first.
    fn releases(&self) -> &[PackageRelease];

    /// Help file metadata (paths and available locales).
    fn help(&self) -> &PackageHelp;

    /// SPDX license identifier, e.g. `"MIT"`.
    #[allow(clippy::unnecessary_literal_bound)]
    fn license(&self) -> &str {
        ""
    }

    /// Upstream homepage URL, if declared.
    fn homepage(&self) -> Option<&str> {
        None
    }

    /// Store-relative screenshot paths, if any.
    fn screenshots(&self) -> &[String] {
        &[]
    }

    /// URL to the changelog / release notes.
    fn changelog_url(&self) -> Option<&str> {
        None
    }

    /// Store-relative path to the secondary (badge) SVG icon, if any.
    ///
    /// When `Some`, the engine renders this icon slightly overlapping `icon_path`
    /// (at `overlap_factor` ratio) to indicate that multiple instances exist.
    fn secondary_icon_path(&self) -> Option<&str> {
        None
    }

    /// Overlap factor for the secondary icon — value in `[0.0, 1.0]`.
    ///
    /// `0.0` = secondary entirely outside primary bounds.
    /// `1.0` = secondary centred on top of primary.
    fn overlap_factor(&self) -> f32 {
        0.3
    }

    /// Suggested display name for an installed instance of this package.
    fn caption(&self) -> Option<&str> {
        None
    }

    /// Filesystem paths reserved by this package.
    ///
    /// Non-container packages return `None`.
    fn storage(&self) -> Option<&StoragePaths> {
        None
    }

    /// REST API endpoints exposed by this package.
    ///
    /// Non-container packages return an empty slice.
    fn api_endpoints(&self) -> &[ApiEndpoint] {
        &[]
    }

    /// The latest available release, if any.
    fn latest_release(&self) -> Option<&PackageRelease> {
        self.releases().first()
    }

    /// The latest version string, if any.
    fn latest_version(&self) -> Option<&str> {
        self.latest_release().map(|r| r.version.as_str())
    }
}
