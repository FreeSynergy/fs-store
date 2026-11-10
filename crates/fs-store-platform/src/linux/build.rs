//! Pattern: Factory (builds the Linux parts).

use super::background::Systemd;
use super::links::LinuxLinks;
use super::menu::DesktopEntries;
use super::paths::LinuxPaths;
use super::permissions::UnixPermissions;
use super::processes::ProcProcesses;
use super::system::{LinuxSystem, Pkexec};
use crate::Platform;

pub(crate) fn platform() -> Platform {
    Platform {
        paths: Box::new(LinuxPaths::new()),
        pointers: Box::new(LinuxLinks),
        commands: Box::new(LinuxLinks),
        menu: Box::new(DesktopEntries),
        processes: Box::new(ProcProcesses),
        permissions: Box::new(UnixPermissions),
        packages: Box::new(LinuxSystem::new()),
        background: Box::new(Systemd),
        elevation: Box::new(Pkexec),
    }
}
