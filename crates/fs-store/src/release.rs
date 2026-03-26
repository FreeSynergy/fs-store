// release.rs — Package release types: version, channel, platform, pin.
//
// TODO(fs-types): PackageVersion → replace String with fs_types::SemVer
//                 once fs-store depends on fs-libs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── ReleaseChannel ────────────────────────────────────────────────────────────

/// The release channel a package version is published on.
///
/// Serializes as a plain string so TOML catalogs stay readable:
/// `channel = "stable"`, `channel = "nightly"`, `channel = "my-fork"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseChannel(String);

impl ReleaseChannel {
    /// The default production channel.
    #[must_use]
    pub fn stable() -> Self {
        Self("stable".to_owned())
    }

    /// Pre-release testing channel.
    #[must_use]
    pub fn beta() -> Self {
        Self("beta".to_owned())
    }

    /// Bleeding-edge, built from HEAD.
    #[must_use]
    pub fn nightly() -> Self {
        Self("nightly".to_owned())
    }

    /// Any custom channel name (e.g. a community fork).
    pub fn custom(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The channel name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `true` for the `"stable"` channel.
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.0 == "stable"
    }
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        Self::stable()
    }
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Platform ──────────────────────────────────────────────────────────────────

/// Target platform for a binary release.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    LinuxX86_64,
    LinuxAarch64,
    MacosX86_64,
    MacosAarch64,
    WindowsX86_64,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LinuxX86_64 => "linux-x86_64",
            Self::LinuxAarch64 => "linux-aarch64",
            Self::MacosX86_64 => "macos-x86_64",
            Self::MacosAarch64 => "macos-aarch64",
            Self::WindowsX86_64 => "windows-x86_64",
        };
        f.write_str(s)
    }
}

// ── DistributionMap ───────────────────────────────────────────────────────────

/// Download URL per platform for a binary package release.
///
/// URL strings may contain `{version}` which is substituted at download time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistributionMap(pub HashMap<Platform, String>);

impl DistributionMap {
    /// Returns the download URL for the given platform, if available.
    #[must_use]
    pub fn url_for(&self, platform: &Platform) -> Option<&str> {
        self.0.get(platform).map(String::as_str)
    }

    /// `true` if no platforms are listed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ── PackageRelease ────────────────────────────────────────────────────────────

/// One released version of a package.
///
/// The `version` string follows `SemVer` (e.g. `"1.3.0"`, `"2.0.0-beta.1"`).
///
/// TODO(fs-types): replace `version: String` with `fs_types::SemVer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRelease {
    /// `SemVer` version string, e.g. `"1.3.0"`.
    pub version: String,

    /// Branch this release was built from, e.g. `"main"`, `"release/1.3"`.
    #[serde(default)]
    pub branch: String,

    /// Release channel.
    #[serde(default)]
    pub channel: ReleaseChannel,

    /// Per-platform download URLs (present for binary packages only).
    #[serde(default)]
    pub distribution: DistributionMap,
}

impl PackageRelease {
    /// Convenience constructor for a stable release without binary downloads
    /// (e.g. container images, themes).
    pub fn catalog_only(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            branch: String::new(),
            channel: ReleaseChannel::stable(),
            distribution: DistributionMap::default(),
        }
    }
}

// ── VersionPin ────────────────────────────────────────────────────────────────

/// How a package version is pinned in the local inventory.
///
/// Version number fields use `u16` — no real-world project will exceed 65 535.
///
/// Uses named struct variants so `#[serde(tag)]` works without ambiguity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VersionPin {
    /// Always update to the latest available release (default).
    #[default]
    Latest,

    /// Pin to a major version, e.g. `1.x.x`.
    Major { value: u16 },

    /// Pin to a major.minor version, e.g. `1.3.x`.
    MajorMinor { major: u16, minor: u16 },

    /// Pin to an exact version; never auto-update.
    Exact { version: String },
}

impl VersionPin {
    /// `true` when the pin allows automatic updates.
    #[must_use]
    pub fn allows_auto_update(&self) -> bool {
        !matches!(self, Self::Exact { .. })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_channel_display() {
        assert_eq!(ReleaseChannel::stable().to_string(), "stable");
        assert_eq!(ReleaseChannel::custom("edge").to_string(), "edge");
    }

    #[test]
    fn version_pin_auto_update() {
        assert!(VersionPin::Latest.allows_auto_update());
        assert!(VersionPin::Major { value: 1 }.allows_auto_update());
        assert!(!VersionPin::Exact {
            version: "1.2.3".into()
        }
        .allows_auto_update());
    }

    #[test]
    fn distribution_map_lookup() {
        let mut map = DistributionMap::default();
        map.0.insert(
            Platform::LinuxX86_64,
            "https://example.com/v{version}/linux.tar.gz".into(),
        );
        assert!(map.url_for(&Platform::LinuxX86_64).is_some());
        assert!(map.url_for(&Platform::WindowsX86_64).is_none());
    }
}
