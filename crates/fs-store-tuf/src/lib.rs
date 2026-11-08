//! Pattern: Adapter (`tough` behind one function). TUF (ADR 0062) protects against rollback, freeze and
//! mix-and-match: the client holds a trusted `root.json`, follows its signed chain (root, timestamp, snapshot, targets)
//! and only then believes a target file. Metadata is checked for signatures, versions that only go up, and expiry.
//!
//! The metadata and target addresses are `file://` directories or `https://` bases (the store clones a Git source and
//! hands the directories over, so no extra network code is here).
#![deny(missing_docs)]

use std::path::Path;

use tough::{IntoVec, RepositoryLoader, TargetName};
use url::Url;

/// Why a file could not be read through TUF.
#[derive(Debug)]
pub enum TufError {
    /// The metadata chain is not valid for the trusted root (signature, version, expiry, missing role).
    Metadata(String),
    /// The target is not listed in the signed metadata.
    NotListed(String),
    /// The target bytes could not be read, or do not match the signed length or hash.
    Target(String),
    /// A path or name is not usable.
    Input(String),
}

/// Reads target `name` from the repository whose signed metadata is in `metadata_dir` and whose target files are in
/// `targets_dir`, trusting only `root` (the bytes of a `root.json`). Verification is complete before any byte is returned.
///
/// # Errors
/// `TufError` on any failure; nothing is returned that was not verified.
pub fn read_target(
    root: &[u8],
    metadata_dir: &Path,
    targets_dir: &Path,
    name: &str,
) -> Result<Vec<u8>, TufError> {
    let url = |p: &Path| {
        Url::from_directory_path(p)
            .map_err(|()| TufError::Input(format!("{}: not an absolute directory", p.display())))
    };
    let (metadata, targets) = (url(metadata_dir)?, url(targets_dir)?);
    let target = TargetName::new(name).map_err(|e| TufError::Input(e.to_string()))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| TufError::Input(e.to_string()))?;
    runtime.block_on(async {
        let repo = RepositoryLoader::new(&root, metadata, targets)
            .load()
            .await
            .map_err(|e| TufError::Metadata(e.to_string()))?;
        let stream = repo
            .read_target(&target)
            .await
            .map_err(|e| TufError::Target(e.to_string()))?
            .ok_or_else(|| TufError::NotListed(name.to_owned()))?;
        stream
            .into_vec()
            .await
            .map_err(|e| TufError::Target(e.to_string()))
    })
}
