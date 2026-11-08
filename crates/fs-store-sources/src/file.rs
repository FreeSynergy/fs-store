//! Pattern: Repository (file-backed implementation of `SourceRepository`).

use std::path::{Path, PathBuf};

use fs_store_core::{REFERENCE_ID, Source, SourceError, SourceId, SourceRepository};
use serde::{Deserialize, Serialize};

/// Why loading or saving failed.
#[derive(Debug)]
pub enum StoreError {
    /// The file could not be read or written.
    Io(std::io::Error),
    /// The file is not valid TOML of the expected shape.
    Format(String),
}

/// The source list kept in a small TOML file.
#[derive(Debug)]
pub struct FileSources {
    path: PathBuf,
    sources: Vec<Source>,
}

/// What the file holds: the ids switched off, and the user's own sources.
/// The reference source is never stored as an entry, only its switch.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Document {
    #[serde(default)]
    disabled: Vec<String>,
    #[serde(default)]
    source: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    id: String,
    url: String,
}

impl FileSources {
    /// Loads the list; a missing file gives just the reference source.
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let doc: Document = match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| StoreError::Format(e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Document::default(),
            Err(e) => return Err(StoreError::Io(e)),
        };
        let off = |id: &str| doc.disabled.iter().any(|d| d == id);
        let mut reference = Source::reference();
        reference.enabled = !off(REFERENCE_ID);
        let mut sources = vec![reference];
        for e in doc.source.iter().filter(|e| e.id != REFERENCE_ID) {
            if sources.iter().all(|s| s.id.0 != e.id) {
                sources.push(Source {
                    id: SourceId(e.id.clone()),
                    url: e.url.clone(),
                    enabled: !off(&e.id),
                    reference: false,
                });
            }
        }
        Ok(Self {
            path: path.to_owned(),
            sources,
        })
    }

    /// Writes the list to the file.
    ///
    /// # Errors
    /// `StoreError::Io` when writing fails.
    pub fn save(&self) -> Result<(), StoreError> {
        let doc = Document {
            disabled: self
                .sources
                .iter()
                .filter(|s| !s.enabled)
                .map(|s| s.id.0.clone())
                .collect(),
            source: self
                .sources
                .iter()
                .filter(|s| !s.reference)
                .map(|s| Entry {
                    id: s.id.0.clone(),
                    url: s.url.clone(),
                })
                .collect(),
        };
        let text = toml::to_string(&doc).map_err(|e| StoreError::Format(e.to_string()))?;
        std::fs::write(&self.path, text).map_err(StoreError::Io)
    }

    fn position(&self, id: &SourceId) -> Result<usize, SourceError> {
        self.sources
            .iter()
            .position(|s| &s.id == id)
            .ok_or(SourceError::NotFound)
    }
}

impl SourceRepository for FileSources {
    fn list(&self) -> Vec<Source> {
        self.sources.clone()
    }
    fn add(&mut self, url: &str, id: SourceId) -> Result<(), SourceError> {
        if self.sources.iter().any(|s| s.id == id) {
            return Err(SourceError::Duplicate);
        }
        self.sources.push(Source {
            id,
            url: url.to_owned(),
            enabled: true,
            reference: false,
        });
        Ok(())
    }
    fn set_enabled(&mut self, id: &SourceId, enabled: bool) -> Result<(), SourceError> {
        let i = self.position(id)?;
        self.sources[i].enabled = enabled;
        Ok(())
    }
    fn remove(&mut self, id: &SourceId) -> Result<(), SourceError> {
        let i = self.position(id)?;
        if self.sources[i].reference {
            return Err(SourceError::ReferenceSource);
        }
        self.sources.remove(i);
        Ok(())
    }
}
