//! Pattern: Factory + Facade (one place that builds the parts of this system and holds them).

use std::sync::OnceLock;

use crate::{
    Background, Commands, Elevation, MenuEntries, Paths, Permissions, Pointers, Processes,
    SystemPackages,
};

/// The parts of the platform this program runs on.
pub struct Platform {
    /// Where things live.
    pub paths: Box<dyn Paths>,
    /// Pointers to folders.
    pub pointers: Box<dyn Pointers>,
    /// Commands on the person's path.
    pub commands: Box<dyn Commands>,
    /// Menu entries.
    pub menu: Box<dyn MenuEntries>,
    /// Process information.
    pub processes: Box<dyn Processes>,
    /// Rights.
    pub permissions: Box<dyn Permissions>,
    /// The system's package manager.
    pub packages: Box<dyn SystemPackages>,
    /// Asking for administrator rights.
    pub elevation: Box<dyn Elevation>,
    /// The timer of the background check and the notices.
    pub background: Box<dyn Background>,
}

impl std::fmt::Debug for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Platform").finish_non_exhaustive()
    }
}

/// The platform of the machine this runs on, built once.
#[must_use]
pub fn current() -> &'static Platform {
    static PLATFORM: OnceLock<Platform> = OnceLock::new();
    PLATFORM.get_or_init(build)
}

#[cfg(unix)]
fn build() -> Platform {
    crate::linux::platform()
}

#[cfg(windows)]
fn build() -> Platform {
    crate::windows::platform()
}
