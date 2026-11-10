//! Pattern: none (reads the display name of a catalog entry).

/// The `<name>` without `xml:lang` (the default one); only if there is none, the first translated one.
pub(crate) fn plain_name(c: roxmltree::Node) -> String {
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
