//! Pattern: Strategy (`InstallStrategy` for `deb`, through `apt-get`), with the same `Commander` as `Rpm`, so tests need
//! neither root nor apt. Shares the file writing with `Rpm` through the helpers in `crate::native`.
//! The person confirms root themselves: `pkexec` asks for it, we never see a password.

use std::fs;
use std::path::{Path, PathBuf};

use fs_store_core::{Format, InstallError, InstallStrategy, VerifiedPackage, Version};

use fs_store_platform::Commander;

use crate::native::{names, run_elevated, write_private};

/// Installs verified deb files with `pkexec apt-get install`. apt works in one transaction, so a failed install has
/// changed nothing and `restore` has nothing to undo; going back to an older deb is the person's decision, the store
/// does not do it silently.
pub struct Deb<C: Commander> {
    dir: PathBuf,
    files: Vec<Vec<u8>>,
    commander: C,
}

impl<C: Commander> Deb<C> {
    /// `dir` is a private directory for the files handed to apt (a folder only this person can use).
    #[must_use]
    pub fn new(dir: &Path, files: Vec<Vec<u8>>, commander: C) -> Self {
        Self {
            dir: dir.to_owned(),
            files,
            commander,
        }
    }
}

impl<C: Commander> InstallStrategy for Deb<C> {
    fn format(&self) -> Format {
        Format::Deb
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        let stem = format!("{}-{}", p.name.as_str(), p.version);
        let (paths, written) = write_private(&self.dir, &stem, "deb", &self.files);
        let result = written.and_then(|()| {
            let names = names(&paths);
            // apt-get treats an argument that contains a `/` as a file; `write_private` makes the paths absolute.
            // `--no-remove`: unlike dnf, apt would otherwise take conflicting packages away without asking.
            let mut args = vec!["install", "-y", "--no-remove"];
            args.extend(names.iter().map(String::as_str));
            run_elevated(&mut self.commander, "apt-get", &args)
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
