//! Pattern: Value object. What the person may read about a package before installing it: size, where it comes from
//! and who vouches for it, what it offers and needs, what changed. The catalog (a signed AppStream file) is the source;
//! nothing here is read from the package itself. The signer is known only after the check and lives on the job.

use crate::capability::Capability;

/// Where a package comes from (ADR 0068 point 2: foreign packages are mirrored, not linked).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Origin {
    /// Git address of the source (catalog) that offers it; set by the store, not by the catalog text.
    pub source: String,
    /// Who made it upstream (`fs:upstream`, e.g. the project page); empty for our own packages.
    pub upstream: String,
    /// Address of the upstream's own signature file that was checked when mirroring (`fs:upstream-signature`).
    pub upstream_signature: String,
    /// Address of the source code of exactly this version (`fs:source-code`), mirrored with the package.
    pub source_code: String,
}

impl Origin {
    /// A package that is mirrored from a foreign maker.
    #[must_use]
    pub fn is_mirror(&self) -> bool {
        !self.upstream.is_empty()
    }
}

/// How far a package is vouched for, shown honestly on the package (never hidden, never rounded up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    /// The maker signed it and we mirrored it: both signatures were checked.
    MakerAndMirror,
    /// Only our key vouches for it (the maker gives no signature; the checksum came from upstream).
    OurKeyOnly,
    /// Only a checksum; no signature at all. Local and test sources only.
    ChecksumOnly,
}

impl TrustLevel {
    /// The level of an offer, derived from what the signed catalog says: `own_signature` = the catalog names a signature
    /// of ours for the release file.
    #[must_use]
    pub fn of(origin: &Origin, own_signature: bool) -> Self {
        match (own_signature, origin.upstream_signature.is_empty()) {
            (false, _) => Self::ChecksumOnly,
            (true, true) => Self::OurKeyOnly,
            (true, false) => Self::MakerAndMirror,
        }
    }
}

/// The long description: text today, later a reference to a wiki block (only the field and type exist, nothing is loaded
/// from the wiki yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Description {
    /// Plain text from the catalog.
    Text(String),
    /// Id of a wiki block that holds the text (`fs:description-wiki`); the store does not load it yet.
    WikiBlock(String),
}

impl Default for Description {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// What changed in one release (`<release>` with its `<description>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseNote {
    /// Version as written in the catalog.
    pub version: String,
    /// Date (`date`), empty when none.
    pub date: String,
    /// Text, tags dropped; empty when the release has none.
    pub text: String,
}

/// A picture of the program, by the maker, shown only with a clear licence (ADR 0068 point 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screenshot {
    /// Address of the image (https).
    pub url: String,
    /// SHA-256 of the file (the catalog is signed, so the hash is trusted).
    pub sha256: String,
    /// Caption, may be empty.
    pub caption: String,
    /// Licence of the picture (SPDX id or expression).
    pub license: String,
}

/// The extra facts of one offer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Details {
    /// Size of the download in bytes (`<size type="download">`); `None` when the catalog does not say.
    pub size: Option<u64>,
    /// Date of the release (`date` of `<release>`); empty when none.
    pub released: String,
    /// What changed in this release (`<description>` of the release, tags dropped); empty when none.
    pub changes: String,
    /// Where it comes from.
    pub origin: Origin,
    /// Topic interfaces it offers (`fs:provides`).
    pub provides: Vec<Capability>,
    /// Topic interfaces it must have (`fs:requires`).
    pub requires: Vec<Capability>,
    /// Topic interfaces it can use (`fs:recommends`).
    pub recommends: Vec<Capability>,
    /// How far it is vouched for.
    pub trust: Option<TrustLevel>,
    /// The long description (text, later a wiki block).
    pub description: Description,
    /// Changes of every release the catalog lists, newest first. Not rights: see `provides`/`requires` below, which
    /// are topic interfaces (ADR 0068), nothing to do with permissions.
    pub releases: Vec<ReleaseNote>,
    /// Pictures with a clear licence.
    pub screenshots: Vec<Screenshot>,
    /// Pictures the catalog names but that are not shown, each with the reason (no licence, not https, no hash).
    pub images_withheld: Vec<String>,
    /// Entries of the three lists that could not be read; never dropped silently.
    pub unreadable: Vec<String>,
}
