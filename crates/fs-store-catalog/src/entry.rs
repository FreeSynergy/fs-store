//! Pattern: Adapter (one `<component>` of the `AppStream` XML into one offer). An entry whose kind this store does not install, or that wants a
//! newer store, is read as far as it can be and carries a `locked` reason: it is never an error.

use crate::catalog::{CatalogEntry, RpmFile};
use crate::fields::{description, homepage, icon, list};
use crate::lang::lang_text;
use fs_store_core::{Format, Kind, PackageName, Version};

/// What a locked offer says when its kind (or its `fs:min-store`) is beyond this store version.
pub const NOT_SUPPORTED: &str = "not supported by this store version";

fn text<'a>(n: roxmltree::Node<'a, 'a>, tag: &str) -> Option<&'a str> {
    n.children()
        .find(|c| c.has_tag_name(tag))
        .and_then(|c| c.text())
        .map(str::trim)
}

/// `platform` like `x86_64-linux-gnu`: the architecture must be ours and the system the one this store runs on
/// (`linux`, `windows`).
fn fits_this_machine(platform: &str) -> bool {
    platform.starts_with(std::env::consts::ARCH) && platform.contains(std::env::consts::OS)
}

/// The native package formats a release may carry next to its main artifact.
const NATIVE: [&str; 3] = ["rpm", "deb", "exe"];

/// What a locked offer says when its `fs:min-store` is not `MAJOR.MINOR.PATCH`.
pub const UNREADABLE_MIN_STORE: &str = "unreadable min-store";

/// What a locked offer of kind server or icon-theme says.
pub const COMING_SOON: &str = "coming soon";

/// The kind as written in `fs:kind`, and the text of a kind this store does not know.
fn kind_of(written: Option<&str>) -> (Kind, String) {
    match written {
        None | Some("program") => (Kind::Program, String::new()),
        Some("frontend") => (Kind::Frontend, String::new()),
        Some("component") => (Kind::Component, String::new()),
        Some("theme") => (Kind::Theme, String::new()),
        Some("server") => (Kind::Server, String::new()),
        Some("icon-theme") => (Kind::IconTheme, String::new()),
        Some(other) => (Kind::Unsupported, other.to_owned()),
    }
}

/// Why this store cannot install the offer, or `None`: its kind is not installable, or `fs:min-store` is newer than this store.
fn lock_reason(kind: Kind, min_store: Option<&str>) -> Option<&'static str> {
    let own = Version::parse(env!("CARGO_PKG_VERSION"));
    // A `fs:min-store` that cannot be read locks the entry: what the entry wants is not known, so it is not guessed.
    let wanted = min_store.filter(|m| !m.is_empty());
    if wanted.is_some_and(|m| Version::parse(m).is_none()) && kind.is_installable() {
        return Some(UNREADABLE_MIN_STORE);
    }
    let too_new = wanted
        .and_then(Version::parse)
        .zip(own)
        .is_some_and(|(want, own)| want > own);
    match kind {
        Kind::Server | Kind::IconTheme => Some(COMING_SOON),
        Kind::Unsupported => Some(NOT_SUPPORTED),
        _ if too_new => Some(NOT_SUPPORTED),
        _ => None,
    }
}

/// An offer this store shows but does not install: read as far as it goes, nothing here can fail the entry. Without a release file the
/// version is `0.0.0` and the addresses are empty; nothing of it is ever downloaded.
fn locked_entry(
    c: roxmltree::Node,
    name: PackageName,
    custom: &dyn Fn(&str) -> Option<String>,
    (kind, kind_text, reason): (Kind, String, &str),
    prefs: &[String],
) -> CatalogEntry {
    let version = c
        .descendants()
        .filter(|n| n.has_tag_name("release"))
        .filter_map(|r| Version::parse(r.attribute("version")?))
        .max_by_key(|v| (v.0, v.1, v.2))
        .unwrap_or(Version::new(0, 0, 0));
    let summary = lang_text(c, "summary", prefs);
    let long_text = Some(description(c, prefs))
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| summary.clone());
    let mut details = crate::details::details(c, c, custom, &long_text, prefs);
    // Without a release file there is nothing that was signed here: no statement about the signature (shown as "unknown").
    // No release was chosen either, so the component's own description must not stand in for "what changed".
    if newest_with_artifact(c).is_err() {
        details.trust = None;
        details.changes.clear();
    }
    CatalogEntry {
        name,
        title: crate::title::plain_name(c),
        kind,
        kind_text,
        locked: Some(reason.to_owned()),
        version,
        format: Format::TarGz,
        location: String::new(),
        sha256: String::new(),
        signature_url: custom("fs:signature").unwrap_or_default(),
        rpms: Vec::new(),
        debs: Vec::new(),
        exes: Vec::new(),
        icon: icon(c),
        description: long_text.clone(),
        summary,
        homepage: homepage(c),
        facets: list(custom("fs:facets")),
        details,
    }
}

/// The newest release that has an artifact for this machine, with that artifact; unparsable versions are reported, not guessed.
fn newest_with_artifact<'a>(
    c: roxmltree::Node<'a, 'a>,
) -> Result<(Version, roxmltree::Node<'a, 'a>), String> {
    let mut best: Option<(Version, roxmltree::Node<'a, 'a>)> = None;
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
    best.ok_or_else(|| {
        bad_version.unwrap_or_else(|| "no release with an artifact for this machine".to_owned())
    })
}

pub(crate) fn entry(
    c: roxmltree::Node,
    prefs: &[String],
) -> Result<CatalogEntry, (String, String)> {
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
    let (kind, kind_text) = kind_of(custom("fs:kind").as_deref());
    if let Some(reason) = lock_reason(kind, custom("fs:min-store").as_deref()) {
        return Ok(locked_entry(
            c,
            name,
            &custom,
            (kind, kind_text, reason),
            prefs,
        ));
    }
    let (version, art) = newest_with_artifact(c).map_err(|why| fail(&why))?;
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
    let format = match custom("fs:format").as_deref() {
        None | Some("tar.gz") => Format::TarGz,
        Some("rpm") => Format::Rpm,
        Some("deb") => Format::Deb,
        Some("exe") => Format::Exe,
        Some("flatpak") => Format::Flatpak,
        Some("appimage") => Format::AppImage,
        Some(other) => return Err(fail(&format!("unknown fs:format `{other}`"))),
    };
    let facets = list(custom("fs:facets"));
    let rpms = collect_native(art, "rpm");
    let debs = collect_native(art, "deb");
    let exes = collect_native(art, "exe");
    let summary = lang_text(c, "summary", prefs);
    let long_text = Some(description(c, prefs))
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| summary.clone());
    Ok(CatalogEntry {
        name,
        title: crate::title::plain_name(c),
        kind,
        kind_text,
        locked: None,
        version,
        format,
        location,
        sha256,
        signature_url,
        rpms,
        debs,
        exes,
        icon: icon(c),
        description: long_text.clone(),
        summary,
        homepage: homepage(c),
        facets,
        details: crate::details::details(c, art, &custom, &long_text, prefs),
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
