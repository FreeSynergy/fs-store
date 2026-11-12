//! Pattern: Value object. What the person may read about a package before installing it: size, where it comes from
//! and who vouches for it, what it offers and needs, what changed. The catalog (a signed `AppStream` file) is the source;
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
    /// Name of the maker (`fs:maker`); empty when not said.
    pub maker: String,
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
    /// A block of the wiki that holds the text (`fs:description-wiki`); the store does not load it yet.
    WikiBlock(WikiRef),
}

impl Default for Description {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// Which block of the wiki holds the description, and what is known about it (ADR 0072: ONE block, a language is only a wish).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WikiRef {
    /// The block id (a `UUIDv7`).
    pub block: String,
    /// The revision of the source text this block was made from (`fs:description-wiki-source-rev`); `None` when not said.
    pub source_rev: Option<u64>,
    /// The language the text is wanted in (`fs:description-wiki-lang`); empty when no wish is said.
    pub lang: String,
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
    /// Caption in the language chosen, may be empty.
    pub caption: String,
    /// Where the picture comes from (`fs-source`: an address or a maker); empty when not said.
    pub source: String,
    /// Licence of the picture (SPDX id or expression).
    pub license: String,
}

/// A feature of a package (`fs:feature.<key>.*`): a part that has its own capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Feature {
    /// The key (`contacts`).
    pub key: String,
    /// The name for people; empty when not said.
    pub name: String,
    /// What the feature offers.
    pub provides: Vec<Capability>,
    /// What the feature must have.
    pub requires: Vec<Capability>,
    /// What the feature can use.
    pub recommends: Vec<Capability>,
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
    /// Size of the unpacked program in bytes (`<size type="installed">`); `None` when the catalog does not say.
    pub installed_size: Option<u64>,
    /// Features the package has on its own (`fs:features`), each with what it offers and needs.
    pub features: Vec<Feature>,
    /// The media types it handles (`<provides><mediatype>`), e.g. `text/calendar`.
    pub mediatypes: Vec<String>,
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

impl Details {
    /// The release date (`YYYY-MM-DD`) as Unix milliseconds at midnight UTC; `None` when the catalog gives no date or a date that is
    /// not one. The wire form of the date (`Timestamp.unixMillis`; 0 there means "not said").
    #[must_use]
    pub fn released_millis(&self) -> Option<i64> {
        Self::millis_of(&self.released)
    }

    /// The same for any date text `YYYY-MM-DD` (the date of a release in the list of changes).
    #[must_use]
    pub fn millis_of(date: &str) -> Option<i64> {
        let mut parts = date.trim().split('-');
        let (y, m, d): (i64, i64, i64) = (
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.get(..2)?.parse().ok()?,
        );
        if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return None;
        }
        // days since 1970-01-01 of a civil date (Howard Hinnant's algorithm)
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        let days = era * 146_097 + doe - 719_468;
        Some(days * 86_400_000)
    }
}
