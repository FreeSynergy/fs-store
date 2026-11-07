//! Core of the FreeSynergy launcher: traits and small value types, no input or output.
//! See `DESIGN.md` for the pattern map.
#![deny(missing_docs)]

pub mod package;
pub mod signature;
pub mod source;
pub mod state;
pub mod strategy;
pub mod update;

pub use package::{Format, Kind, Package, PackageName, Version};
pub use signature::{
    SignatureOutcome, SignaturePolicy, SignatureVerifier, TRUSTED_FINGERPRINT, VerifiedPackage,
};
pub use source::{Source, SourceError, SourceId, SourceRepository};
pub use state::{Event, InstallState, TransitionError};
pub use strategy::{InstallError, InstallStrategy};
pub use update::{RunningCheck, Update, UpdateError, run_update};
