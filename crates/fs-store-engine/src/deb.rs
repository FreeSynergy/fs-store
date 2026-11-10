//! Pattern: Strategy (`InstallStrategy` for `deb`, through `apt-get`), with the same `Commander` as `Rpm`, so tests need
//! neither root nor apt. Shares the steps with `Rpm` through `crate::native` (Template method).
//! The person confirms root themselves: `pkexec` asks for it, we never see a password.

use std::fs;
use std::path::{Path, PathBuf};

use fs_store_core::{Format, InstallError, InstallStrategy, VerifiedPackage, Version};

use crate::native::{names, write_private};
use crate::rpm::Commander;

/// Installs verified deb files with `pkexec apt-get install`. apt works in one transaction, so a failed install has
/// changed nothing and `restore` has nothing to undo; going back to an older deb is the person's decision, the store
/// does not do it silently.
pub struct Deb<C: Commander> {
    dir: PathBuf,
    files: Vec<Vec<u8>>,
    commander: C,
}

impl<C: Commander> Deb<C> {
    /// `dir` is a private directory for the files handed to apt (created with mode 0700).
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
            // apt-get treats an argument that contains a `/` as a file; the paths are absolute.
            let mut args = vec!["apt-get", "install", "-y"];
            args.extend(names.iter().map(String::as_str));
            self.commander.run("pkexec", &args)
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

/// Removes an installed deb with `pkexec apt-get remove -y <name>`; the person's data is never touched.
/// `name` must be a plain package name.
///
/// # Errors
/// A sentence when the name is not plain or `apt-get` fails.
pub fn remove_deb<C: Commander>(name: &str, commander: &mut C) -> Result<(), String> {
    if fs_store_core::PackageName::new(name).is_none_or(|n| n.as_str() != name) {
        return Err(format!("`{name}` is not a plain package name"));
    }
    commander.run("pkexec", &["apt-get", "remove", "-y", name])
}

/// Removes an installed package of this system the way its package manager does (`dnf remove`, `apt-get remove`).
///
/// # Errors
/// A sentence when the name is not plain, the command fails, or this system has no package manager the store drives.
pub fn remove_native<C: Commander>(name: &str, commander: &mut C) -> Result<(), String> {
    match crate::mode::detect_system() {
        crate::mode::System::Rpm => crate::rpm::remove_rpm(name, commander),
        crate::mode::System::Deb => remove_deb(name, commander),
        crate::mode::System::Windows | crate::mode::System::Other => {
            Err("Unsupported: this system has no package manager the store can drive".to_owned())
        }
    }
}
