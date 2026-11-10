//! Pattern: Strategy (Linux: the XDG-like places below `$HOME`).

use std::path::{Path, PathBuf};

use crate::{Paths, Vars, process_vars};

/// Paths on Linux: `~/.local/share/fs-store`, `~/.local/bin`, `~/.local/share/applications`.
pub struct LinuxPaths {
    vars: Vars,
}

impl LinuxPaths {
    /// Reads the environment of this process.
    #[must_use]
    pub fn new() -> Self {
        Self::with_vars(process_vars())
    }

    /// Reads `vars` instead of the environment (tests).
    #[must_use]
    pub fn with_vars(vars: Vars) -> Self {
        Self { vars }
    }

    fn home(&self) -> PathBuf {
        PathBuf::from((self.vars)("HOME").unwrap_or_default())
    }

    fn or_home(&self, var: &str, below: &str) -> PathBuf {
        (self.vars)(var).map_or_else(|| self.home().join(below), PathBuf::from)
    }
}

impl Default for LinuxPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl Paths for LinuxPaths {
    fn store_root(&self) -> PathBuf {
        self.or_home("FS_STORE_ROOT", ".local/share/fs-store")
    }

    fn bin_dir(&self) -> PathBuf {
        self.or_home("FS_STORE_BIN", ".local/bin")
    }

    fn menu_dir(&self) -> PathBuf {
        self.or_home("FS_STORE_APPS", ".local/share/applications")
    }

    fn config_dir(&self) -> PathBuf {
        self.home().join(".config")
    }

    fn native_program_dirs(&self, _package: &str) -> Vec<PathBuf> {
        vec![PathBuf::from("/usr/bin")]
    }

    fn program_file_name(&self, command: &str) -> String {
        command.to_owned()
    }

    fn command_files(&self, command: &str) -> Vec<String> {
        vec![command.to_owned()]
    }

    fn command_name(&self, file: &str) -> String {
        file.to_owned()
    }

    fn is_below(&self, path: &Path, dir: &Path) -> bool {
        path.starts_with(dir)
    }
}
