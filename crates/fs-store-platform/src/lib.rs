//! The parts of the store that differ between Linux and Windows. Every part is a trait with two implementations,
//! `linux` (Unix) and `windows`; [`current`] hands out the ones of the machine this runs on (ADR 0043).
//! Parts: installation (`SystemPackages`, `Elevation`, `Commander`), paths (`Paths`), links instead of symlinks
//! (`Pointers`, `Commands`, `MenuEntries`), process information instead of `/proc` (`Processes`), rights (`Permissions`),
//! the timer of the background check and the notices (`Background`).
#![deny(missing_docs)]

mod background;
mod commander;
mod facade;
mod links;
#[cfg(unix)]
mod linux;
mod packages;
mod paths;
mod permissions;
mod processes;
#[cfg(windows)]
mod windows;

pub use background::{Background, Every};
pub use commander::{Commander, SystemCommander};
pub use facade::{Platform, current};
pub use links::{Commands, EntryState, MenuEntries, MenuEntry, Pointers};
#[cfg(unix)]
pub use linux::LinuxPaths;
pub use packages::{
    Elevation, System, SystemPackages, remove_deb, remove_rpm, system_from_os_release,
};
pub use paths::{Paths, Vars, display_name, process_vars};
pub use permissions::Permissions;
pub use processes::{ProcessInfo, Processes};
#[cfg(windows)]
pub use windows::WindowsPaths;
