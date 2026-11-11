//! Pattern: Adapter (the details of one offer out of `AppStream` XML). Standard elements where `AppStream` has them
//! (`<size type="download">`, the release `<description>` and `date`), our own facts as `fs:` custom values (ADR 0068).

use fs_store_core::{
    Capability, Description, Details, Origin, ReleaseNote, Screenshot, TrustLevel,
};
use roxmltree::Node;

/// The details of the offer whose artifact is `art` (the main file of the chosen release).
pub(crate) fn details(
    component: Node,
    art: Node,
    custom: &dyn Fn(&str) -> Option<String>,
) -> Details {
    let release = art
        .ancestors()
        .find(|n| n.has_tag_name("release"))
        .unwrap_or(component);
    let mut unreadable = Vec::new();
    let mut list = |key: &str| {
        let (good, bad) = Capability::parse_list(&custom(key).unwrap_or_default());
        unreadable.extend(bad.into_iter().map(|b| format!("{key}: `{b}`")));
        good
    };
    let (provides, requires, recommends) = (
        list("fs:provides"),
        list("fs:requires"),
        list("fs:recommends"),
    );
    let origin = Origin {
        source: String::new(),
        upstream: custom("fs:upstream").unwrap_or_default(),
        upstream_signature: custom("fs:upstream-signature").unwrap_or_default(),
        source_code: custom("fs:source-code").unwrap_or_default(),
    };
    let (screenshots, images_withheld) = screenshots(component);
    Details {
        trust: Some(TrustLevel::of(
            &origin,
            custom("fs:signature").is_some_and(|s| !s.is_empty()),
        )),
        description: custom("fs:description-wiki")
            .filter(|w| !w.is_empty())
            .map_or_else(|| Description::Text(String::new()), Description::WikiBlock),
        releases: release_notes(component),
        screenshots,
        images_withheld,
        size: art
            .children()
            .find(|n| n.has_tag_name("size") && n.attribute("type") == Some("download"))
            .and_then(|n| n.text())
            .and_then(|t| t.trim().parse().ok()),
        released: release.attribute("date").unwrap_or_default().to_owned(),
        changes: release
            .children()
            .find(|n| n.has_tag_name("description"))
            .map(crate::catalog::flat_text)
            .unwrap_or_default(),
        origin,
        provides,
        requires,
        recommends,
        unreadable,
    }
}

/// Every release with its text, newest first (by version when it parses, else as listed).
fn release_notes(component: Node) -> Vec<ReleaseNote> {
    let mut all: Vec<(Option<fs_store_core::Version>, ReleaseNote)> = component
        .descendants()
        .filter(|n| n.has_tag_name("release"))
        .map(|r| {
            let version = r.attribute("version").unwrap_or_default();
            (
                fs_store_core::Version::parse(version),
                ReleaseNote {
                    version: version.to_owned(),
                    date: r.attribute("date").unwrap_or_default().to_owned(),
                    text: r
                        .children()
                        .find(|n| n.has_tag_name("description"))
                        .map(crate::catalog::flat_text)
                        .unwrap_or_default(),
                },
            )
        })
        .collect();
    all.sort_by(|a, b| b.0.cmp(&a.0));
    all.into_iter().map(|(_, n)| n).collect()
}

/// A licence is clear when it is named and is not one of the words that mean "unknown".
fn clear_license(l: &str) -> bool {
    let l = l.trim();
    let plain = |c: char| c.is_ascii_alphanumeric() || " .-+()".contains(c);
    !l.is_empty()
        && l.chars().all(plain)
        && !["unknown", "none", "proprietary", "all rights reserved"]
            .contains(&l.to_ascii_lowercase().as_str())
}

/// `<screenshots><screenshot><image type="source" fs-sha256=".." fs-license="..">URL</image><caption>..`. A picture is
/// shown only with an https address, a hash and a clear licence; any other is listed as withheld with the reason.
fn screenshots(component: Node) -> (Vec<Screenshot>, Vec<String>) {
    let (mut shown, mut withheld) = (Vec::new(), Vec::new());
    for s in component
        .descendants()
        .filter(|n| n.has_tag_name("screenshot"))
    {
        let Some(img) = s
            .children()
            .find(|n| n.has_tag_name("image") && n.attribute("type") != Some("thumbnail"))
        else {
            continue;
        };
        let url = img.text().unwrap_or_default().trim();
        let license = img.attribute("fs-license").unwrap_or_default();
        let sha256 = img.attribute("fs-sha256").unwrap_or_default();
        let why = if !url.starts_with("https://") {
            Some("not an https address")
        } else if sha256.is_empty() {
            Some("no hash")
        } else if !clear_license(license) {
            Some("no clear licence")
        } else {
            None
        };
        match why {
            Some(w) => withheld.push(format!("{url}: {w}")),
            None => shown.push(Screenshot {
                url: url.to_owned(),
                sha256: sha256.to_owned(),
                caption: s
                    .children()
                    .find(|n| n.has_tag_name("caption"))
                    .map(crate::catalog::flat_text)
                    .unwrap_or_default(),
                license: license.trim().to_owned(),
            }),
        }
    }
    (shown, withheld)
}
