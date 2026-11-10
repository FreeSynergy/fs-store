//! Pattern: Strategy selection (how this store was installed decides how it installs the rest).
//! Installed by rpm -> programs come as rpm; installed by deb -> as deb; Windows -> `exe`; unpacked by hand or by itself
//! -> programs come as `tar.gz`.

use std::path::{Path, PathBuf};
use std::process::Command;

use fs_store_core::{Format, Version};

/// The path of the running executable. After the store updated itself by rpm, Linux reports the old file as
/// `/usr/bin/fs-store (deleted)`; that suffix is dropped, so the new file at the same path is used. Never use
/// `std::env::current_exe` directly for starting, linking or asking rpm.
///
/// # Errors
/// `io::Error` when the system cannot say.
pub fn own_exe() -> std::io::Result<PathBuf> {
    Ok(without_deleted_suffix(&std::env::current_exe()?))
}

/// `/usr/bin/fs-store (deleted)` becomes `/usr/bin/fs-store`; every other path stays as it is.
#[must_use]
pub fn without_deleted_suffix(path: &Path) -> PathBuf {
    path.to_str()
        .and_then(|t| t.strip_suffix(" (deleted)"))
        .map_or_else(|| path.to_owned(), PathBuf::from)
}

/// The package system of the machine the store runs on (Strategy selection: it decides which native package is offered).
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

/// The system this program runs on, asked once.
#[must_use]
pub fn detect_system() -> System {
    static SYSTEM: std::sync::OnceLock<System> = std::sync::OnceLock::new();
    *SYSTEM.get_or_init(|| {
        if cfg!(windows) {
            System::Windows
        } else {
            std::fs::read_to_string("/etc/os-release")
                .map_or(System::Other, |t| system_from_os_release(&t))
        }
    })
}

/// Which format the store brings for `system`: the native package when the store itself came that way
/// (`owned_by_package_manager`), else `TarGz`; on Windows always `Exe` (there is no `tar.gz` that runs there).
#[must_use]
pub fn format_for(system: System, owned_by_package_manager: bool) -> Format {
    match system {
        System::Windows => Format::Exe,
        System::Rpm if owned_by_package_manager => Format::Rpm,
        System::Deb if owned_by_package_manager => Format::Deb,
        System::Rpm | System::Deb | System::Other => Format::TarGz,
    }
}

/// `Rpm` or `Deb` when the running executable belongs to an installed package of this system, `Exe` on Windows,
/// else `TarGz`.
#[must_use]
pub fn preferred_format() -> Format {
    let system = detect_system();
    let owned = own_exe().is_ok_and(|exe| match system {
        System::Rpm => is_rpm_owned(&exe),
        System::Deb => is_deb_owned(&exe),
        System::Windows | System::Other => false,
    });
    format_for(system, owned)
}

fn is_rpm_owned(path: &Path) -> bool {
    Command::new("rpm")
        .args(["-qf", "--quiet"])
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn is_deb_owned(path: &Path) -> bool {
    Command::new("dpkg")
        .arg("-S")
        .arg(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// The version of an installed rpm called `name` (`0.1.0~rc.6` is read as `0.1.0-rc.6`), if there is one.
#[must_use]
pub fn rpm_installed_version(name: &str) -> Option<Version> {
    let out = Command::new("rpm")
        .args(["-q", "--qf", "%{VERSION}", name])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| Version::parse(String::from_utf8_lossy(&out.stdout).trim()))
        .flatten()
}

/// The version of an installed deb called `name` (upstream part, without the `-1` revision), if it is installed.
#[must_use]
pub fn deb_installed_version(name: &str) -> Option<Version> {
    let out = Command::new("dpkg-query")
        .args([
            "-W",
            "-f=${db:Status-Abbrev}|${source:Upstream-Version}",
            name,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_deb_status(&String::from_utf8_lossy(&out.stdout))
}

/// Reads `dpkg-query -W -f='${db:Status-Abbrev}|${source:Upstream-Version}'`: `ii |1.2.3` is installed, `rc |1.2.3` is
/// removed with its configuration left behind (not installed).
fn parse_deb_status(text: &str) -> Option<Version> {
    let (status, version) = text.split_once('|')?;
    status
        .trim()
        .starts_with("ii")
        .then(|| Version::parse(version.trim()))
        .flatten()
}

/// The version the package manager of this system has installed under `name`: rpm or deb, nothing elsewhere.
#[must_use]
pub fn native_installed_version(name: &str) -> Option<Version> {
    match detect_system() {
        System::Rpm => rpm_installed_version(name),
        System::Deb => deb_installed_version(name),
        System::Windows | System::Other => None,
    }
}

/// Starts the (possibly just replaced) executable again with `args` as a separate process, for "restart now" after the
/// store updated itself. The caller then ends its own process.
///
/// # Errors
/// `io::Error` when the executable cannot be found or started.
pub fn restart(args: &[String]) -> std::io::Result<()> {
    Command::new(own_exe()?)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpkg_status_installed_removed_and_prerelease() {
        assert_eq!(parse_deb_status("ii |1.2.3"), Version::parse("1.2.3"));
        assert_eq!(
            parse_deb_status("ii |0.1.0~rc.6\n"),
            Version::parse("0.1.0-rc.6")
        );
        assert_eq!(parse_deb_status("rc |1.2.3"), None);
        assert_eq!(parse_deb_status("ii |not-a-version"), None);
        assert_eq!(parse_deb_status(""), None);
    }
}
