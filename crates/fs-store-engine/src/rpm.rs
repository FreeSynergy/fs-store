//! Pattern: Strategy (`InstallStrategy` for `rpm`, through `dnf`), with a small Strategy for running commands
//! (`Commander`) so tests need neither root nor dnf.
//! The person confirms root themselves: `pkexec` asks for it, we never see a password.

use std::fs;
use std::path::{Path, PathBuf};

use fs_store_core::{Format, InstallError, InstallStrategy, VerifiedPackage, Version};

use fs_store_platform::Commander;

use crate::native::{names, run_elevated, write_private};

/// Installs a verified rpm with `pkexec dnf install`. dnf works in one transaction, so a failed install has
/// changed nothing and `restore` has nothing to undo; going back to an older rpm is the person's decision
/// (`dnf downgrade`), the store does not do it silently.
pub struct Rpm<C: Commander> {
    dir: PathBuf,
    files: Vec<Vec<u8>>,
    commander: C,
}

impl<C: Commander> Rpm<C> {
    /// `dir` is a private directory for the file handed to dnf (a folder only this person can use).
    #[must_use]
    pub fn new(dir: &Path, files: Vec<Vec<u8>>, commander: C) -> Self {
        Self {
            dir: dir.to_owned(),
            files,
            commander,
        }
    }
}

impl<C: Commander> InstallStrategy for Rpm<C> {
    fn format(&self) -> Format {
        Format::Rpm
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        let stem = format!("{}-{}", p.name.as_str(), p.version);
        let (paths, held, written) = write_private(&self.dir, &stem, "rpm", &self.files);
        let result = written.and_then(|()| {
            let names = names(&paths);
            // A local rpm needs no fresh repository metadata (measured 2026-11-08: a refresh of all repositories is what
            // makes `dnf install` slow); dnf still downloads metadata when there is none.
            let mut args = vec!["install", "-y", "--setopt=metadata_expire=never"];
            args.extend(names.iter().map(String::as_str));
            run_elevated(&mut self.commander, "dnf", &args)
        });
        drop(held);
        for f in &paths {
            let _ = fs::remove_file(f);
        }
        result.map_err(InstallError)
    }

    fn restore(&mut self, _name: &str, _previous: Option<Version>) -> Result<(), InstallError> {
        Ok(())
    }
}
