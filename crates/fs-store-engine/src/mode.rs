//! Pattern: Strategy selection (how this store was installed decides how it installs the rest).
//! Installed by rpm -> programs come as rpm; unpacked by hand or by itself -> programs come as `tar.gz`.

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

/// `Rpm` when the running executable belongs to an installed rpm, else `TarGz`.
#[must_use]
pub fn preferred_format() -> Format {
    let Ok(exe) = own_exe() else {
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
