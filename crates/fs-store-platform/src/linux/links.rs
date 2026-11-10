//! Pattern: Strategy (Linux: a symlink is the pointer and the command).

use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::{Commands, Pointers};

/// Pointers and commands as symbolic links.
pub(super) struct LinuxLinks;

impl Pointers for LinuxLinks {
    fn point(&self, link: &Path, target: &Path) -> io::Result<()> {
        let dir = link.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;
        let name = link.file_name().unwrap_or_default().to_string_lossy();
        let tmp = dir.join(format!(".{name}.new.{}", std::process::id()));
        let _ = fs::remove_file(&tmp);
        symlink(target, &tmp)?;
        fs::rename(&tmp, link)
    }

    fn pointer_target(&self, link: &Path) -> Option<PathBuf> {
        fs::read_link(link).ok()
    }

    fn pointer_exists(&self, link: &Path) -> bool {
        fs::symlink_metadata(link).is_ok()
    }

    fn remove_pointer(&self, link: &Path) -> io::Result<()> {
        fs::remove_file(link)
    }
}

impl Commands for LinuxLinks {
    fn make_command(&self, bin_dir: &Path, name: &str, target: &Path) -> io::Result<()> {
        fs::create_dir_all(bin_dir)?;
        symlink(target, bin_dir.join(name))
    }

    fn command_path(&self, bin_dir: &Path, name: &str) -> PathBuf {
        bin_dir.join(name)
    }

    fn command_target(&self, bin_dir: &Path, name: &str) -> Option<PathBuf> {
        fs::read_link(bin_dir.join(name)).ok()
    }

    fn command_exists(&self, bin_dir: &Path, name: &str) -> bool {
        fs::symlink_metadata(bin_dir.join(name)).is_ok()
    }

    fn remove_command(&self, bin_dir: &Path, name: &str) -> io::Result<()> {
        fs::remove_file(bin_dir.join(name))
    }

    fn commands(&self, bin_dir: &Path) -> Vec<(String, PathBuf)> {
        let Ok(dir) = fs::read_dir(bin_dir) else {
            return Vec::new();
        };
        dir.flatten()
            .filter_map(|e| {
                let target = fs::read_link(e.path()).ok()?;
                Some((e.file_name().to_string_lossy().into_owned(), target))
            })
            .collect()
    }
}
