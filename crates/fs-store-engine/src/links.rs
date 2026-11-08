//! Pattern: none (two functions). Puts the programs of a package on the person's PATH, and takes them away again.
//! A link points through `current/<name>/`, so an update changes what it runs without touching the link.

use std::fs;
use std::io;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;

/// Links every executable file at the top of `current/<name>/` into `bin_dir`. Returns the names linked.
/// A name that exists and is not one of our links is left alone and reported in `skipped`.
///
/// # Errors
/// `io::Error` when the directory cannot be read or a link cannot be made.
pub fn link_programs(
    root: &Path,
    name: &str,
    bin_dir: &Path,
) -> io::Result<(Vec<String>, Vec<String>)> {
    let pkg = root.join("current").join(name);
    fs::create_dir_all(bin_dir)?;
    let (mut linked, mut skipped) = (Vec::new(), Vec::new());
    for entry in fs::read_dir(&pkg)? {
        let entry = entry?;
        let file = entry.file_name().to_string_lossy().into_owned();
        let meta = fs::metadata(entry.path())?;
        if !meta.is_file() || meta.permissions().mode() & 0o111 == 0 || file.starts_with('.') {
            continue;
        }
        let link = bin_dir.join(&file);
        let target = pkg.join(&file);
        match fs::read_link(&link) {
            Ok(old) if old.starts_with(root.join("current")) => fs::remove_file(&link)?,
            Ok(_) => {
                skipped.push(file);
                continue;
            }
            Err(_) if fs::symlink_metadata(&link).is_ok() => {
                skipped.push(file);
                continue;
            }
            Err(_) => {}
        }
        symlink(&target, &link)?;
        linked.push(file);
    }
    Ok((linked, skipped))
}

/// Removes the links in `bin_dir` that point into `current/<name>/`. Returns how many were removed.
///
/// # Errors
/// `io::Error` when the directory cannot be read or a link cannot be removed.
pub fn unlink_programs(root: &Path, name: &str, bin_dir: &Path) -> io::Result<usize> {
    let pkg = root.join("current").join(name);
    let mut n = 0;
    let Ok(dir) = fs::read_dir(bin_dir) else {
        return Ok(0);
    };
    for entry in dir {
        let path = entry?.path();
        if fs::read_link(&path).is_ok_and(|t| t.starts_with(&pkg)) {
            fs::remove_file(&path)?;
            n += 1;
        }
    }
    Ok(n)
}
