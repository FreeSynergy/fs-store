//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

pub mod net;
pub mod service;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use fs_store_core::{
    Format, InstallError, InstallStrategy, RunningCheck, VerifiedPackage, Version,
};

/// Installs a downloaded `tar.gz` under `root`.
///
/// Layout: `root/packages/<name>/<version>/` holds the files, `root/current/<name>` is a symlink to the active
/// version. Switching is one atomic rename of a new symlink; old versions stay for the rollback.
#[derive(Debug)]
pub struct TarGz {
    root: PathBuf,
    archive: PathBuf,
}

impl TarGz {
    /// `archive` is the already downloaded and signature-checked file.
    #[must_use]
    pub fn new(root: &Path, archive: &Path) -> Self {
        Self {
            root: root.to_owned(),
            archive: archive.to_owned(),
        }
    }

    fn dir(&self, name: &str, v: Version) -> PathBuf {
        self.root
            .join("packages")
            .join(name)
            .join(format!("{}.{}.{}", v.0, v.1, v.2))
    }

    fn point_current(&self, name: &str, target: &Path) -> io::Result<()> {
        let cur = self.root.join("current");
        fs::create_dir_all(&cur)?;
        let tmp = cur.join(format!(".{name}.new"));
        let _ = fs::remove_file(&tmp);
        std::os::unix::fs::symlink(target, &tmp)?;
        fs::rename(&tmp, cur.join(name))
    }

    fn unpack(&self, dest: &Path) -> io::Result<()> {
        fs::create_dir_all(dest)?;
        let mut ar = tar::Archive::new(GzDecoder::new(fs::File::open(&self.archive)?));
        for entry in ar.entries()? {
            // `unpack_in` skips entries that leave `dest` (`..`, absolute paths); that is an error for us.
            if !entry?.unpack_in(dest)? {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive entry leaves its directory",
                ));
            }
        }
        Ok(())
    }
}

fn err(e: &io::Error) -> InstallError {
    InstallError(e.to_string())
}

fn safe_name(name: &str) -> Result<(), InstallError> {
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(InstallError(format!("unsafe package name `{name}`")));
    }
    Ok(())
}

impl InstallStrategy for TarGz {
    fn format(&self) -> Format {
        Format::TarGz
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        safe_name(p.name.as_str())?;
        let dest = self.dir(p.name.as_str(), p.version);
        let _ = fs::remove_dir_all(&dest);
        if let Err(e) = self.unpack(&dest) {
            let _ = fs::remove_dir_all(&dest);
            return Err(err(&e));
        }
        self.point_current(p.name.as_str(), &dest)
            .map_err(|e| err(&e))
    }

    fn restore(&mut self, name: &str, previous: Option<Version>) -> Result<(), InstallError> {
        safe_name(name)?;
        match previous {
            Some(v) => {
                let dir = self.dir(name, v);
                if !dir.is_dir() {
                    return Err(InstallError(format!(
                        "previous version of `{name}` is gone"
                    )));
                }
                self.point_current(name, &dir).map_err(|e| err(&e))
            }
            None => match fs::remove_file(self.root.join("current").join(name)) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(err(&e)),
            },
        }
    }
}

/// Linux: a package is running when some process executes a file below `root/packages/<name>/`.
#[derive(Debug)]
pub struct ProcRunningCheck {
    root: PathBuf,
}

impl ProcRunningCheck {
    /// Looks for processes started from packages under `root`.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
        }
    }
}

impl RunningCheck for ProcRunningCheck {
    fn is_running(&self, name: &str) -> bool {
        let base = self.root.join("packages").join(name);
        let Ok(procs) = fs::read_dir("/proc") else {
            return false;
        };
        procs
            .flatten()
            .any(|p| fs::read_link(p.path().join("exe")).is_ok_and(|exe| exe.starts_with(&base)))
    }
}
