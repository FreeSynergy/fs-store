//! Pattern: Repository (file-backed): the person's settings for updates, and what the last check found.
//! `settings.toml` is the person's; `updates.toml` is written by the background check and read by the window.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// How often the store looks for updates by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CheckEvery {
    /// Never by itself (only when asked).
    Off,
    /// When the window opens.
    Start,
    /// Once a day (default).
    #[default]
    Daily,
    /// Every hour.
    Hourly,
}

/// What to do with what the check finds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum InstallMode {
    /// Tell the person and wait (default).
    #[default]
    Ask,
    /// Install updates by itself (only programs with automatic updates on).
    Auto,
}

/// The settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// How often to look.
    #[serde(default)]
    pub check: CheckEvery,
    /// What to do with what is found.
    #[serde(default)]
    pub install: InstallMode,
    /// Tell the person (desktop notification and badge) when the check found something (default: yes).
    #[serde(default = "yes")]
    pub notify: bool,
}

fn yes() -> bool {
    true
}

impl Settings {
    /// Loads `path`; a missing file gives the defaults (check daily, ask before installing, notify).
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| StoreError::Format(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                notify: true,
                ..Self::default()
            }),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    /// Writes the file atomically.
    ///
    /// # Errors
    /// `StoreError` when writing fails.
    pub fn save(&self, path: &Path) -> Result<(), StoreError> {
        write_atomic(
            path,
            &toml::to_string(self).map_err(|e| StoreError::Format(e.to_string()))?,
        )
    }
}

/// An installed program with a newer version on offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    /// Package name.
    pub name: String,
    /// Installed version.
    pub from: String,
    /// Offered version.
    pub to: String,
}

/// A program the catalog offers that was not there at the last look.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Novelty {
    /// Package name.
    pub name: String,
    /// Offered version.
    pub version: String,
}

/// What the last check found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Findings {
    /// Unix seconds of the last check; 0 = never.
    #[serde(default)]
    pub checked: u64,
    /// Newer versions of installed programs.
    #[serde(default)]
    pub update: Vec<Update>,
    /// Programs offered that the person has not seen yet.
    #[serde(default)]
    pub new: Vec<Novelty>,
    /// Names of all programs the catalog offered at the last check (to find new ones next time).
    #[serde(default)]
    pub offered: Vec<String>,
}

impl Findings {
    /// Loads `path`; a missing file is "never checked".
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| StoreError::Format(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    /// Writes the file atomically.
    ///
    /// # Errors
    /// `StoreError` when writing fails.
    pub fn save(&self, path: &Path) -> Result<(), StoreError> {
        write_atomic(
            path,
            &toml::to_string(self).map_err(|e| StoreError::Format(e.to_string()))?,
        )
    }

    /// The person has looked at the new programs: they are not new any more.
    pub fn mark_seen(&mut self) {
        self.new.clear();
    }
}

fn write_atomic(path: &Path, text: &str) -> Result<(), StoreError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(StoreError::Io)?;
    }
    let tmp = path.with_extension("toml.new");
    std::fs::write(&tmp, text).map_err(StoreError::Io)?;
    std::fs::rename(&tmp, path).map_err(StoreError::Io)
}
