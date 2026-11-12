//! Pattern: Value object (the languages the person reads, in order of preference). A text that comes in several languages
//! (`xml:lang` of the catalog) is chosen from this list: the language of the program, then English, then the first there is.

/// The languages to prefer, most wanted first, as primary subtags (`de`, `en`): from `FS_LANG`, else `LANG` (`de_DE.UTF-8` is `de`), with English
/// always last as the fallback. Never empty.
#[must_use]
pub fn preferred() -> Vec<String> {
    let value = std::env::var("FS_LANG")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    preferred_from(&value)
}

/// The same for a given value (for tests and for callers that know the language already).
#[must_use]
pub fn preferred_from(value: &str) -> Vec<String> {
    let primary = value
        .split(['_', '-', '.', '@'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let mut out = Vec::new();
    if !primary.is_empty() && primary != "c" && primary != "posix" && primary != "en" {
        out.push(primary);
    }
    out.push("en".to_owned());
    out
}

/// The index of the best variant of `tags` (the `xml:lang` of each, empty when it has none) for `prefs`: the first preference that is there,
/// else the one without a language when the catalog says no more, else the first. `None` for no variants.
#[must_use]
pub fn pick(tags: &[&str], prefs: &[String]) -> Option<usize> {
    if tags.is_empty() {
        return None;
    }
    let primary = |t: &str| {
        t.split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    for want in prefs {
        if let Some(i) = tags.iter().position(|t| primary(t) == *want) {
            return Some(i);
        }
    }
    Some(tags.iter().position(|t| t.is_empty()).unwrap_or(0))
}
