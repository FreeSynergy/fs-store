//! Pattern: Adapter (the format 2 fields of one component: installed size, wiki reference, features, media types).

use fs_store_core::{Capability, Feature, WikiRef};
use roxmltree::Node;

/// `<size type="installed">` of the artifact, in bytes.
pub(crate) fn installed_size(art: Node) -> Option<u64> {
    art.children()
        .find(|n| n.has_tag_name("size") && n.attribute("type") == Some("installed"))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse().ok())
}

/// `fs:description-wiki` (ONE block id), `fs:description-wiki-source-rev` and the optional `fs:description-wiki-lang`. Loading the block is not
/// done here nor anywhere in the store yet.
pub(crate) fn wiki(custom: &dyn Fn(&str) -> Option<String>) -> Option<WikiRef> {
    let block = custom("fs:description-wiki").filter(|w| !w.is_empty())?;
    Some(WikiRef {
        block,
        source_rev: custom("fs:description-wiki-source-rev").and_then(|r| r.parse().ok()),
        lang: custom("fs:description-wiki-lang").unwrap_or_default(),
    })
}

/// `<provides><mediatype>text/calendar</mediatype></provides>`, in the order written, without empty or doubled ones.
pub(crate) fn mediatypes(component: Node) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in component
        .children()
        .filter(|n| n.has_tag_name("provides"))
        .flat_map(|p| p.children())
        .filter(|n| n.has_tag_name("mediatype"))
        .filter_map(|n| n.text())
        .map(|t| t.trim().to_owned())
    {
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// `fs:features` (comma list of keys) with `fs:feature.<key>.provides|requires|recommends|name`. A capability the list does not know is
/// reported in `unreadable`, as for the package itself.
pub(crate) fn features(
    custom: &dyn Fn(&str) -> Option<String>,
    unreadable: &mut Vec<String>,
) -> Vec<Feature> {
    crate::fields::list(custom("fs:features"))
        .into_iter()
        .map(|key| {
            let mut caps = |what: &str| {
                let full = format!("fs:feature.{key}.{what}");
                let (good, bad) = Capability::parse_list(&custom(&full).unwrap_or_default());
                unreadable.extend(bad.into_iter().map(|b| format!("{full}: `{b}`")));
                good
            };
            let (provides, requires, recommends) =
                (caps("provides"), caps("requires"), caps("recommends"));
            Feature {
                name: custom(&format!("fs:feature.{key}.name")).unwrap_or_default(),
                key,
                provides,
                requires,
                recommends,
            }
        })
        .collect()
}
