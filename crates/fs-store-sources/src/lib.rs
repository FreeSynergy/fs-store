//! Pattern: Repository (file-backed, `SourceRepository`; the register of installed packages) with TOML as the store.
#![deny(missing_docs)]

mod file;
mod registry;
mod serials;
mod settings;

pub use file::{FileSources, StoreError};
pub use registry::{Installed, Registry};
pub use serials::{Serials, Stale, ensure_fresh};
pub use settings::{CheckEvery, Findings, InstallMode, Novelty, Settings, Update};
