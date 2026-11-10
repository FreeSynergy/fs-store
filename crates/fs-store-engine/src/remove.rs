//! Pattern: none (one function). Takes an installed `tar.gz` package away again.

use std::fs;
use std::io;
use std::path::Path;

use fs_store_core::{PackageName, RunningCheck};

/// Why a package was not removed.
#[derive(Debug, PartialEq, Eq)]
pub enum RemoveError {
    /// Not a plain package name.
    BadName,
    /// Nothing of that name is installed.
    NotInstalled,
    /// The program is running; it is never taken away under its own feet.
    Running,
    /// The disk refused.
    Io(String),
}

/// Removes `current/<name>` and every version below `packages/<name>/`.
///
/// # Errors
/// `RemoveError`; nothing is removed when the program is running.
pub fn remove(root: &Path, name: &str, running: &dyn RunningCheck) -> Result<(), RemoveError> {
    if PackageName::new(name).is_none_or(|n| n.as_str() != name) {
        return Err(RemoveError::BadName);
    }
    let link = root.join("current").join(name);
    let pointers = &fs_store_platform::current().pointers;
    if !pointers.pointer_exists(&link) {
        return Err(RemoveError::NotInstalled);
    }
    if running.is_running(name) {
        return Err(RemoveError::Running);
    }
    let io = |e: io::Error| RemoveError::Io(e.to_string());
    pointers.remove_pointer(&link).map_err(io)?;
    match fs::remove_dir_all(root.join("packages").join(name)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(e)),
    }
}

/// Removes an installed package of this system the way its package manager does (`dnf remove`, `apt-get remove`, the
/// uninstaller of the Windows setup).
///
/// # Errors
/// A sentence when the name is not plain, the command fails, or this system has no manager the store drives.
pub fn remove_native<C: fs_store_platform::Commander>(
    name: &str,
    commander: &mut C,
) -> Result<(), String> {
    fs_store_platform::current()
        .packages
        .remove(name, commander)
}
