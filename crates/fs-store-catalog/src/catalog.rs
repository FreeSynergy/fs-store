//! Pattern: Adapter (`AppStream` collection XML into our package offers).
//! Format: freedesktop `AppStream` "collection metadata" (XML). Our extra fields live in `<custom>` values with the
//! `fs:` prefix, which is how `AppStream` allows extensions; no format of our own (see `DESIGN.md`).

use fs_store_core::{Details, Format, Kind, PackageName, Version};

/// One package as the catalog offers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// `AppStream` component id.
    pub name: PackageName,
    /// The name to show (`<name>`, e.g. `FreeSynergy.People`); empty when the entry has none.
    pub title: String,
    /// Kind (`fs:kind`, default program).
    pub kind: Kind,
    /// The `fs:kind` as written when this store does not know it (`Kind::Unsupported`); empty otherwise.
    pub kind_text: String,
    /// Why the offer is shown locked and cannot be installed (kind not known or not installable yet, or a newer store is wanted); `None` when it can be.
    pub locked: Option<String>,
    /// Newest release that has an artifact.
    pub version: Version,
    /// Delivery format (`fs:format`, default tar.gz).
    pub format: Format,
    /// Download address of the release file.
    pub location: String,
    /// SHA-256 of the release file (integrity only; the signature is the proof).
    pub sha256: String,
    /// Download address of the detached OpenPGP signature of the release file.
    pub signature_url: String,
    /// The rpm files of the same release (`fs-format="rpm"` artifacts for this machine); empty when there are none.
    pub rpms: Vec<RpmFile>,
    /// The deb files of the same release (`fs-format="deb"`, platform fits this machine); empty when there are none.
    pub debs: Vec<RpmFile>,
    /// The Windows setup files of the same release (`fs-format="exe"`, platform fits this machine); empty when none.
    pub exes: Vec<RpmFile>,
    /// The icon, when the component has a remote icon with a hash (an icon without a hash is not used).
    pub icon: Option<Icon>,
    /// A longer description (`<description>` text, tags dropped); the one-line `<summary>` when there is none.
    pub description: String,
    /// The one-line `<summary>`; empty when there is none.
    pub summary: String,
    /// The project page (`<url type="homepage">`), e.g. the Git repository; empty when there is none.
    pub homepage: String,
    /// Facets the program offers (ADR 0061), `fs:facets`, comma separated.
    pub facets: Vec<String>,
    /// Size, release date, changes, origin and capabilities (ADR 0068); see `Details`.
    pub details: Details,
}

impl CatalogEntry {
    /// The icon is a required field; a package without one is shown as "icon missing" in the sources view, never
    /// silently with a letter.
    #[must_use]
    pub fn icon_missing(&self) -> bool {
        self.icon.is_none()
    }

    /// Can this store version install the offer?
    #[must_use]
    pub fn installable(&self) -> bool {
        self.locked.is_none()
    }
}

/// One native package file of a release (rpm, deb or exe; a program may come as several, e.g. app and server).
/// The name `RpmFile` is older than deb and exe and stays so callers keep compiling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpmFile {
    /// Download address.
    pub location: String,
    /// SHA-256 of the file.
    pub sha256: String,
    /// Download address of its detached OpenPGP signature (`fs-signature` on the artifact).
    pub signature_url: String,
}

/// The icon of a package: its address and the SHA-256 of the file (the catalog is signed, so the hash is trusted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    /// Download address (PNG).
    pub url: String,
    /// SHA-256 of the file (`fs-sha256` on the `<icon type="remote">` element).
    pub sha256: String,
}

/// A component that could not be offered, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    /// Component id.
    pub id: String,
    /// Reason.
    pub reason: String,
}

/// Result of reading a catalog.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Catalog {
    /// Offers.
    pub entries: Vec<CatalogEntry>,
    /// Components left out, with the reason.
    pub skipped: Vec<Skipped>,
    /// Counts up with every published catalog (`fs-serial`); an older one than seen before is a rollback.
    pub serial: Option<u64>,
    /// Unix seconds after which the catalog is stale (`fs-expires`); a mirror cannot keep an old one alive.
    pub expires: Option<u64>,
}

/// The catalog text is not usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError(pub String);

/// Reads an `AppStream` collection.
///
/// # Errors
/// `CatalogError` when the text is not XML or not a `<components>` document.
pub fn parse_catalog(xml: &str) -> Result<Catalog, CatalogError> {
    parse_catalog_in(xml, &["en".to_owned()])
}

/// Reads an `AppStream` collection and chooses every text that comes in several languages (`xml:lang`) by `languages`, most wanted first
/// (see `fs_store_core::language::preferred`): the first that is there, else the text without a language, else the first.
///
/// # Errors
/// `CatalogError` when the text is not XML or not a `<components>` document.
pub fn parse_catalog_in(xml: &str, languages: &[String]) -> Result<Catalog, CatalogError> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| CatalogError(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "components" {
        return Err(CatalogError("root element is not <components>".into()));
    }
    let mut out = Catalog {
        serial: root.attribute("fs-serial").and_then(|v| v.parse().ok()),
        expires: root.attribute("fs-expires").and_then(|v| v.parse().ok()),
        ..Catalog::default()
    };
    for c in root.children().filter(|n| n.has_tag_name("component")) {
        match crate::entry::entry(c, languages) {
            Ok(e) if out.entries.iter().any(|o| o.name == e.name) => out.skipped.push(Skipped {
                id: e.name.as_str().to_owned(),
                reason: "duplicate id in the catalog".to_owned(),
            }),
            Ok(e) => out.entries.push(e),
            Err((id, reason)) => out.skipped.push(Skipped { id, reason }),
        }
    }
    Ok(out)
}
