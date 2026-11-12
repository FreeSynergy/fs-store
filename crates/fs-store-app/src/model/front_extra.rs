//! Pattern: Query (the facts of format 2 of one package in words: installed size, features, media types, where a picture comes from, which
//! wiki block holds the description). Pure Rust, no GUI types.

use fs_store_core::{Details, Screenshot, WikiRef};

use crate::words;
use crate::words_front as w;

use super::front_text::{or_unknown, size_text};

/// The size line of the page: the download, and the installed size when the catalog says it; `unknown` when it says neither.
pub(super) fn size_line(d: Option<&Details>) -> String {
    let Some(d) = d else {
        return w::UNKNOWN.to_owned();
    };
    let installed = d
        .installed_size
        .map(|b| words::fill(w::SIZE_INSTALLED, &[("size", &size_text(b))]));
    match (d.size.map(size_text), installed) {
        (Some(download), Some(installed)) => format!("{download}, {installed}"),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => w::UNKNOWN.to_owned(),
    }
}

/// The rows after the origin: features (with the capabilities each offers and needs) and media types. A row only when there is something.
pub(super) fn extra_rows(d: &Details) -> Vec<(&'static str, String)> {
    let mut rows = Vec::new();
    if !d.features.is_empty() {
        let names: Vec<String> = d
            .features
            .iter()
            .map(|f| {
                let name = if f.name.is_empty() { &f.key } else { &f.name };
                let caps = |label: &str, list: &[fs_store_core::Capability]| {
                    (!list.is_empty()).then(|| {
                        let all: Vec<String> = list.iter().map(ToString::to_string).collect();
                        format!("{label} {}", all.join(", "))
                    })
                };
                let [offers, needs, uses] = crate::words_lang::feature_words(w::Lang::current());
                let parts: Vec<String> = [
                    caps(offers, &f.provides),
                    caps(needs, &f.requires),
                    caps(uses, &f.recommends),
                ]
                .into_iter()
                .flatten()
                .collect();
                if parts.is_empty() {
                    name.clone()
                } else {
                    format!("{name} ({})", parts.join("; "))
                }
            })
            .collect();
        rows.push((w::FACT_FEATURES, names.join(", ")));
    }
    if !d.mediatypes.is_empty() {
        rows.push((w::FACT_MEDIATYPES, d.mediatypes.join(", ")));
    }
    rows
}

/// The second column of a picture: its licence, and where it comes from when the catalog says.
pub(super) fn picture_line(s: &Screenshot) -> String {
    if s.source.is_empty() {
        s.license.clone()
    } else {
        words::fill(
            w::PICTURE_SOURCE,
            &[("license", &s.license), ("source", &s.source)],
        )
    }
}

/// The wiki block as said on the page: its id, and the revision it was made from when known.
pub(super) fn wiki_line(wiki: &WikiRef) -> String {
    match wiki.source_rev {
        Some(rev) => words::fill(
            w::WIKI_WITH_REV,
            &[("block", &wiki.block), ("rev", &rev.to_string())],
        ),
        None => wiki.block.clone(),
    }
}

/// The origin rows of the page: source, maker (and the maker's signature of a mirrored package), then the extra rows.
pub(super) fn origin_rows(d: Option<&Details>, source_url: &str) -> Vec<(&'static str, String)> {
    let mut origin = vec![(w::ORIGIN_SOURCE, or_unknown(source_url))];
    match d.map(|d| &d.origin) {
        Some(o) if o.is_mirror() => {
            origin.push((
                w::ORIGIN_MAKER,
                if o.maker.is_empty() {
                    o.upstream.clone()
                } else {
                    format!("{} ({})", o.maker, o.upstream)
                },
            ));
            origin.push((w::ORIGIN_MAKER_SIG, or_unknown(&o.upstream_signature)));
        }
        Some(o) if !o.maker.is_empty() => origin.push((w::ORIGIN_MAKER, o.maker.clone())),
        Some(_) => origin.push((w::ORIGIN_MAKER, w::ORIGIN_OURS.to_owned())),
        None => origin.push((w::ORIGIN_MAKER, w::UNKNOWN.to_owned())),
    }
    if let Some(d) = d {
        origin.extend(extra_rows(d));
    }
    origin
}
