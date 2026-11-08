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
    Format, InstallError, InstallStrategy, PackageName, RunningCheck, VerifiedPackage, Version,
};

/// Largest unpacked size and entry count we accept from one archive (decompression-bomb limit).
const MAX_UNPACKED: u64 = 4 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

enum Archive {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

/// Installs a downloaded `tar.gz` under `root`.
///
/// Layout: `root/packages/<name>/<version>/` holds the files, `root/current/<name>` is a symlink to the active
/// version. Switching is one atomic rename of a new symlink; old versions stay for the rollback. An archive is
/// unpacked into a fresh sibling directory and renamed into place, so a version that is installed is never
/// touched again. Only plain files and directories are accepted: symlink and hard-link entries are refused.
pub struct TarGz {
    root: PathBuf,
    archive: Archive,
}

impl std::fmt::Debug for TarGz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TarGz")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

/// An entry path without `.` parts; `None` when it is empty (the archive root) or leaves its directory.
fn clean(path: &Path) -> Result<Option<PathBuf>, io::Error> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::Normal(n) => out.push(n),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive entry leaves its directory",
                ));
            }
        }
    }
    Ok((!out.as_os_str().is_empty()).then_some(out))
}

impl TarGz {
    fn make(root: &Path, archive: Archive) -> Self {
        // Absolute, so the `current` links do not depend on the working directory.
        let root = std::path::absolute(root).unwrap_or_else(|_| root.to_owned());
        Self { root, archive }
    }

    /// `archive` is a downloaded and signature-checked file.
    #[must_use]
    pub fn new(root: &Path, archive: &Path) -> Self {
        Self::make(root, Archive::Path(archive.to_owned()))
    }

    /// The signature-checked bytes themselves: nothing is written to a place others could swap.
    #[must_use]
    pub fn from_bytes(root: &Path, bytes: Vec<u8>) -> Self {
        Self::make(root, Archive::Bytes(bytes))
    }

    fn dir(&self, name: &str, v: Version) -> PathBuf {
        self.root.join("packages").join(name).join(v.to_string())
    }

    fn point_current(&self, name: &str, target: &Path) -> io::Result<()> {
        let cur = self.root.join("current");
        fs::create_dir_all(&cur)?;
        let tmp = cur.join(format!(".{name}.new.{}", std::process::id()));
        let _ = fs::remove_file(&tmp);
        std::os::unix::fs::symlink(target, &tmp)?;
        fs::rename(&tmp, cur.join(name))
    }

    fn open(&self) -> io::Result<tar::Archive<GzDecoder<Box<dyn io::Read + '_>>>> {
        let src: Box<dyn io::Read + '_> = match &self.archive {
            Archive::Path(p) => Box::new(fs::File::open(p)?),
            Archive::Bytes(b) => Box::new(b.as_slice()),
        };
        Ok(tar::Archive::new(GzDecoder::new(src)))
    }

    /// The one directory every entry sits in, when there is exactly one and it is named after the package.
    fn common_top(&self, name: &str) -> io::Result<Option<std::ffi::OsString>> {
        let mut top: Option<std::ffi::OsString> = None;
        for entry in self.open()?.entries()? {
            let entry = entry?;
            let Some(path) = clean(&entry.path()?)? else {
                continue;
            };
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
        let (mut total, mut count) = (0u64, 0usize);
        for entry in self.open()?.entries()? {
            let mut entry = entry?;
            count += 1;
            total = total.saturating_add(entry.header().size()?);
            if count > MAX_ENTRIES || total > MAX_UNPACKED {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive is larger than allowed",
                ));
            }
            let kind = entry.header().entry_type();
            let Some(path) = clean(&entry.path()?)? else {
                continue;
            };
            let rel = match &top {
                Some(t) => match path.strip_prefix(t) {
                    Ok(r) if r.as_os_str().is_empty() => continue,
                    Ok(r) => r.to_owned(),
                    Err(e) => return Err(io::Error::other(e)),
                },
                None => path,
            };
            if !(kind.is_file() || kind.is_dir()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive has a link or special entry",
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
    if PackageName::new(name).is_some_and(|n| n.as_str() == name) {
        Ok(())
    } else {
        Err(InstallError(format!("unsafe package name `{name}`")))
    }
}

impl InstallStrategy for TarGz {
    fn format(&self) -> Format {
        Format::TarGz
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        safe_name(p.name.as_str())?;
        let dest = self.dir(p.name.as_str(), p.version);
        if !dest.is_dir() {
            // Unpack beside the target and rename: a half-unpacked version never exists under its name.
            let tmp = dest.with_extension(format!("tmp{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            let done = self
                .unpack(p.name.as_str(), &tmp)
                .and_then(|()| fs::rename(&tmp, &dest));
            if let Err(e) = done {
                let _ = fs::remove_dir_all(&tmp);
                return Err(err(&e));
            }
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
