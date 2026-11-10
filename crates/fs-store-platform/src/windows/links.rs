//! Pattern: Strategy (Windows: a junction is the pointer, a small `.cmd` file is the command). No symlinks: they need a
//! right most people do not have. A junction needs none and works for folders, which is all a pointer is.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{Commands, Pointers};

/// Pointers as junctions and commands as launcher files.
pub(super) struct WindowsLinks;

const MARK: &str = "rem fs-store-link ";

fn without_verbatim(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    text.strip_prefix(r"\\?\")
        .map_or_else(|| path.to_owned(), PathBuf::from)
}

impl Pointers for WindowsLinks {
    fn point(&self, link: &Path, target: &Path) -> io::Result<()> {
        let dir = link.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;
        let name = link.file_name().unwrap_or_default().to_string_lossy();
        let tmp = dir.join(format!(".{name}.new.{}", std::process::id()));
        let _ = self.remove_pointer(&tmp);
        junction::create(target, &tmp)?;
        if self.pointer_exists(link) {
            self.remove_pointer(link)?;
        }
        fs::rename(&tmp, link)
    }

    fn pointer_target(&self, link: &Path) -> Option<PathBuf> {
        junction::exists(link)
            .unwrap_or(false)
            .then(|| junction::get_target(link).ok())
            .flatten()
            .map(|p| without_verbatim(&p))
    }

    fn pointer_exists(&self, link: &Path) -> bool {
        fs::symlink_metadata(link).is_ok()
    }

    fn remove_pointer(&self, link: &Path) -> io::Result<()> {
        if junction::exists(link).unwrap_or(false) {
            // Removes the junction itself, never the folder it points at.
            fs::remove_dir(link)
        } else {
            fs::remove_file(link)
        }
    }
}

fn file(bin_dir: &Path, name: &str) -> PathBuf {
    bin_dir.join(format!("{name}.cmd"))
}

fn target_of(text: &str) -> Option<PathBuf> {
    text.lines()
        .find_map(|l| l.strip_prefix(MARK))
        .map(|t| PathBuf::from(t.trim_end()))
}

impl Commands for WindowsLinks {
    fn make_command(&self, bin_dir: &Path, name: &str, target: &Path) -> io::Result<()> {
        let path = target.to_string_lossy();
        if path.contains(['"', '%', '\r', '\n']) || name.contains(['"', '%', '\r', '\n']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a path with a quote or a percent sign cannot be a command",
            ));
        }
        fs::create_dir_all(bin_dir)?;
        // `FS_COMMAND` tells the program by which name it was called: a launcher file cannot change `argv[0]`.
        fs::write(
            file(bin_dir, name),
            format!("@echo off\r\n{MARK}{path}\r\nset \"FS_COMMAND={name}\"\r\n\"{path}\" %*\r\n"),
        )
    }

    fn command_path(&self, bin_dir: &Path, name: &str) -> PathBuf {
        file(bin_dir, name)
    }

    fn command_target(&self, bin_dir: &Path, name: &str) -> Option<PathBuf> {
        target_of(&fs::read_to_string(file(bin_dir, name)).ok()?)
    }

    fn command_exists(&self, bin_dir: &Path, name: &str) -> bool {
        file(bin_dir, name).exists()
    }

    fn remove_command(&self, bin_dir: &Path, name: &str) -> io::Result<()> {
        fs::remove_file(file(bin_dir, name))
    }

    fn commands(&self, bin_dir: &Path) -> Vec<(String, PathBuf)> {
        let Ok(dir) = fs::read_dir(bin_dir) else {
            return Vec::new();
        };
        dir.flatten()
            .filter_map(|e| {
                let path = e.path();
                let stem = path.file_stem()?.to_str()?.to_owned();
                (path.extension()? == "cmd").then_some(())?;
                Some((stem, target_of(&fs::read_to_string(&path).ok()?)?))
            })
            .collect()
    }
}
