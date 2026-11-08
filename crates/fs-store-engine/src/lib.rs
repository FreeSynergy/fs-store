//! Engine of FreeSynergy.Store: the parts that touch the machine. Pattern: Strategy implementations
//! (`TarGz` for `InstallStrategy`, `ProcRunningCheck` for `RunningCheck`). No root needed.
#![deny(missing_docs)]

pub mod net;
mod proc;
mod rpm;
pub mod service;
mod targz;

pub use proc::ProcRunningCheck;
pub use rpm::{Commander, Rpm, SystemCommander};
pub use targz::TarGz;
