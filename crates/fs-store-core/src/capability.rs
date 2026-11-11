//! Pattern: Value object (`Capability`, `Need`) and Specification (`Capability::satisfies`), with one pure
//! function (`resolve`) that picks providers. ADR 0068 point 1: a package says what it offers and what it needs, the
//! store resolves; a foreign program gets its capabilities from the catalog, not from itself.

/// A versioned topic interface such as `people.persons 1.2`. The version is `major.minor`; the major part marks a
/// break, so only the same major with an equal or higher minor satisfies a need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// The topic interface (`people.persons`, `identity.oidc`); the names are the ones of `fs-interfaces`.
    pub name: String,
    /// `(major, minor)`; `None` when the catalog names none (any version).
    pub version: Option<(u32, u32)>,
}

impl Capability {
    /// Reads `people.persons`, `people.persons 1.2`, `people.persons>=1.2` or `people.persons >= 1.2`; `None` for an
    /// empty name or a version that is not `major` or `major.minor`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let at = text.find(|c: char| c.is_whitespace() || c == '>' || c == '=');
        let (name, rest) = at.map_or((text, ""), |i| text.split_at(i));
        let rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == '>' || c == '=');
        let plain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
        if name.is_empty() || !name.chars().all(plain) {
            return None;
        }
        let version = if rest.is_empty() {
            None
        } else {
            let (major, minor) = rest.split_once('.').unwrap_or((rest, "0"));
            Some((major.parse().ok()?, minor.parse().ok()?))
        };
        Some(Self {
            name: name.to_owned(),
            version,
        })
    }

    /// Reads a comma separated list; an entry that does not parse is returned as the second value (never dropped
    /// silently, the catalog reports it).
    #[must_use]
    pub fn parse_list(text: &str) -> (Vec<Self>, Vec<String>) {
        let mut good = Vec::new();
        let mut bad = Vec::new();
        for part in text.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            match Self::parse(part) {
                Some(c) => good.push(c),
                None => bad.push(part.to_owned()),
            }
        }
        (good, bad)
    }

    /// Does this offer satisfy `need`? Same name, same major, offered minor at least the needed one. A need without a
    /// version is satisfied by any version; an offer without a version satisfies only a need without one.
    #[must_use]
    pub fn satisfies(&self, need: &Self) -> bool {
        if self.name != need.name {
            return false;
        }
        match (self.version, need.version) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some((a, b)), Some((x, y))) => a == x && b >= y,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.version {
            Some((a, b)) => write!(f, "{} {a}.{b}", self.name),
            None => f.write_str(&self.name),
        }
    }
}

/// What one package offers, for `resolve`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    /// Package name.
    pub package: String,
    /// What it offers.
    pub provides: Vec<Capability>,
}

/// How a need was answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Exactly one package offers it: the store takes it.
    One(String),
    /// Several offer it: the store lets the person choose (packages in the order given).
    Choice(Vec<String>),
    /// Nobody offers it: the store says so.
    Missing,
}

/// Answers one need from the list of providers. Pure: no I/O, same input, same answer. A package never answers its own
/// need (`own`).
#[must_use]
pub fn resolve(need: &Capability, providers: &[Provider], own: &str) -> Resolution {
    let mut fit: Vec<String> = providers
        .iter()
        .filter(|p| p.package != own && p.provides.iter().any(|c| c.satisfies(need)))
        .map(|p| p.package.clone())
        .collect();
    fit.dedup();
    match fit.len() {
        0 => Resolution::Missing,
        1 => Resolution::One(fit.remove(0)),
        _ => Resolution::Choice(fit),
    }
}
