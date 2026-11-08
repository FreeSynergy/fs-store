//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

mod links;
pub mod net;
mod proc;
mod remove;
mod rpm;
pub mod service;
mod targz;

pub use links::{link_programs, unlink_programs};
pub use proc::ProcRunningCheck;
pub use remove::{RemoveError, remove};
pub use rpm::{Commander, Rpm, SystemCommander};
pub use targz::TarGz;
