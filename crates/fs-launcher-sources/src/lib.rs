//! Pattern: Repository (file-backed, `SourceRepository`) with TOML as the store.
#![deny(missing_docs)]

mod file;

pub use file::{FileSources, StoreError};
