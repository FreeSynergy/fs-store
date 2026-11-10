//! Pattern: Strategy (the package manager of the system: dnf, apt, or the Windows setup). `Elevation` is the way to
//! ask for the administrator's rights; the person confirms, the store never sees a password.

use std::path::Path;

use fs_store_core::Version;

use crate::Commander;

/// The package system of the machine the store runs on: it decides which native package is offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    /// Fedora, RHEL and relatives that bring `dnf`: rpm through dnf (openSUSE has rpm but `zypper`: `Other`).
    Rpm,
    /// Debian, Ubuntu and relatives: deb through apt.
    Deb,
    /// Windows: `-setup.exe`.
    Windows,
    /// Anything else (Arch, NixOS, macOS, ...): no native package is offered, only the `tar.gz`.
    Other,
}

/// What the system's package manager knows and does.
pub trait SystemPackages: Send + Sync {
    /// Which system this is.
    fn system(&self) -> System;
    /// The version of the installed system package called `name`, if there is one.
    fn installed_version(&self, name: &str) -> Option<Version>;
    /// True when the file `path` belongs to an installed system package (or setup).
    fn owns_executable(&self, path: &Path) -> bool;
    /// Takes the installed system package `name` away; the person's data is never touched.
    ///
    /// # Errors
    /// A sentence when the name is not plain, the command fails, or this system has no manager the store drives.
    fn remove(&self, name: &str, commander: &mut dyn Commander) -> Result<(), String>;
}

/// How a command is run with the administrator's rights.
pub trait Elevation: Send + Sync {
    /// The program and arguments that run `program args` elevated (`pkexec` on Linux, a UAC prompt on Windows).
    fn wrap(&self, program: &str, args: &[&str]) -> (String, Vec<String>);
}

/// Reads `/etc/os-release` text: `ID` and `ID_LIKE` name the family (`ubuntu` is `debian`-like, `opensuse-leap` is
/// `suse`-like). `Windows` is never found this way.
#[must_use]
pub fn system_from_os_release(text: &str) -> System {
    let mut words = Vec::new();
    for line in text.lines() {
        if let Some(v) = line
            .strip_prefix("ID=")
            .or_else(|| line.strip_prefix("ID_LIKE="))
        {
            words.extend(
                v.trim_matches(['"', '\''])
                    .split_whitespace()
                    .map(str::to_ascii_lowercase),
            );
        }
    }
    let any = |names: &[&str]| words.iter().any(|w| names.contains(&w.as_str()));
    if any(&["debian", "ubuntu"]) {
        System::Deb
    } else if any(&["fedora", "rhel", "centos", "rocky", "almalinux"]) {
        System::Rpm
    } else {
        System::Other
    }
}

fn plain(name: &str) -> Result<(), String> {
    if fs_store_core::PackageName::new(name).is_none_or(|n| n.as_str() != name) {
        return Err(format!("`{name}` is not a plain package name"));
    }
    Ok(())
}

/// Removes an installed rpm with `pkexec dnf remove -y <name>`; the person's data is never touched.
///
/// # Errors
/// A sentence when the name is not plain or `dnf` fails.
pub fn remove_rpm(name: &str, commander: &mut dyn Commander) -> Result<(), String> {
    plain(name)?;
    commander.run("pkexec", &["dnf", "remove", "-y", name])
}

/// Removes an installed deb with `pkexec apt-get remove -y <name>`; the person's data is never touched.
///
/// # Errors
/// A sentence when the name is not plain or `apt-get` fails.
pub fn remove_deb(name: &str, commander: &mut dyn Commander) -> Result<(), String> {
    plain(name)?;
    commander.run("pkexec", &["apt-get", "remove", "-y", name])
}
