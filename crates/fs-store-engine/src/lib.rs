//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

mod facets;
mod links;
mod mode;
pub mod net;
mod proc;
mod remove;
mod rpm;
pub mod service;
mod targz;

pub use facets::{default_apps_dir, ensure_own_menu_entry, link_facets, unlink_facets};
pub use links::{default_bin_dir, link_programs, unlink_programs};
pub use mode::{preferred_format, rpm_installed_version};
pub use proc::ProcRunningCheck;
pub use remove::{RemoveError, remove};
pub use rpm::{Commander, Rpm, SystemCommander};
pub use targz::TarGz;
