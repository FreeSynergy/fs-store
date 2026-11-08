//! The command line of FreeSynergy.Store as a library: `fs-store` (crate `fs-store`) calls [`run`].
//! Pattern: Command; see `commands`.
#![deny(missing_docs)]

mod commands;

pub use commands::run;
