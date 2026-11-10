//! Pattern: Strategy (`InstallStrategy` for the Windows `-setup.exe`), with the same `Commander` as `Rpm` and `Deb`, so
//! tests need no Windows. The setup is NSIS: `/S` runs it without a window. It needs the administrator's rights (it writes
//! below Program Files), which the person grants in the UAC prompt; we never see a password.

use std::fs;
use std::path::{Path, PathBuf};

use fs_store_core::{Format, InstallError, InstallStrategy, VerifiedPackage, Version};
use fs_store_platform::Commander;

use crate::native::{names, run_elevated, write_private};

/// Runs the verified setup files, one after the other. A setup replaces the installed version in place; going back to an
/// older one is the person's decision, the store does not do it silently.
pub struct Exe<C: Commander> {
    dir: PathBuf,
    files: Vec<Vec<u8>>,
    commander: C,
}

impl<C: Commander> Exe<C> {
    /// `dir` is a private folder for the files that are run (a folder only this person can use).
    #[must_use]
    pub fn new(dir: &Path, files: Vec<Vec<u8>>, commander: C) -> Self {
        Self {
            dir: dir.to_owned(),
            files,
            commander,
        }
    }
}

impl<C: Commander> InstallStrategy for Exe<C> {
    fn format(&self) -> Format {
        Format::Exe
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        let stem = format!("{}-{}", p.name.as_str(), p.version);
        let (paths, written) = write_private(&self.dir, &stem, "exe", &self.files);
        let result = written.and_then(|()| {
            names(&paths)
                .iter()
                .try_for_each(|file| run_elevated(&mut self.commander, file, &["/S"]))
        });
        for f in &paths {
            let _ = fs::remove_file(f);
        }
        result.map_err(InstallError)
    }

    fn restore(&mut self, _name: &str, _previous: Option<Version>) -> Result<(), InstallError> {
        Ok(())
    }
}
