//! Pattern: Factory (builds the Windows parts).

use super::background::Scheduler;
use super::links::WindowsLinks;
use super::menu::Shortcuts;
use super::paths::WindowsPaths;
use super::permissions::WindowsPermissions;
use super::processes::TableProcesses;
use super::system::{Uac, WindowsSystem};
use crate::Platform;

pub(crate) fn platform() -> Platform {
    Platform {
        paths: Box::new(WindowsPaths::new()),
        pointers: Box::new(WindowsLinks),
        commands: Box::new(WindowsLinks),
        menu: Box::new(Shortcuts),
        processes: Box::new(TableProcesses),
        permissions: Box::new(WindowsPermissions),
        packages: Box::new(WindowsSystem::new()),
        background: Box::new(Scheduler),
        elevation: Box::new(Uac),
    }
}
