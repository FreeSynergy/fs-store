//! Pattern: Repository (file-backed, `SourceRepository`; the register of installed packages) with TOML as the store.
#![deny(missing_docs)]

mod file;
mod registry;

pub use file::{FileSources, StoreError};
pub use registry::{Installed, Registry};
