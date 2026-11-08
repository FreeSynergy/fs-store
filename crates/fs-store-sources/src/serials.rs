//! Pattern: Repository (file-backed): the highest catalog serial seen per source, so an older catalog is a rollback.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::StoreError;

/// Why a catalog is not fresh.
#[derive(Debug, PartialEq, Eq)]
pub enum Stale {
    /// Past its expiry date.
    Expired,
    /// Older than one already seen from this source.
    RolledBack {
        /// The highest serial seen before.
        seen: u64,
    },
}

/// The memory of serials, kept in `serials.toml`.
#[derive(Debug)]
pub struct Serials {
    path: PathBuf,
    seen: BTreeMap<String, u64>,
}

impl Serials {
    /// Loads the file; a missing file is an empty memory.
    ///
    /// # Errors
    /// `StoreError` when the file exists but cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let seen = match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| StoreError::Format(e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(e) => return Err(StoreError::Io(e)),
        };
        Ok(Self {
            path: path.to_owned(),
            seen,
        })
    }

    /// Checks a catalog from `source` against `now` (Unix seconds) and the serials seen before; remembers a newer
    /// serial. A catalog without serial or expiry is accepted (the caller says so); with them it must be fresh.
    ///
    /// # Errors
    /// `Stale` when expired or rolled back; nothing is remembered then.
    pub fn check(
        &mut self,
        source: &str,
        serial: Option<u64>,
        expires: Option<u64>,
        now: u64,
    ) -> Result<(), Stale> {
        if expires.is_some_and(|e| now > e) {
            return Err(Stale::Expired);
        }
        if let Some(n) = serial {
            match self.seen.get(source) {
                Some(&seen) if n < seen => return Err(Stale::RolledBack { seen }),
                _ => {
                    self.seen.insert(source.to_owned(), n);
                }
            }
        }
        Ok(())
    }

    /// Writes the file atomically.
    ///
    /// # Errors
    /// `StoreError` when writing fails.
    pub fn save(&self) -> Result<(), StoreError> {
        let text = toml::to_string(&self.seen).map_err(|e| StoreError::Format(e.to_string()))?;
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(StoreError::Io)?;
        }
        let tmp = self.path.with_extension("toml.new");
        std::fs::write(&tmp, text).map_err(StoreError::Io)?;
        std::fs::rename(&tmp, &self.path).map_err(StoreError::Io)
    }
}

/// Freshness of one catalog read just now: expiry against the clock, serial against the memory in `file`.
/// A catalog without serial and expiry passes (it only cannot be protected).
///
/// # Errors
/// A sentence for the person when the catalog is expired or older than one already seen, or the memory is unreadable.
pub fn ensure_fresh(
    file: &Path,
    source: &str,
    serial: Option<u64>,
    expires: Option<u64>,
) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut memory = Serials::load(file).map_err(|e| format!("{e:?}"))?;
    memory.check(source, serial, expires, now).map_err(|stale| match stale {
        Stale::Expired => format!("{source}: the catalog is expired (a mirror may be serving an old copy)"),
        Stale::RolledBack { seen } => format!("{source}: the catalog is older than one seen before (serial {seen}): rollback refused"),
    })?;
    memory.save().map_err(|e| format!("{e:?}"))
}
