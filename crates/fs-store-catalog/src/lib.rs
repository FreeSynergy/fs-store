//! Pattern: Adapter (`AppStream` collection XML into our package offers).
//! Format: freedesktop `AppStream` "collection metadata" (XML). Our extra fields live in `<custom>` values with the
//! `fs:` prefix, which is how `AppStream` allows extensions; no format of our own (see `DESIGN.md`).
#![deny(missing_docs)]

use fs_store_core::{Format, Kind, PackageName, Version};

/// One package as the catalog offers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// `AppStream` component id.
    pub name: PackageName,
    /// Kind (`fs:kind`, default program).
    pub kind: Kind,
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
    /// Facets the program offers (ADR 0061), `fs:facets`, comma separated.
    pub facets: Vec<String>,
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
}

/// The catalog text is not usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError(pub String);

/// Reads an `AppStream` collection.
///
/// # Errors
/// `CatalogError` when the text is not XML or not a `<components>` document.
pub fn parse_catalog(xml: &str) -> Result<Catalog, CatalogError> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| CatalogError(e.to_string()))?;
    let root = doc.root_element();
    if root.tag_name().name() != "components" {
        return Err(CatalogError("root element is not <components>".into()));
    }
    let mut out = Catalog::default();
    for c in root.children().filter(|n| n.has_tag_name("component")) {
        match entry(c) {
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

fn text<'a>(n: roxmltree::Node<'a, 'a>, tag: &str) -> Option<&'a str> {
    n.children()
        .find(|c| c.has_tag_name(tag))
        .and_then(|c| c.text())
        .map(str::trim)
}

/// `platform` like `x86_64-linux-gnu`: the architecture must be ours and the system Linux.
fn fits_this_machine(platform: &str) -> bool {
    platform.starts_with(std::env::consts::ARCH) && platform.contains("linux")
}

fn entry(c: roxmltree::Node) -> Result<CatalogEntry, (String, String)> {
    let id = text(c, "id").unwrap_or_default().to_owned();
    let fail = |r: &str| (id.clone(), r.to_owned());
    let name = PackageName::new(&id).ok_or_else(|| fail("component has no id"))?;
    let custom = |key: &str| {
        c.children()
            .find(|n| n.has_tag_name("custom"))
            .and_then(|cu| {
                cu.children()
                    .find(|v| v.has_tag_name("value") && v.attribute("key") == Some(key))
            })
            .and_then(|v| v.text())
            .map(|t| t.trim().to_owned())
    };
    // Newest release that has an artifact; unparsable versions are reported, not guessed.
    let mut best: Option<(Version, roxmltree::Node)> = None;
    let mut bad_version: Option<String> = None;
    for r in c.descendants().filter(|n| n.has_tag_name("release")) {
        // The first binary artifact for this machine (no `platform` means any).
        let Some(art) = r.descendants().find(|n| {
            n.has_tag_name("artifact")
                && n.attribute("type") != Some("source")
                && n.attribute("platform").is_none_or(fits_this_machine)
        }) else {
            continue;
        };
        let raw = r.attribute("version").unwrap_or_default();
        // A bad old release does not hide the others; it is only reported when nothing else is usable.
        let Some(v) = Version::parse(raw) else {
            bad_version.get_or_insert_with(|| format!("version `{raw}` is not major.minor.patch"));
            continue;
        };
        if best.is_none_or(|(b, _)| v > b) {
            best = Some((v, art));
        }
    }
    let (version, art) = best.ok_or_else(|| {
        fail(
            &bad_version
                .unwrap_or_else(|| "no release with an artifact for this machine".to_owned()),
        )
    })?;
    let location = text(art, "location")
        .ok_or_else(|| fail("artifact has no location"))?
        .to_owned();
    let sha256 = art
        .children()
        .find(|n| n.has_tag_name("checksum") && n.attribute("type") == Some("sha256"))
        .and_then(|n| n.text())
        .ok_or_else(|| fail("artifact has no sha256 checksum"))?
        .trim()
        .to_owned();
    let signature_url = custom("fs:signature")
        .ok_or_else(|| fail("no fs:signature (unsigned packages are never offered)"))?;
    let kind = match custom("fs:kind").as_deref() {
        None | Some("program") => Kind::Program,
        Some("frontend") => Kind::Frontend,
        Some("component") => Kind::Component,
        Some("theme") => Kind::Theme,
        Some(other) => return Err(fail(&format!("unknown fs:kind `{other}`"))),
    };
    let format = match custom("fs:format").as_deref() {
        None | Some("tar.gz") => Format::TarGz,
        Some("rpm") => Format::Rpm,
        Some("flatpak") => Format::Flatpak,
        Some("appimage") => Format::AppImage,
        Some(other) => return Err(fail(&format!("unknown fs:format `{other}`"))),
    };
    let facets = custom("fs:facets")
        .map(|f| {
            f.split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Ok(CatalogEntry {
        name,
        kind,
        version,
        format,
        location,
        sha256,
        signature_url,
        facets,
    })
}
