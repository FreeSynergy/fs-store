//! Pattern: none - two plain helper functions that the Strategies `Rpm` and `Deb` share (write the verified bytes into a
//! private directory; the command that hands them to the package manager stays in each Strategy).
//! The person confirms root themselves: `pkexec` asks for it, we never see a password.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// Writes `files` below `dir` (created with mode 0700) as `<name>-<version>-<pid>-<n>.<ext>`: names nobody can guess,
/// created exclusively with mode 0600 and never through a link, so there is nothing to swap after the check.
/// Returns every path that was tried (also after a failure, so the caller can delete them) and the outcome.
pub(crate) fn write_private(
    dir: &Path,
    stem: &str,
    ext: &str,
    files: &[Vec<u8>],
) -> (Vec<PathBuf>, Result<(), String>) {
    let mut paths = Vec::new();
    // Absolute, so the package manager reads a path (a name with a `/` is a file to apt) and no relative root confuses it.
    let dir = &std::path::absolute(dir).unwrap_or_else(|_| dir.to_owned());
    let prepared = fs::create_dir_all(dir).and_then(|()| {
        fs::set_permissions(dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
    });
    if let Err(e) = prepared {
        return (paths, Err(e.to_string()));
    }
    for (n, bytes) in files.iter().enumerate() {
        let file = dir.join(format!("{stem}-{}-{n}.{ext}", std::process::id()));
        let _ = fs::remove_file(&file);
        let one = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(0x2_0000) // O_NOFOLLOW
            .open(&file)
            .and_then(|mut f| f.write_all(bytes));
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
