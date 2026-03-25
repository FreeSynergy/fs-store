// fs-store — FreeSynergy Store library
//
// This crate is the core of the FreeSynergy Store. It provides:
//
//   Package (trait)       — base interface for every package type
//   Package sub-types     — AppPackage, ContainerPackage, ThemePackage, ...
//   PackageCategory       — object (not enum) describing a package namespace
//   PackageRelease        — a versioned release with channel + distribution
//   VersionPin            — how a local install is pinned for updates
//   InstallRecord         — one locally installed package version
//   PackageState          — available (Store) + installed (local) combined view
//   Inventory             — central hub: all packages + install state + settings
//   StoreReader           — fetches TOML catalogs, FTL help, i18n, icons
//   StoreSource           — where to fetch from (Local / Http, Open/Closed)
//   BootableInstaller     — USB installer (init/, not a Package)
//   IconSource            — trait for icon retrieval from multiple sources
//   Settings              — unified settings interface
//   StoreSettings         — concrete settings for the Store
//   i18n types            — I18nSnippets (TOML), FtlContent (.ftl), FtlKind

pub mod category;
pub mod i18n;
pub mod icon;
pub mod init;
pub mod install_record;
pub mod inventory;
pub mod package;
pub mod reader;
pub mod release;
pub mod settings;
pub mod source;

// ── Top-level re-exports ──────────────────────────────────────────────────────

pub use category::PackageCategory;
pub use i18n::{FtlContent, FtlKind, I18nSnippets};
pub use icon::IconSource;
pub use init::BootableInstaller;
pub use install_record::InstallRecord;
pub use inventory::{Inventory, NamespaceMap, PackageState};
pub use package::{Package, PackageData, PackageHelp};
pub use reader::StoreReader;
pub use release::{DistributionMap, PackageRelease, Platform, ReleaseChannel, VersionPin};
pub use settings::{Settings, StoreSettings};
pub use source::StoreSource;
