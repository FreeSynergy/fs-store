//! Pattern: Strategy (Linux: rpm and dnf, or deb and apt, by what `/etc/os-release` says), with `Pkexec` as the way to
//! ask for root.

use std::path::Path;
use std::process::{Command, Stdio};

use fs_store_core::Version;

use crate::{
    Commander, Elevation, System, SystemPackages, remove_deb, remove_rpm, system_from_os_release,
};

/// The package manager of the Linux system, found once.
pub(super) struct LinuxSystem {
    system: System,
}

impl LinuxSystem {
    pub(super) fn new() -> Self {
        let system = std::fs::read_to_string("/etc/os-release")
            .map_or(System::Other, |t| system_from_os_release(&t));
        Self { system }
    }
}

fn succeeds(program: &str, args: &[&str], path: &Path) -> bool {
    Command::new(program)
        .args(args)
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn rpm_version(name: &str) -> Option<Version> {
    let out = Command::new("rpm")
        .args(["-q", "--qf", "%{VERSION}", name])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| Version::parse(String::from_utf8_lossy(&out.stdout).trim()))
        .flatten()
}

fn deb_version(name: &str) -> Option<Version> {
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

impl SystemPackages for LinuxSystem {
    fn system(&self) -> System {
        self.system
    }

    fn installed_version(&self, name: &str) -> Option<Version> {
        match self.system {
            System::Rpm => rpm_version(name),
            System::Deb => deb_version(name),
            System::Windows | System::Other => None,
        }
    }

    fn owns_executable(&self, path: &Path) -> bool {
        match self.system {
            System::Rpm => succeeds("rpm", &["-qf", "--quiet"], path),
            System::Deb => succeeds("dpkg", &["-S"], path),
            System::Windows | System::Other => false,
        }
    }

    fn remove(&self, name: &str, commander: &mut dyn Commander) -> Result<(), String> {
        match self.system {
            System::Rpm => remove_rpm(name, commander),
            System::Deb => remove_deb(name, commander),
            System::Windows | System::Other => Err(
                "Unsupported: this system has no package manager the store can drive".to_owned(),
            ),
        }
    }
}

/// Root through `pkexec`: it asks the person, we never see a password.
pub(super) struct Pkexec;

impl Elevation for Pkexec {
    fn wrap(&self, program: &str, args: &[&str]) -> (String, Vec<String>) {
        let mut all = vec![program.to_owned()];
        all.extend(args.iter().map(|a| (*a).to_owned()));
        ("pkexec".to_owned(), all)
    }
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
