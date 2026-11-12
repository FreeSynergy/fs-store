//! Pattern: Guard (one install of a package at a time, across processes: the window, a command, the timer). A lock file below the store root,
//! one per package name; the lock is the operating system's (`flock`), so it goes away with the process, also after a crash.

use std::fs::{self, File, TryLockError};
use std::io;
use std::path::Path;

/// Held while one process installs or updates a package; let go when dropped.
#[derive(Debug)]
pub struct PackageLock {
    _file: File,
}

impl PackageLock {
    /// Takes the lock of `name` below `root`. `Ok(None)` when another process holds it (the caller says so and does not install).
    ///
    /// # Errors
    /// `io::Error` when the lock file cannot be made.
    pub fn acquire(root: &Path, name: &str) -> io::Result<Option<Self>> {
        let dir = root.join("locks");
        fs::create_dir_all(&dir)?;
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join(format!("{name}.lock")))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(e)) => Err(e),
        }
    }
}
