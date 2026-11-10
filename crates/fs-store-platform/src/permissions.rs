//! Pattern: Strategy (what the system calls a right: mode bits on Linux, extensions and ACLs on Windows).

use std::io;
use std::path::Path;

/// Rights on files and who is running.
pub trait Permissions: Send + Sync {
    /// True when `path` is a file the system can start.
    fn is_executable(&self, path: &Path) -> bool;
    /// Creates the folder (and the ones above it) so that only this person can use it.
    ///
    /// # Errors
    /// `io::Error` when the folder cannot be made or restricted.
    fn create_private_dir(&self, dir: &Path) -> io::Result<()>;
    /// Creates a new file for this person alone, never through a link and never over an existing file, and writes `bytes`.
    /// The answer is a guard: while it lives, nobody can change or replace the file (Windows: it stays open for reading and
    /// refuses other writers and deleters, so what was checked is what an elevated setup then starts). On Unix the folder
    /// of mode 0700 is what keeps others out and the guard holds nothing.
    ///
    /// # Errors
    /// `io::Error` (also when the file exists already).
    fn create_private_file(&self, path: &Path, bytes: &[u8]) -> io::Result<Box<dyn Send>>;
    /// True when this process runs with the rights of the administrator (root).
    fn is_elevated(&self) -> bool;
}
