//! Pattern: none - two plain helper functions that the Strategies `Rpm`, `Deb` and `Exe` share (write the verified bytes
//! into a private directory; the command that hands them to the package manager stays in each Strategy).
//! The person confirms the administrator's rights themselves (`pkexec`, the UAC prompt): we never see a password.

use std::path::{Path, PathBuf};

/// Writes `files` below `dir` (a folder only this person can use) as `<name>-<version>-<pid>-<n>.<ext>`: names nobody can
/// guess, created exclusively for this person alone and never through a link, so there is nothing to swap after the check.
/// Returns every path that was tried (also after a failure, so the caller can delete them) and the outcome.
pub(crate) fn write_private(
    dir: &Path,
    stem: &str,
    ext: &str,
    files: &[Vec<u8>],
) -> (Vec<PathBuf>, Result<(), String>) {
    let permissions = &fs_store_platform::current().permissions;
    let mut paths = Vec::new();
    // Absolute, so the package manager reads a path (a name with a `/` is a file to apt) and no relative root confuses it.
    let dir = &std::path::absolute(dir).unwrap_or_else(|_| dir.to_owned());
    if let Err(e) = permissions.create_private_dir(dir) {
        return (paths, Err(e.to_string()));
    }
    for (n, bytes) in files.iter().enumerate() {
        let file = dir.join(format!("{stem}-{}-{n}.{ext}", std::process::id()));
        let _ = std::fs::remove_file(&file);
        let one = permissions.create_private_file(&file, bytes);
        paths.push(file);
        if let Err(e) = one {
            return (paths, Err(e.to_string()));
        }
    }
    (paths, Ok(()))
}

/// The paths as the text a command line needs.
pub(crate) fn names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|f| f.to_string_lossy().into_owned())
        .collect()
}

/// Runs `program args` with the administrator's rights of this system (`pkexec`, the UAC prompt).
pub(crate) fn run_elevated(
    commander: &mut dyn fs_store_platform::Commander,
    program: &str,
    args: &[&str],
) -> Result<(), String> {
    let (wrapped, all) = fs_store_platform::current().elevation.wrap(program, args);
    let all: Vec<&str> = all.iter().map(String::as_str).collect();
    commander.run(&wrapped, &all)
}
