//! Pattern: Strategy (choose one of several language variants of an element by `xml:lang`, see `fs_store_core::language`).

use fs_store_core::language::pick;
use roxmltree::Node;

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// Among the children of `parent` called `tag`, the one in the best language for `prefs`; `None` when there is none.
pub(crate) fn variant<'a>(
    parent: Node<'a, 'a>,
    tag: &str,
    prefs: &[String],
) -> Option<Node<'a, 'a>> {
    let all: Vec<Node> = parent.children().filter(|n| n.has_tag_name(tag)).collect();
    let tags: Vec<&str> = all
        .iter()
        .map(|n| n.attribute((XML_NS, "lang")).unwrap_or_default())
        .collect();
    pick(&tags, prefs).map(|i| all[i])
}

/// The text (tags dropped, white space squeezed) of the best variant of `tag` below `parent`; empty when there is none.
pub(crate) fn lang_text(parent: Node, tag: &str, prefs: &[String]) -> String {
    variant(parent, tag, prefs)
        .map(crate::fields::flat_text)
        .unwrap_or_default()
}
