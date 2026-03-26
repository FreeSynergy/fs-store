// settings.rs — Settings trait and StoreSettings.
//
// Settings is a unified, extendable object. Each program implements the
// Settings trait and adds its own program-specific fields on top of the
// base categories. A future SettingsManager will aggregate all of them.

use serde::{Deserialize, Serialize};

use crate::source::StoreSource;

// ── Base settings categories ──────────────────────────────────────────────────

/// Where data and caches are stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    /// Root directory for installed package data.
    /// Default: `~/.local/share/freesynergy/store/packages`
    pub data_dir: std::path::PathBuf,

    /// Directory for cached icons and help files.
    /// Default: `~/.cache/freesynergy/store`
    pub cache_dir: std::path::PathBuf,
}

impl Default for StorageSettings {
    fn default() -> Self {
        let base = dirs_base();
        Self {
            data_dir: base.join("store/packages"),
            cache_dir: cache_base().join("store"),
        }
    }
}

/// Where package catalogs are fetched from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesSettings {
    /// Ordered list of store sources. First match wins.
    /// The official `FreeSynergy` Store is always the first entry.
    pub sources: Vec<String>,
}

impl Default for SourcesSettings {
    fn default() -> Self {
        Self {
            sources: vec!["https://raw.githubusercontent.com/FreeSynergy/Store/main".to_owned()],
        }
    }
}

/// Network behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    /// HTTP request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u16,

    /// Maximum number of retries on transient errors.
    #[serde(default = "default_retries")]
    pub retries: u8,

    /// Operate without network access (use local cache only).
    #[serde(default)]
    pub offline_mode: bool,
}

fn default_timeout() -> u16 {
    30
}
fn default_retries() -> u8 {
    3
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            retries: default_retries(),
            offline_mode: false,
        }
    }
}

/// Auto-update and version-pin behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Automatically update packages that have no version pin.
    #[serde(default = "default_true")]
    pub auto_update: bool,

    /// Preferred release channel for packages without an explicit channel.
    #[serde(default)]
    pub default_channel: String,
}

fn default_true() -> bool {
    true
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_update: true,
            default_channel: "stable".to_owned(),
        }
    }
}

/// General UX behaviour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    /// Always ask for confirmation before install/remove.
    #[serde(default)]
    pub always_confirm: bool,

    /// UI display locale, e.g. `"de"`, `"en"`.
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_locale() -> String {
    "en".to_owned()
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            always_confirm: false,
            locale: default_locale(),
        }
    }
}

/// Trust and signature verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Verify Ed25519 signatures on catalog files.
    #[serde(default = "default_true")]
    pub verify_signatures: bool,

    /// Paths to additional trusted public keys.
    #[serde(default)]
    pub extra_trust_anchors: Vec<std::path::PathBuf>,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            verify_signatures: true,
            extra_trust_anchors: Vec::new(),
        }
    }
}

// ── Settings trait ────────────────────────────────────────────────────────────

/// The unified settings interface.
///
/// Every program that has configurable settings implements this trait.
/// Programs may override individual category methods to return their own
/// extended settings structs. The `SettingsManager` (future) aggregates
/// all implementations.
pub trait Settings {
    fn storage(&self) -> &StorageSettings;
    fn sources(&self) -> &SourcesSettings;
    fn network(&self) -> &NetworkSettings;
    fn updates(&self) -> &UpdateSettings;
    fn behavior(&self) -> &BehaviorSettings;
    fn security(&self) -> &SecuritySettings;
}

// ── StoreSettings ─────────────────────────────────────────────────────────────

/// Settings for the `FreeSynergy` Store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreSettings {
    pub storage: StorageSettings,
    pub sources: SourcesSettings,
    pub network: NetworkSettings,
    pub updates: UpdateSettings,
    pub behavior: BehaviorSettings,
    pub security: SecuritySettings,
}

impl StoreSettings {
    /// Build a `StoreSource` list from the configured source URLs.
    #[must_use]
    pub fn store_sources(&self) -> Vec<StoreSource> {
        self.sources
            .sources
            .iter()
            .map(|url| StoreSource::Http(url.clone()))
            .collect()
    }
}

impl Settings for StoreSettings {
    fn storage(&self) -> &StorageSettings {
        &self.storage
    }
    fn sources(&self) -> &SourcesSettings {
        &self.sources
    }
    fn network(&self) -> &NetworkSettings {
        &self.network
    }
    fn updates(&self) -> &UpdateSettings {
        &self.updates
    }
    fn behavior(&self) -> &BehaviorSettings {
        &self.behavior
    }
    fn security(&self) -> &SecuritySettings {
        &self.security
    }
}

// ── Platform helpers ──────────────────────────────────────────────────────────

fn dirs_base() -> std::path::PathBuf {
    std::env::var("XDG_DATA_HOME").map_or_else(
        |_| dirs_home().join(".local/share/freesynergy"),
        std::path::PathBuf::from,
    )
}

fn cache_base() -> std::path::PathBuf {
    std::env::var("XDG_CACHE_HOME").map_or_else(
        |_| dirs_home().join(".cache/freesynergy"),
        std::path::PathBuf::from,
    )
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME").map_or_else(
        |_| std::path::PathBuf::from("/tmp"),
        std::path::PathBuf::from,
    )
}
