//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

mod batch;
mod deb;
mod end;
mod ending;
mod facets;
mod icons;
mod links;
mod mode;
mod native;
pub mod net;
mod prepare;
mod proc;
mod remove;
mod rpm;
mod servers;
pub mod service;
mod targz;

pub use batch::{CommitObserver, Observer};
pub use deb::{Deb, remove_deb, remove_native};
pub use end::{Ended, end_server, end_servers, server_info, socket_of, stop_servers};
pub use facets::{default_apps_dir, ensure_own_menu_entry, link_facets, unlink_facets};
pub use icons::store_icon;
pub use links::{default_bin_dir, link_programs, unlink_programs};
pub use mode::{
    System, deb_installed_version, detect_system, format_for, native_installed_version, own_exe,
    preferred_format, restart, rpm_installed_version, system_from_os_release,
    without_deleted_suffix,
};
pub use prepare::{Downloaded, Plan, Prepared};
pub use proc::ProcRunningCheck;
pub use remove::{RemoveError, remove};
pub use rpm::{Commander, Rpm, SystemCommander, remove_rpm};
pub use servers::{Server, find_servers, is_alive, is_server_of, program_of, start_server};
pub use targz::TarGz;
