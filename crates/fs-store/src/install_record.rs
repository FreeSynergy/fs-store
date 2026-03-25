// install_record.rs — Local installation state for a package.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::release::VersionPin;

// ── InstallRecord ─────────────────────────────────────────────────────────────

/// Records one installed instance of a package.
///
/// Multiple records can exist for the same package (different versions or
/// branches installed side by side). Only one is `is_active` at a time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRecord {
    /// Package id this record belongs to.
    pub package_id: String,

    /// Installed version string, e.g. `"1.3.0"`.
    pub version: String,

    /// Branch the installed release was built from, e.g. `"main"`.
    #[serde(default)]
    pub branch: String,

    /// How the installed version is pinned for future updates.
    #[serde(default)]
    pub pin: VersionPin,

    /// When this version was installed.
    pub installed_at: DateTime<Utc>,

    /// Where the package data lives on disk.
    pub install_path: PathBuf,

    /// `true` when this is the active version (only one per package).
    #[serde(default)]
    pub is_active: bool,
}

impl InstallRecord {
    /// Create a new install record for the current time, marked active.
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        install_path: PathBuf,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            version: version.into(),
            branch: String::new(),
            pin: VersionPin::default(),
            installed_at: Utc::now(),
            install_path,
            is_active: true,
        }
    }
}
