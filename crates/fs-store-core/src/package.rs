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

/// Name of a package (an `AppStream` id): 1 to 128 of `A-Z a-z 0-9 . _ -`, not starting with `.` or `-`, so it is
/// safe as a file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageName(String);

impl PackageName {
    /// Creates a name; `None` when it is not a plain name.
    #[must_use]
    pub fn new(name: &str) -> Option<Self> {
        let name = name.trim();
        let plain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
        let ok = !name.is_empty()
            && name.len() <= 128
            && name.chars().all(plain)
            && !name.starts_with(['.', '-']);
        ok.then(|| Self(name.to_owned()))
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stage of a pre-release, ordered `alpha < beta < rc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// `alpha`.
    Alpha,
    /// `beta`.
    Beta,
    /// Release candidate.
    Rc,
}

/// A pre-release part such as `rc.2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pre(pub Stage, pub u64);

/// `major.minor.patch` with an optional pre-release (`-rc.2`, also written `~rc.2` in `AppStream`/rpm style).
/// A pre-release is older than the release it precedes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(pub u64, pub u64, pub u64, pub Option<Pre>);

impl Version {
    /// A release version without pre-release part.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(major, minor, patch, None)
    }

    /// Parses `1.2.3`, `1.2.3-rc.2`, `1.2.3~beta`; `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (core, pre) = match text.split_once(['-', '~']) {
            Some((c, p)) => (c, Some(p)),
            None => (text, None),
        };
        let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
        let (major, minor, patch) = (parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(())?;
        let pre = match pre {
            None => None,
            Some(text) => {
                let (stage, n) = text.split_once('.').unwrap_or((text, "0"));
                let stage = match stage {
                    "alpha" => Stage::Alpha,
                    "beta" => Stage::Beta,
                    "rc" => Stage::Rc,
                    _ => return None,
                };
                Some(Pre(stage, n.parse().ok()?))
            }
        };
        Some(Self(major, minor, patch, pre))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Greater, Less};
        (self.0, self.1, self.2)
            .cmp(&(other.0, other.1, other.2))
            .then_with(|| match (self.3, other.3) {
                (None, None) => Equal,
                (None, Some(_)) => Greater,
                (Some(_), None) => Less,
                (Some(a), Some(b)) => a.cmp(&b),
            })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)?;
        if let Some(Pre(stage, n)) = self.3 {
            let s = match stage {
                Stage::Alpha => "alpha",
                Stage::Beta => "beta",
                Stage::Rc => "rc",
            };
            write!(f, "-{s}.{n}")?;
        }
        Ok(())
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
