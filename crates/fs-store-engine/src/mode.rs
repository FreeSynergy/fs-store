//! Pattern: Strategy selection (how this store was installed decides how it installs the rest).
//! Installed by rpm -> programs come as rpm; unpacked by hand or by itself -> programs come as `tar.gz`.

use std::path::Path;
use std::process::Command;

use fs_store_core::{Format, Version};

/// `Rpm` when the running executable belongs to an installed rpm, else `TarGz`.
#[must_use]
pub fn preferred_format() -> Format {
    let Ok(exe) = std::env::current_exe() else {
        return Format::TarGz;
    };
    if is_rpm_owned(&exe) {
        Format::Rpm
    } else {
        Format::TarGz
    }
}

fn is_rpm_owned(path: &Path) -> bool {
    Command::new("rpm")
        .args(["-qf", "--quiet"])
        .arg(path)
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
