// catalog.rs — Raw TOML deserialization types for the FreeSynergy Store catalog.
// Fields mirror the full Store TOML schema; not all are consumed yet.
#![allow(dead_code)]
//
// These structs mirror the on-disk TOML format in FreeSynergy/Store exactly.
// They are consumed only by StoreReader and converted to domain types in package/.
//
// Two levels of catalog file exist in the Store:
//
//   NamespaceIndex   — packages/{namespace}/catalog.toml
//                      ([catalog] header + [[packages]] list of refs)
//
//   RawCatalogEntry  — packages/{namespace}/{name}/catalog.toml
//                      ([package] meta + optional type-specific sections)

use std::collections::HashMap;

use serde::Deserialize;

// ── Namespace index ───────────────────────────────────────────────────────────

/// Deserializes a namespace-level index: `packages/{namespace}/catalog.toml`.
#[derive(Debug, Deserialize)]
pub(crate) struct NamespaceIndex {
    /// Accepts both `[catalog]` (standard) and `[namespace]` (legacy/external).
    #[serde(alias = "namespace")]
    pub catalog: NamespaceMeta,
    #[serde(default)]
    pub packages: Vec<PackageRef>,
}

/// The `[catalog]` section of a namespace index file.
#[derive(Debug, Deserialize)]
pub(crate) struct NamespaceMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    /// Present in `[namespace]` blocks but not used by the reader.
    #[serde(rename = "type", default)]
    pub namespace_type: Option<String>,
}

/// One entry in the `[[packages]]` array — a pointer to a package catalog file.
#[derive(Debug, Deserialize)]
pub(crate) struct PackageRef {
    pub id: String,
    /// Relative path from the namespace directory, e.g. `"kanidm/catalog.toml"`.
    pub catalog: String,
}

// ── Individual package catalog ────────────────────────────────────────────────

/// Deserializes an individual package `catalog.toml`.
///
/// The `[package].type` field determines which type-specific section is present.
/// All optional sections default to `None` when absent.
#[derive(Debug, Deserialize)]
pub(crate) struct RawCatalogEntry {
    pub package: RawPackageMeta,

    #[serde(default)]
    pub source: Option<RawSource>,

    /// `[app]` — present when `type = "app"`.
    #[serde(default)]
    pub app: Option<RawAppSection>,

    /// `[container]` — present when `type = "container"`.
    #[serde(default)]
    pub container: Option<RawContainerSection>,

    /// `[widget]` — present when `type = "widget"`.
    #[serde(default)]
    pub widget: Option<RawWidgetSection>,

    /// `[bundle]` — present when `type = "bundle"`.
    #[serde(default)]
    pub bundle: Option<RawBundleSection>,

    /// `[icon_set]` — present when `type = "icon_set"`.
    #[serde(default)]
    pub icon_set: Option<RawIconSetSection>,

    /// `[language]` — present when `type = "language"`.
    #[serde(default)]
    pub language: Option<RawLanguageSection>,

    /// `[repo]` — present when `type = "repo"`.
    #[serde(default)]
    pub repo: Option<RawRepoSection>,

    /// `[bootstrap]` — present when `type = "bootstrap"`.
    #[serde(default)]
    pub bootstrap: Option<RawBootstrapSection>,

    #[serde(default)]
    pub provides: RawProvides,

    #[serde(default)]
    pub requires: RawRequires,

    #[serde(default)]
    pub variables: Vec<RawVariable>,

    #[serde(default)]
    pub contract: Option<RawContract>,

    /// `[[binaries]]` — individual named binaries shipped in the package.
    #[serde(default)]
    pub binaries: Vec<RawBinary>,

    /// `[distribution]` — per-platform download URL templates.
    #[serde(default)]
    pub distribution: HashMap<String, String>,

    /// `[interfaces]` — which interfaces the package exposes.
    #[serde(default)]
    pub interfaces: Option<RawInterfaces>,

    /// `[storage]` — filesystem paths reserved by this package.
    #[serde(default)]
    pub storage: Option<RawStorage>,

    /// `[api]` — API endpoints exposed by this package.
    #[serde(default)]
    pub api: Option<RawApiSection>,

    /// `[install_targets]` — supported installation methods.
    #[serde(default)]
    pub install_targets: Option<RawInstallTargets>,
}

// ── [package] ─────────────────────────────────────────────────────────────────

/// The `[package]` section shared by every catalog entry.
#[derive(Debug, Deserialize)]
pub(crate) struct RawPackageMeta {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub package_type: String,
    pub version: String,
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_file: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
    /// Set to `true` for system-critical packages that cannot be removed.
    #[serde(default)]
    pub protected: bool,
    #[serde(default)]
    pub origin: Option<RawOrigin>,
    /// Store-relative paths to screenshot images.
    #[serde(default)]
    pub screenshots: Vec<String>,
    /// URL to the changelog or release notes.
    #[serde(default)]
    pub changelog: Option<String>,
}

/// The `[package.origin]` sub-section.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawOrigin {
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub git: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
}

// ── [source] ──────────────────────────────────────────────────────────────────

/// The `[source]` section — where and how to fetch the package binary.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub artifact: Option<String>,
}

// ── Type-specific sections ────────────────────────────────────────────────────

/// `[app]` — native binary metadata.
#[derive(Debug, Deserialize)]
pub(crate) struct RawAppSection {
    #[serde(default)]
    pub binary: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default)]
    pub config_dir: Option<String>,
}

/// `[container]` — container image metadata.
#[derive(Debug, Deserialize)]
pub(crate) struct RawContainerSection {
    #[serde(default)]
    pub image: Option<String>,
}

