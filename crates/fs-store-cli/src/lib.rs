//! The command line of FreeSynergy.Store as a library: `fs-store` (crate `fs-store`) calls [`run`].
//! Pattern: Command; see `commands`.
#![deny(missing_docs)]

mod background;
pub use background::notification_lines;
mod batch;
mod commands;
mod locked;
mod own;
mod source_list;
mod tuf;
mod updates;

pub use commands::run;
