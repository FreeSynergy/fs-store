//! Pattern: none (one pure function: the name a person reads for a package).
//! The only place that turns a package id (`fs-people`) into a display name (`FreeSynergy.People`). The id stays
//! for repository, package and command; every window, message and list shows what this returns.

/// The name a person reads for the package `id`.
///
/// `title` is the name the catalog gives (the `AppStream` `<name>`). It is used as it is, unless it is empty or
/// still starts with `fs-` (any case): then the name is derived like for a missing title. Derived: a leading
/// `fs-` becomes `FreeSynergy.` and each word of the rest starts with a capital, joined (`fs-people` is
/// `FreeSynergy.People`, `fs-file-manager` is `FreeSynergy.FileManager`). A program of somebody else keeps its
/// own words, only the first letter is a capital (`gnome-text-editor` is `Gnome-text-editor`, `kanidm` is `Kanidm`).
#[must_use]
pub fn display_name(id: &str, title: Option<&str>) -> String {
    if let Some(title) = title
        .map(str::trim)
        .filter(|t| !t.is_empty() && !has_fs_prefix(t))
    {
        return title.to_owned();
    }
    if has_fs_prefix(id) {
        let words: String = id[3..]
            .split(['-', '_'])
            .filter(|w| !w.is_empty())
            .map(capital)
            .collect();
        if words.is_empty() {
            return id.to_owned();
        }
        return format!("FreeSynergy.{words}");
    }
    capital(id)
}

fn has_fs_prefix(text: &str) -> bool {
    text.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("fs-"))
}

fn capital(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}
