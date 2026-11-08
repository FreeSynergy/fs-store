//! The command line of FreeSynergy.Store as a library: `fs-store` (crate `fs-store`) calls [`run`].
//! Pattern: Command; see `commands`.
#![deny(missing_docs)]

mod background;
mod commands;
mod source_list;
mod tuf;
mod updates;

pub use commands::run;
