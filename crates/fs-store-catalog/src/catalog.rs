//! Pattern: Adapter (`AppStream` collection XML into our package offers).
//! Format: freedesktop `AppStream` "collection metadata" (XML). Our extra fields live in `<custom>` values with the
//! `fs:` prefix, which is how `AppStream` allows extensions; no format of our own (see `DESIGN.md`).

use fs_store_core::{Format, Kind, PackageName, Version};

/// One package as the catalog offers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    /// `AppStream` component id.
    pub name: PackageName,
    /// The name to show (`<name>`, e.g. `FreeSynergy.People`); empty when the entry has none.
    pub title: String,
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
    /// The project page (`<url type="homepage">`), e.g. the Git repository; empty when there is none.
    pub homepage: String,
    /// Facets the program offers (ADR 0061), `fs:facets`, comma separated.
    pub facets: Vec<String>,
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

/// The `<name>` without `xml:lang` (the default one); only if there is none, the first translated one.
fn plain_name(c: roxmltree::Node) -> String {
    let mut names = c.children().filter(|n| n.has_tag_name("name"));
    let first = names.clone().next();
    names
        .find(|n| {
            n.attribute(("http://www.w3.org/XML/1998/namespace", "lang"))
                .is_none()
        })
        .or(first)
        .and_then(|n| n.text())
        .map(str::trim)
        .unwrap_or_default()
        .to_owned()
}

/// `platform` like `x86_64-linux-gnu`: the architecture must be ours and the system the one this store runs on
/// (`linux`, `windows`).
fn fits_this_machine(platform: &str) -> bool {
    platform.starts_with(std::env::consts::ARCH) && platform.contains(std::env::consts::OS)
}

/// The native package formats a release may carry next to its main artifact.
const NATIVE: [&str; 3] = ["rpm", "deb", "exe"];

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
                && !n
                    .attribute("fs-format")
                    .is_some_and(|f| NATIVE.contains(&f))
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
        Some("deb") => Format::Deb,
        Some("exe") => Format::Exe,
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
    let rpms = collect_native(art, "rpm");
    let debs = collect_native(art, "deb");
    let exes = collect_native(art, "exe");
    Ok(CatalogEntry {
        name,
        title: plain_name(c),
        kind,
        version,
        format,
        location,
        sha256,
        signature_url,
        rpms,
        debs,
        exes,
        icon: icon(c),
        description: Some(description(c))
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| text(c, "summary").unwrap_or_default().to_owned()),
        homepage: homepage(c),
        facets,
    })
}

/// The artifacts of `format` (`fs-format`) next to `art` in the same release that fit this machine and carry a
/// signature address.
fn collect_native(art: roxmltree::Node, format: &str) -> Vec<RpmFile> {
    art.parent()
        .into_iter()
        .flat_map(|artifacts| artifacts.children())
        .filter(|n| {
            n.has_tag_name("artifact")
                && n.attribute("fs-format") == Some(format)
                && n.attribute("platform").is_none_or(fits_this_machine)
        })
        .filter_map(|n| {
            Some(RpmFile {
                location: text(n, "location")?.to_owned(),
                sha256: n
                    .children()
                    .find(|c| c.has_tag_name("checksum") && c.attribute("type") == Some("sha256"))?
                    .text()?
                    .trim()
                    .to_owned(),
                signature_url: n.attribute("fs-signature")?.to_owned(),
            })
        })
        .collect()
}

/// `<icon type="remote" fs-sha256="...">URL</icon>`; one without a hash is never used.
fn icon(c: roxmltree::Node) -> Option<Icon> {
    let n = c
        .children()
        .find(|n| n.has_tag_name("icon") && n.attribute("type") == Some("remote"))?;
    Some(Icon {
        url: n.text()?.trim().to_owned(),
        sha256: n.attribute("fs-sha256")?.to_owned(),
    })
}

/// All text below `<description>`, tags dropped, white space squeezed.
fn description(c: roxmltree::Node) -> String {
    c.children()
        .find(|n| n.has_tag_name("description"))
        .map(|d| {
            d.descendants()
                .filter(roxmltree::Node::is_text)
                .filter_map(|n| n.text())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

fn homepage(c: roxmltree::Node) -> String {
    c.children()
        .find(|n| n.has_tag_name("url") && n.attribute("type") == Some("homepage"))
        .and_then(|n| n.text())
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}
