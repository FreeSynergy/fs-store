//! Pattern: Repository (file-backed list of the files the store itself put on the disk, one TOML file per package under
//! `<root>/manifests/`). It is the truth for "what WE installed": path, package, version, source, signer and a checksum
//! per file. The same on every system, because it does not ask rpm, dpkg or a registry.
//!
//! Schema (version 1) is chosen to become an AS2 `Add` activity later (draft `ENTWURF-files-kern.md` section 1.3):
//! `files` is the `object` list (`path` + `hash`), `source` the `origin`, `name`/`version`/`signer` the `instrument`,
//! `recorded_at` the time. Hashes are written `<algorithm>:<hex>`, like the `blake3:<hex>` of the files program, so a
//! second algorithm can sit next to `sha256` without a change of the schema.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// How the list came to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Basis {
    /// Written by the store while it installed the package: the checksums are those of the verified release.
    Installed,
    /// Made later from the tree that was found (an installation older than the list). It cannot tell a change made
    /// before the list was made; it is never shown as the original.
    Retroactive,
}

/// One file of a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    /// Path inside the package directory, with `/` on every system.
    pub path: String,
    /// `sha256:<hex>`.
    pub hash: String,
    /// Size in bytes.
    pub size: u64,
    /// Unix permission bits (0 where the system has none).
    pub mode: u32,
}

/// The files of one installed package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Version of this schema.
    pub schema: u32,
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Git address of the source it came from; empty when unknown (retroactive lists).
    pub source: String,
    /// Fingerprint that signed the release file, upper-case hex; empty when unknown.
    pub signer: String,
    /// Original or retroactive.
    pub basis: Basis,
    /// Unix seconds when the list was made.
    pub recorded_at: u64,
    /// The files, sorted by path.
    pub files: Vec<FileRecord>,
}

/// Where the lists are kept.
#[derive(Debug, Clone)]
pub struct ManifestStore {
    dir: PathBuf,
}

fn plain(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && !name.starts_with('.')
}

impl ManifestStore {
    /// The lists of the store at `root` (`<root>/manifests/`).
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            dir: root.join("manifests"),
        }
    }

    fn file(&self, name: &str) -> Result<PathBuf, StoreError> {
        if plain(name) {
            Ok(self.dir.join(format!("{name}.toml")))
        } else {
            Err(StoreError::Format(format!(
                "`{name}` is not a plain package name"
            )))
        }
    }

    /// The list of `name`; `None` when there is none.
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(&self, name: &str) -> Result<Option<Manifest>, StoreError> {
        match std::fs::read_to_string(self.file(name)?) {
            Ok(text) => toml::from_str(&text)
                .map(Some)
                .map_err(|e| StoreError::Format(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    /// Writes the list atomically (temporary file, then rename); an older list of the package is replaced.
    ///
    /// # Errors
    /// `StoreError` when writing fails.
    pub fn save(&self, manifest: &Manifest) -> Result<(), StoreError> {
        let path = self.file(&manifest.name)?;
        let text = toml::to_string(manifest).map_err(|e| StoreError::Format(e.to_string()))?;
        std::fs::create_dir_all(&self.dir).map_err(StoreError::Io)?;
        let tmp = path.with_extension("toml.new");
        std::fs::write(&tmp, text).map_err(StoreError::Io)?;
        std::fs::rename(&tmp, &path).map_err(StoreError::Io)
    }

    /// Forgets the list of `name` (nothing to do when there is none).
    ///
    /// # Errors
    /// `StoreError` when the file cannot be removed.
    pub fn forget(&self, name: &str) -> Result<(), StoreError> {
        match std::fs::remove_file(self.file(name)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::Io(e)),
        }
    }

    /// The names that have a list, sorted.
    ///
    /// # Errors
    /// `StoreError` when the folder cannot be read (a missing folder means no lists).
    pub fn names(&self) -> Result<Vec<String>, StoreError> {
        let dir = match std::fs::read_dir(&self.dir) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(StoreError::Io(e)),
        };
        let mut names: Vec<String> = dir
            .flatten()
            .filter_map(|e| {
                e.file_name()
                    .to_str()?
                    .strip_suffix(".toml")
                    .map(str::to_owned)
            })
            .collect();
        names.sort();
        Ok(names)
    }
}
