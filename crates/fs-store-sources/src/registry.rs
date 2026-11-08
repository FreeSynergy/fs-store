//! Pattern: Repository (file-backed register of installed packages, TOML).
//! Knows, per installed package, from which source it came and whether it may update itself.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// One installed package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Installed {
    /// Package name.
    pub name: String,
    /// Version as installed (`1.2.3` or `1.2.3-rc.1`).
    pub version: String,
    /// Git address of the source it came from.
    pub source: String,
    /// Whether the store may update it by itself (default: yes).
    #[serde(default = "yes")]
    pub auto_update: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Document {
    #[serde(default)]
    package: Vec<Installed>,
}

/// The register, kept in `installed.toml`.
#[derive(Debug)]
pub struct Registry {
    path: PathBuf,
    entries: Vec<Installed>,
}

impl Registry {
    /// Loads the register; a missing file is an empty register.
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let doc: Document = match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| StoreError::Format(e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Document::default(),
            Err(e) => return Err(StoreError::Io(e)),
        };
        Ok(Self {
            path: path.to_owned(),
            entries: doc.package,
        })
    }

    /// All installed packages, in the order they were first installed.
    #[must_use]
    pub fn list(&self) -> &[Installed] {
        &self.entries
    }

    /// One package by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Installed> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Notes an install or update; keeps the person's `auto_update` choice for a known package.
    pub fn record(&mut self, name: &str, version: &str, source: &str) {
        match self.entries.iter_mut().find(|e| e.name == name) {
            Some(e) => {
                version.clone_into(&mut e.version);
                source.clone_into(&mut e.source);
            }
            None => self.entries.push(Installed {
                name: name.to_owned(),
                version: version.to_owned(),
                source: source.to_owned(),
                auto_update: true,
            }),
        }
    }

    /// Switches automatic updates of one package; `false` when it is not installed.
    pub fn set_auto_update(&mut self, name: &str, on: bool) -> bool {
        self.entries
            .iter_mut()
            .find(|e| e.name == name)
            .map(|e| e.auto_update = on)
            .is_some()
    }

    /// Forgets a package.
    pub fn forget(&mut self, name: &str) {
        self.entries.retain(|e| e.name != name);
    }

    /// Writes the file atomically (temporary file, then rename).
    ///
    /// # Errors
    /// `StoreError` when writing fails.
    pub fn save(&self) -> Result<(), StoreError> {
        let text = toml::to_string(&Document {
            package: self.entries.clone(),
        })
        .map_err(|e| StoreError::Format(e.to_string()))?;
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(StoreError::Io)?;
        }
        let tmp = self.path.with_extension("toml.new");
        std::fs::write(&tmp, text).map_err(StoreError::Io)?;
        std::fs::rename(&tmp, &self.path).map_err(StoreError::Io)
    }
}
