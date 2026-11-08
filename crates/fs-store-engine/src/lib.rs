//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

pub mod net;
pub mod service;

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

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
        self.root.join("packages").join(name).join(v.to_string())
    }

    fn point_current(&self, name: &str, target: &Path) -> io::Result<()> {
        let cur = self.root.join("current");
        fs::create_dir_all(&cur)?;
        let tmp = cur.join(format!(".{name}.new"));
        let _ = fs::remove_file(&tmp);
        std::os::unix::fs::symlink(target, &tmp)?;
        fs::rename(&tmp, cur.join(name))
    }

    fn open(&self) -> io::Result<tar::Archive<GzDecoder<fs::File>>> {
        Ok(tar::Archive::new(GzDecoder::new(fs::File::open(
            &self.archive,
        )?)))
    }

    /// The one directory every entry sits in, when there is exactly one and it is named after the package.
    fn common_top(&self, name: &str) -> io::Result<Option<std::ffi::OsString>> {
        let mut top: Option<std::ffi::OsString> = None;
        for entry in self.open()?.entries()? {
            let entry = entry?;
            let path = entry.path()?;
            let mut comps = path.components();
            let Some(Component::Normal(first)) = comps.next() else {
                return Ok(None);
            };
            let nested = comps.next().is_some() || entry.header().entry_type().is_dir();
            if !nested || top.as_ref().is_some_and(|t| t != first) {
                return Ok(None);
            }
            top = Some(first.to_owned());
        }
        // Only a directory named after the package is a wrapper; `bin/` or `share/` alone are content.
        Ok(top.filter(|t| t.to_str().is_some_and(|t| t.starts_with(name))))
    }

    fn unpack(&self, name: &str, dest: &Path) -> io::Result<()> {
        fs::create_dir_all(dest)?;
        let top = self.common_top(name)?;
        for entry in self.open()?.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let rel: PathBuf = match &top {
                Some(t) => path.strip_prefix(t).map_err(io::Error::other)?.to_owned(),
                None => path,
            };
            if rel.as_os_str().is_empty() {
                continue;
            }
            // Only plain names: no `..`, no absolute paths. That is an error for us, not a silent skip.
            if !rel.components().all(|c| matches!(c, Component::Normal(_))) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive entry leaves its directory",
                ));
            }
            if let Some(parent) = dest.join(&rel).parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(dest.join(&rel))?;
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
        if let Err(e) = self.unpack(p.name.as_str(), &dest) {
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
/// The calling process itself is ignored, so the store can update itself: its own old files stay on disk
/// (the switch is a symlink, nothing is overwritten) and the new version starts at the next launch.
#[derive(Debug)]
pub struct ProcRunningCheck {
    root: PathBuf,
    ignore: u32,
}

impl ProcRunningCheck {
    /// Looks for processes started from packages under `root`, ignoring this process.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self::ignoring(root, std::process::id())
    }

    /// Like [`Self::new`], ignoring the process `pid` instead.
    #[must_use]
    pub fn ignoring(root: &Path, pid: u32) -> Self {
        Self {
            root: root.to_owned(),
            ignore: pid,
        }
    }
}

impl RunningCheck for ProcRunningCheck {
    fn is_running(&self, name: &str) -> bool {
        let base = self.root.join("packages").join(name);
        let Ok(procs) = fs::read_dir("/proc") else {
            return false;
        };
        procs.flatten().any(|p| {
            let own =
                p.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) == Some(self.ignore);
            !own && fs::read_link(p.path().join("exe")).is_ok_and(|exe| exe.starts_with(&base))
        })
    }
}
