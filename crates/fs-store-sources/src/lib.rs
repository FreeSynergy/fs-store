//! Pattern: Repository (file-backed, `SourceRepository`; the register of installed packages) with TOML as the store.
#![deny(missing_docs)]

mod file;
mod registry;
mod serials;

pub use file::{FileSources, StoreError};
pub use registry::{Installed, Registry};
pub use serials::{Serials, Stale, ensure_fresh};
