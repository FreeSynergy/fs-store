//! Pattern: Adapter (small fields of one component out of `AppStream` XML).

use crate::catalog::Icon;

/// A comma separated custom value as a list without empty parts.
pub(crate) fn list(value: Option<String>) -> Vec<String> {
    value
        .map(|f| {
            f.split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// `<icon type="remote" fs-sha256="...">URL</icon>`; one without a hash is never used.
pub(crate) fn icon(c: roxmltree::Node) -> Option<Icon> {
    let n = c
        .children()
        .find(|n| n.has_tag_name("icon") && n.attribute("type") == Some("remote"))?;
    Some(Icon {
        url: n.text()?.trim().to_owned(),
        sha256: n.attribute("fs-sha256")?.to_owned(),
    })
}

/// All text below the `<description>` in the best language, tags dropped, white space squeezed.
pub(crate) fn description(c: roxmltree::Node, prefs: &[String]) -> String {
    crate::lang::lang_text(c, "description", prefs)
}

/// All text below `d`, tags dropped, white space squeezed.
pub(crate) fn flat_text(d: roxmltree::Node) -> String {
    d.descendants()
        .filter(roxmltree::Node::is_text)
        .filter_map(|n| n.text())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn homepage(c: roxmltree::Node) -> String {
    c.children()
        .find(|n| n.has_tag_name("url") && n.attribute("type") == Some("homepage"))
        .and_then(|n| n.text())
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}
