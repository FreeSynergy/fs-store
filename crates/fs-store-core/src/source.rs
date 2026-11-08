//! Pattern: Repository over the list of Git sources.
//! The reference source can be disabled but never removed.

/// Id of a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceId(pub String);

/// A Git source of packages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Id.
    pub id: SourceId,
    /// Git URL.
    pub url: String,
    /// Whether packages from here are offered.
    pub enabled: bool,
    /// True for the built-in FreeSynergy source.
    pub reference: bool,
}

/// Id of the built-in reference source.
pub const REFERENCE_ID: &str = "freesynergy";
/// Git URL of the built-in reference source (provisional).
pub const REFERENCE_URL: &str = "https://github.com/FreeSynergy/fs-packages";

impl Source {
    /// The built-in reference source, enabled.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            id: SourceId(REFERENCE_ID.to_owned()),
            url: REFERENCE_URL.to_owned(),
            enabled: true,
            reference: true,
        }
    }
}

/// Why a source operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    /// The built-in reference source cannot be removed.
    ReferenceSource,
    /// No source with this id.
    NotFound,
    /// A source with this id or URL exists already.
    Duplicate,
}

/// The list of sources.
pub trait SourceRepository {
    /// All sources, reference first.
    fn list(&self) -> Vec<Source>;
    /// Adds a (never reference) source.
    ///
    /// # Errors
    /// `Duplicate` when the id exists.
    fn add(&mut self, url: &str, id: SourceId) -> Result<(), SourceError>;
    /// Enables or disables a source; allowed for the reference source too.
    ///
    /// # Errors
    /// `NotFound` for an unknown id.
    fn set_enabled(&mut self, id: &SourceId, enabled: bool) -> Result<(), SourceError>;
    /// Removes a source.
    ///
    /// # Errors
    /// `ReferenceSource` for the built-in one, `NotFound` for an unknown id.
    fn remove(&mut self, id: &SourceId) -> Result<(), SourceError>;
}
