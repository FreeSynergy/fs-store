//! What the server needs to know, as plain values.
//!
//! Pattern: Strategy — [`StoreView`] is the read side the server asks; [`crate::LocalStore`] answers
//! from the files of this machine, a test answers from memory. The server never touches files itself.

use fs_store_core::{Details, Format, Kind};

/// A source of packages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    /// Id of the source.
    pub id: String,
    /// Git address.
    pub url: String,
    /// Whether packages are offered from here.
    pub enabled: bool,
    /// The built-in FreeSynergy source.
    pub reference: bool,
}

/// One package, offered or installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    /// `AppStream` component id.
    pub name: String,
    /// Program, frontend, component or theme.
    pub kind: Kind,
    /// Version text.
    pub version: String,
    /// How it is delivered.
    pub format: Format,
    /// Id of the source (or its address, when the source is no longer listed).
    pub source: String,
    /// Facets it offers.
    pub facets: Vec<String>,
    /// One line about it (`<summary>` of the catalog); empty when the catalog has none.
    pub summary: String,
    /// What the signed catalog says besides: size, date, changes, origin, trust level, capabilities, pictures, description. Empty
    /// (`Details::default()`) for a package the store knows only from its register, so the wire says "not said" for all of it.
    pub details: Details,
    /// Lower-case hex SHA-256 of the icon file; empty when the catalog has no icon.
    pub icon_sha256: String,
}

/// A catalog entry (or a whole source) that was not offered, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedInfo {
    /// Component id, or source id when the whole source failed.
    pub id: String,
    /// Why.
    pub reason: String,
}

/// What `available` answers: the offers, and what was left out.
pub type Offers = (Vec<PackageInfo>, Vec<SkippedInfo>);

/// The read side of the store. Blocking: the server runs it off the RPC thread.
pub trait StoreView: Send + Sync {
    /// All sources, reference first.
    ///
    /// # Errors
    /// Text for the person when the list cannot be read.
    fn sources(&self) -> Result<Vec<SourceInfo>, String>;

    /// What the enabled sources offer; a source that cannot be read answers as a skipped entry.
    ///
    /// # Errors
    /// Text for the person when the list of sources cannot be read.
    fn available(&self) -> Result<Offers, String>;

    /// What is installed on this machine.
    ///
    /// # Errors
    /// Text for the person when the register cannot be read.
    fn installed(&self) -> Result<Vec<PackageInfo>, String>;
}
