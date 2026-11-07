//! Pattern: Repository (file-backed implementation of `SourceRepository`).

use std::path::{Path, PathBuf};

use fs_launcher_core::{Source, SourceError, SourceId, SourceRepository};

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

impl FileSources {
    /// Loads the list; a missing file gives just the reference source.
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let _ = path;
        todo!()
    }

    /// Writes the list to the file.
    ///
    /// # Errors
    /// `StoreError::Io` when writing fails.
    pub fn save(&self) -> Result<(), StoreError> {
        let _ = &self.path;
        todo!()
    }
}

impl SourceRepository for FileSources {
    fn list(&self) -> Vec<Source> {
        self.sources.clone()
    }
    fn add(&mut self, url: &str, id: SourceId) -> Result<(), SourceError> {
        let _ = (url, id);
        todo!()
    }
    fn set_enabled(&mut self, id: &SourceId, enabled: bool) -> Result<(), SourceError> {
        let _ = (id, enabled);
        todo!()
    }
    fn remove(&mut self, id: &SourceId) -> Result<(), SourceError> {
        let _ = id;
        todo!()
    }
}