/// A `{cols, rows}` grid size used in `[widget]`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawGridSize {
    pub cols: u16,
    pub rows: u16,
}

/// `[widget]` — grid size constraints for desktop widgets.
#[derive(Debug, Deserialize)]
pub(crate) struct RawWidgetSection {
    #[serde(default)]
    pub min_size: Option<RawGridSize>,
    #[serde(default)]
    pub max_size: Option<RawGridSize>,
}

/// One component reference inside `[bundle].components`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBundleComponent {
    pub id: String,
    /// Store-relative path to the component's catalog. Informational only.
    #[serde(default)]
    pub catalog: Option<String>,
}

/// `[bundle]` — list of component packages.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBundleSection {
    #[serde(default)]
    pub components: Vec<RawBundleComponent>,
}

/// `[icon_set]` — SVG icon set metadata.
#[derive(Debug, Deserialize)]
pub(crate) struct RawIconSetSection {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub sizes: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// `[language]` — locale and direction metadata.
#[derive(Debug, Deserialize)]
pub(crate) struct RawLanguageSection {
    /// BCP-47 locale code, e.g. `"de-DE"`.
    #[serde(default)]
    pub locale: Option<String>,
    /// Text direction: `"ltr"` or `"rtl"`.
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub name_native: Option<String>,
    #[serde(default)]
    pub coverage: Option<u8>,
    #[serde(default)]
    pub snippets_dir: Option<String>,
}

/// `[repo]` — remote Store repository metadata.
#[derive(Debug, Deserialize)]
pub(crate) struct RawRepoSection {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

/// `[bootstrap]` — USB installer download URLs.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBootstrapSection {
    #[serde(default)]
    pub versions: Vec<String>,
    /// Map of `"platform-arch"` → download URL template.
    #[serde(default)]
    pub downloads: HashMap<String, String>,
}

// ── [provides] / [requires] ───────────────────────────────────────────────────

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawProvides {
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub bus: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawRequires {
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub bus: Vec<String>,
}

// ── [[variables]] ─────────────────────────────────────────────────────────────

/// Boolean flags for a `[[variables]]` entry — direct TOML field mapping.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawVariableFlags {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub needs_restart: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub auto_generate: bool,
}

/// One entry in the `[[variables]]` array — a user-configurable setting.
#[derive(Debug, Deserialize)]
pub(crate) struct RawVariable {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub field_type: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(flatten)]
    pub flags: RawVariableFlags,
}

// ── [contract] ────────────────────────────────────────────────────────────────

/// `[contract]` — reverse proxy integration via Zentinel.
#[derive(Debug, Deserialize)]
pub(crate) struct RawContract {
    #[serde(default)]
    pub upstream_tls: bool,
    #[serde(default)]
    pub routes: Vec<RawRoute>,
}

/// One entry in `[[contract.routes]]`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawRoute {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub strip: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub proto: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

// ── [[binaries]] ──────────────────────────────────────────────────────────────

/// One entry in `[[binaries]]` — a named executable shipped with the package.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBinary {
    /// Human-readable name, e.g. `"kanidmd"`.
    pub name: String,
    /// File name of the binary on disk, e.g. `"kanidmd"`.
    pub binary: String,
    /// Optional short description of what this binary does.
    #[serde(default)]
    pub description: Option<String>,
}

// ── [interfaces] ──────────────────────────────────────────────────────────────

/// `[interfaces]` — which user-facing interfaces the package exposes.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawInterfaces {
    #[serde(default)]
    pub cli: bool,
    #[serde(default)]
    pub api: bool,
    #[serde(default)]
    pub wgui: bool,
    #[serde(default)]
    pub tui: bool,
}

// ── [storage] ─────────────────────────────────────────────────────────────────

/// `[storage]` — filesystem paths reserved by this package.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawStorage {
    /// Per-user private data directory, e.g. `"~/.local/share/freesynergy/{pkg}"`.
    #[serde(default)]
    pub user: Option<String>,
    /// Global/system-wide data directory, e.g. `"/var/lib/freesynergy/{pkg}"`.
    #[serde(default)]
    pub global: Option<String>,
    /// Configuration directory, e.g. `"/etc/freesynergy/{pkg}"`.
    #[serde(default)]
    pub config: Option<String>,
    /// Cache directory, e.g. `"/var/cache/freesynergy/{pkg}"`.
    #[serde(default)]
    pub cache: Option<String>,
}

// ── [api] ─────────────────────────────────────────────────────────────────────

/// One entry in `[[api.rest]]`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawApiRestEndpoint {
    pub base: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub proto: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// URL path to the `OpenAPI` spec, if available.
    #[serde(default)]
    pub spec: Option<String>,
}

/// One entry in `[[api.grpc]]`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawApiGrpcEndpoint {
    pub service: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub description: Option<String>,
}

/// `[api]` — API endpoints exposed by this package.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawApiSection {
    #[serde(default)]
    pub rest: Vec<RawApiRestEndpoint>,
    #[serde(default)]
    pub grpc: Vec<RawApiGrpcEndpoint>,
}

// ── [install_targets] ─────────────────────────────────────────────────────────

/// `[install_targets]` — supported installation methods.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawInstallTargets {
    #[serde(default)]
    pub container: bool,
    #[serde(default)]
    pub rpm: bool,
    #[serde(default)]
    pub deb: bool,
    #[serde(default)]
    pub flatpak: bool,
    /// Pure Rust library crate — loaded as an artifact (no binary).
    #[serde(default)]
    pub artifact: bool,
}
