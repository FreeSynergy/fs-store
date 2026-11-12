//! Core of the FreeSynergy launcher: traits and small value types, no input or output.
//! See `DESIGN.md` for the pattern map.
#![deny(missing_docs)]

pub mod capability;
pub mod control;
pub mod details;
pub mod names;
pub mod package;
pub mod signature;
pub mod source;
pub mod state;
pub mod strategy;
pub mod update;
pub mod words;

pub use capability::{Capability, Provider, Resolution, resolve};
pub use control::{CANCELLED, CancelToken, Progress};
pub use details::{Description, Details, Origin, ReleaseNote, Screenshot, TrustLevel};
pub use names::display_name;
pub use package::{Format, Kind, Package, PackageName, Pre, Stage, Version};
pub use signature::{
    SignatureOutcome, SignaturePolicy, SignatureVerifier, TRUSTED_FINGERPRINT, VerifiedPackage,
};
pub use source::{REFERENCE_ID, REFERENCE_URL, Source, SourceError, SourceId, SourceRepository};
pub use state::{Event, InstallState, TransitionError};
pub use strategy::{InstallError, InstallStrategy};
pub use update::{RunningCheck, Update, UpdateError, run_update};
