//! Pattern: none (one pure function: the name a person reads for a package).
//! The only place that turns a package id (`fs-people`) into a display name (`FreeSynergy.People`). The id stays
//! for repository, package and command; every window, message and list shows what this returns.

/// The name a person reads for the package `id`.
///
/// `title` is the name the catalog gives (the `AppStream` `<name>`); when there is one it is used as it is. Without
/// it the name is derived: a leading `fs-` becomes `FreeSynergy.`, and each word of the rest starts with a capital
/// (`fs-people` is `FreeSynergy.People`, `fs-file-manager` is `FreeSynergy.FileManager`). A program of somebody
/// else keeps its own name with a capital (`kanidm` is `Kanidm`).
#[must_use]
pub fn display_name(id: &str, title: Option<&str>) -> String {
    if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
        return title.to_owned();
    }
    let (own, rest) = match id.strip_prefix("fs-") {
        Some(rest) => (true, rest),
        None => (false, id),
    };
    let words: String = rest
        .split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(capital)
        .collect();
    if own && !words.is_empty() {
        format!("FreeSynergy.{words}")
    } else if words.is_empty() {
        id.to_owned()
    } else {
        words
    }
}

fn capital(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}
