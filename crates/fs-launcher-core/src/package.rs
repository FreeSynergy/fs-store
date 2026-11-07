//! Pattern: Value object (and Newtype for the name and version).
//! What can be installed: a program, frontend, component or theme, in one of several formats.

/// What kind of thing a package is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A program (engine).
    Program,
    /// A frontend for a program.
    Frontend,
    /// A component.
    Component,
    /// A theme.
    Theme,
}

/// How the package is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// rpm, installed through dnf.
    Rpm,
    /// Flatpak.
    Flatpak,
    /// AppImage.
    AppImage,
    /// tar.gz archive.
    TarGz,
}

/// Name of a package; never empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageName(String);

impl PackageName {
    /// Creates a name; `None` when empty or blank.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        let name = name.trim();
        (!name.is_empty()).then(|| Self(name.to_owned()))
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A `major.minor.patch` version, ordered numerically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    /// Parses `1.2.3`; `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.').map(|p| p.parse::<u64>().ok());
        let v = Self(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(v)
    }
}

/// One installable package as offered by a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// Name.
    pub name: PackageName,
    /// Kind.
    pub kind: Kind,
    /// Version.
    pub version: Version,
    /// Delivery format.
    pub format: Format,
    /// Id of the source it came from.
    pub source: crate::source::SourceId,
}
