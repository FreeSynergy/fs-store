//! Pattern: Specification (is a listed file still as it was: content, type, permission bits, readable; and which files in the folder are strangers).

use std::fs;
use std::path::Path;

use fs_store_sources::FileRecord;

use crate::own_files::{Verdict, hash_file, mode_of};

/// What one listed file is now: gone, unreadable, no longer a regular file, other content, other permission bits, or as listed. The
/// content comes first (a changed file is changed whatever its mode), then the type and the mode.
pub(crate) fn judge(path: &Path, record: &FileRecord) -> Verdict {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Verdict::Missing,
        Err(_) => return Verdict::Unreadable,
    };
    if !meta.is_file() {
        return Verdict::Type;
    }
    match hash_file(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Verdict::Missing,
        Err(_) => Verdict::Unreadable,
        Ok(h) if h != record.hash => Verdict::Changed,
        Ok(_) => {
            let mode = mode_of(&meta);
            // a listed mode of 0 means "the system has none" (not Unix); setuid, setgid and sticky are never allowed
            if (record.mode != 0 && mode != record.mode) || mode & 0o7000 != 0 {
                Verdict::Mode
            } else {
                Verdict::Intact
            }
        }
    }
}

/// Everything in `dir` that is not one of the `listed` regular files: other files, links, special files (a folder is no stranger).
pub(crate) fn stray(dir: &Path, listed: &[FileRecord]) -> std::io::Result<Vec<String>> {
    fn walk(
        base: &Path,
        dir: &Path,
        listed: &std::collections::HashSet<&str>,
        out: &mut Vec<String>,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let meta = fs::symlink_metadata(entry.path())?;
            if meta.is_dir() {
                walk(base, &entry.path(), listed, out)?;
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(base)
                .map_err(std::io::Error::other)?
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if !(meta.is_file() && listed.contains(rel.as_str())) {
                out.push(rel);
            }
        }
        Ok(())
    }
    let known: std::collections::HashSet<&str> = listed.iter().map(|f| f.path.as_str()).collect();
    let mut out = Vec::new();
    walk(dir, dir, &known, &mut out)?;
    out.sort();
    Ok(out)
}
