//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

mod batch;
mod facets;
mod icons;
mod links;
mod mode;
pub mod net;
mod prepare;
mod proc;
mod remove;
mod rpm;
pub mod service;
mod targz;

pub use batch::{CommitObserver, Observer};
pub use facets::{default_apps_dir, ensure_own_menu_entry, link_facets, unlink_facets};
pub use icons::store_icon;
pub use links::{default_bin_dir, link_programs, unlink_programs};
pub use mode::{own_exe, preferred_format, restart, rpm_installed_version, without_deleted_suffix};
pub use prepare::{Downloaded, Plan, Prepared};
pub use proc::ProcRunningCheck;
pub use remove::{RemoveError, remove};
pub use rpm::{Commander, Rpm, SystemCommander, remove_rpm};
pub use targz::TarGz;
