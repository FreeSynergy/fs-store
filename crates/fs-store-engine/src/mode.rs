//! Pattern: Strategy selection (how this store was installed decides how it installs the rest). The questions go to the
//! platform (`SystemPackages`): installed by rpm -> programs come as rpm; installed by deb -> as deb; Windows -> `exe`;
//! unpacked by hand or by itself -> programs come as `tar.gz`.

use std::path::{Path, PathBuf};
use std::process::Command;

use fs_store_core::{Format, Version};
pub use fs_store_platform::{System, system_from_os_release};

/// The path of the running executable. After the store updated itself by rpm, Linux reports the old file as
/// `/usr/bin/fs-store (deleted)`; that suffix is dropped, so the new file at the same path is used. Never use
/// `std::env::current_exe` directly for starting, linking or asking the package manager.
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

/// The system this program runs on.
#[must_use]
pub fn detect_system() -> System {
    fs_store_platform::current().packages.system()
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
    let packages = &fs_store_platform::current().packages;
    let owned = own_exe().is_ok_and(|exe| packages.owns_executable(&exe));
    format_for(packages.system(), owned)
}

/// The version of the system package (rpm, deb or Windows setup) called `name`, if there is one.
#[must_use]
pub fn native_installed_version(name: &str) -> Option<Version> {
    fs_store_platform::current()
        .packages
        .installed_version(name)
}

/// The version of an installed rpm called `name`, if there is one (nothing on other systems).
#[must_use]
pub fn rpm_installed_version(name: &str) -> Option<Version> {
    (detect_system() == System::Rpm)
        .then(|| native_installed_version(name))
        .flatten()
}

/// The version of an installed deb called `name`, if there is one (nothing on other systems).
#[must_use]
pub fn deb_installed_version(name: &str) -> Option<Version> {
    (detect_system() == System::Deb)
        .then(|| native_installed_version(name))
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
