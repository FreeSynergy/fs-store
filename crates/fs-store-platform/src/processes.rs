//! Pattern: Strategy (asking the system about processes: `/proc` on Linux, the process table on Windows).

use std::io;
use std::path::{Path, PathBuf};

/// One running process of this person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    /// Process id.
    pub pid: u32,
    /// The command line: the program, then its arguments.
    pub argv: Vec<String>,
    /// The file it runs from, when known.
    pub exe: Option<PathBuf>,
}

/// Process information and control.
pub trait Processes: Send + Sync {
    /// The processes of the person running this code (others are never listed).
    fn mine(&self) -> Vec<ProcessInfo>;
    /// True while process `pid` runs (a zombie does not count).
    fn is_alive(&self, pid: u32) -> bool;
    /// Asks the process to end in order. Never kills.
    ///
    /// # Errors
    /// A sentence when the system could not deliver the request.
    fn request_stop(&self, pid: u32) -> Result<(), String>;
    /// Starts `program` in the background, detached from this process, without input or output.
    ///
    /// # Errors
    /// `io::Error` when it cannot be started.
    fn spawn_detached(&self, program: &Path, args: &[String]) -> io::Result<()>;
    /// Opens a web address with the system's opener, as a process of its own.
    ///
    /// # Errors
    /// A sentence when there is no opener or it could not be started.
    fn open_url(&self, url: &str) -> Result<(), String>;
}
