//! Pattern: Query (the page of one package: every fact the catalog gives, in words; a fact it does not give is
//! "unknown"). Capabilities (what it offers, needs, can use) are shown apart from rights, and each need says who provides it
//! (the resolver of the core). Nothing is loaded from the wiki or the network here.

use fs_store_core::{Capability, Description, Provider, Resolution, resolve};

use crate::words;
use crate::words_front as w;

use super::cards::{Card, Placeholder};
use super::front_text::{kind_text, or_unknown, size_text, trust_sentence, trust_word};
use super::front_types::NeedsState;
use super::store::StoreModel;

/// One line of a capability list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapLine {
    /// The capability and its level.
    pub text: String,
    /// How it is answered (for a need); `Met` for the lines that need no answer.
    pub state: NeedsState,
    /// The answer in words (who provides it); empty for an offer.
    pub answer: String,
}

/// The three lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityLines {
    /// What it offers.
    pub provides: Vec<CapLine>,
    /// What it must have.
    pub requires: Vec<CapLine>,
    /// What it can use.
    pub recommends: Vec<CapLine>,
}

/// One release with its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseLine {
    /// The version.
    pub version: String,
    /// The date or `unknown`.
    pub date: String,
    /// The text; empty when the release has none.
    pub text: String,
}

/// The description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionView {
    /// Text of the catalog.
    Text(String),
    /// A block of the wiki; only said, not loaded.
    Wiki(String),
}

/// The whole page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePage {
    /// The card (buttons, state, working).
    pub card: Card,
    /// Kind in words.
    pub kind: String,
    /// `1.2.0 (2026-11-09)` or `-> 1.3.0` for an update.
    pub version: String,
    /// Download size or `unknown`.
    pub size: String,
    /// Where it comes from: label and value.
    pub origin: Vec<(&'static str, String)>,
    /// The signature level in a word.
    pub trust_word: &'static str,
    /// What the level means.
    pub trust_text: &'static str,
    /// Offers, needs, can use.
    pub capabilities: CapabilityLines,
    /// Pictures with a clear licence: caption and licence.
    pub pictures: Vec<(String, String)>,
    /// What to say about the pictures.
    pub pictures_note: String,
    /// Changes per version, newest first.
    pub releases: Vec<ReleaseLine>,
    /// The description.
    pub description: DescriptionView,
    /// The project page; empty when none.
    pub homepage: String,
    /// The source code of this version; empty when none.
    pub source_code: String,
    /// The catalog has no icon.
    pub icon_missing: bool,
    /// One line for the blocks that have nothing to show ("Offers nothing \u{b7} Uses nothing extra"); `None` when every block has something.
    pub nothing_line: Option<String>,
    /// The stand-in for the logo.
    pub placeholder: Placeholder,
}

fn lines_of(list: &[Capability]) -> Vec<CapLine> {
    list.iter()
        .map(|c| CapLine {
            text: c.to_string(),
            state: NeedsState::Met,
            answer: String::new(),
        })
        .collect()
}

impl StoreModel {
    /// Who offers what: one provider entry per offered package.
    fn providers(&self) -> Vec<Provider> {
        let mut out: Vec<Provider> = self
            .available
            .iter()
            .map(|r| Provider {
                package: r.package.name.clone(),
                provides: r.package.details.provides.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.package.cmp(&b.package));
        out
    }

    /// The capability lists of `name`; every need is answered by the resolver.
    #[must_use]
    pub fn capability_lines(&self, name: &str) -> CapabilityLines {
        let Some(row) = self.available.iter().find(|r| r.package.name == name) else {
            return CapabilityLines::default();
        };
        let d = &row.package.details;
        let providers = self.providers();
        let answer = |need: &Capability| -> CapLine {
            let (state, text) = match resolve(need, &providers, name) {
                Resolution::One(by) => (
                    NeedsState::Met,
                    words::fill(w::CAP_MET_BY, &[("by", &self.title(&by))]),
                ),
                Resolution::Choice(list) => {
                    let by: Vec<String> = list.iter().map(|p| self.title(p)).collect();
                    (
                        NeedsState::Choice,
                        words::fill(w::CAP_CHOICE, &[("by", &by.join(", "))]),
                    )
                }
                Resolution::Missing => (NeedsState::Missing, w::CAP_MISSING.to_owned()),
            };
            CapLine {
                text: need.to_string(),
                state,
                answer: text,
            }
        };
        CapabilityLines {
            provides: lines_of(&d.provides),
            requires: d.requires.iter().map(&answer).collect(),
            recommends: d.recommends.iter().map(&answer).collect(),
        }
    }

    /// The page of the selected package (`None` when none is selected or it is gone).
    #[must_use]
    pub fn package_page_selected(&self) -> Option<PackagePage> {
        self.package_page(self.page_name()?)
    }

    /// The page of `name`, as it is now; `None` when no source offers it and it is not installed.
    #[must_use]
    pub fn package_page(&self, name: &str) -> Option<PackagePage> {
        let card = self.cards().into_iter().find(|c| c.name == name)?;
        let row = self.available.iter().find(|r| r.package.name == name);
        let d = row.map(|r| &r.package.details);
        let source_url = row
            .and_then(|r| self.sources.iter().find(|s| s.id == r.package.source))
            .map(|s| s.url.clone())
            .unwrap_or_default();
        let mut origin = vec![(w::ORIGIN_SOURCE, or_unknown(&source_url))];
        match d.map(|d| &d.origin) {
            Some(o) if o.is_mirror() => {
                origin.push((w::ORIGIN_MAKER, o.upstream.clone()));
                origin.push((w::ORIGIN_MAKER_SIG, or_unknown(&o.upstream_signature)));
            }
            Some(_) => origin.push((w::ORIGIN_MAKER, w::ORIGIN_OURS.to_owned())),
            None => origin.push((w::ORIGIN_MAKER, w::UNKNOWN.to_owned())),
        }
        let released = or_unknown(d.map_or("", |d| d.released.as_str()));
        let version = match (card.version_line(), released.as_str()) {
            (line, w::UNKNOWN) => line,
            (line, date) => format!("{line} ({date})"),
        };
        let shown = d.map_or(0, |d| d.screenshots.len());
        let withheld = d.map_or(0, |d| d.images_withheld.len());
        let mut notes = Vec::new();
        if shown > 0 {
            notes.push(words::fill(
                w::PICTURES_SHOWN,
                &[("count", &shown.to_string())],
            ));
        }
        if withheld > 0 {
            notes.push(words::fill(
                w::PICTURES_WITHHELD,
                &[("count", &withheld.to_string())],
            ));
        }
        if notes.is_empty() {
            notes.push(w::PICTURES_NONE.to_owned());
        }
        let trust = d.and_then(|d| d.trust);
        Some(PackagePage {
            kind: kind_text(card.kind).to_owned(),
            version,
            size: d
                .and_then(|d| d.size)
                .map_or_else(|| w::UNKNOWN.to_owned(), size_text),
            origin,
            trust_word: trust_word(trust),
            trust_text: trust_sentence(trust),
            capabilities: self.capability_lines(name),
            pictures: d
                .map(|d| {
                    d.screenshots
                        .iter()
                        .map(|s| (s.caption.clone(), s.license.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            pictures_note: notes.join(" "),
            releases: d
                .map(|d| {
                    d.releases
                        .iter()
                        .map(|r| ReleaseLine {
                            version: r.version.clone(),
                            date: or_unknown(&r.date),
                            text: r.text.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            description: match d.map(|d| &d.description) {
                Some(Description::WikiBlock(id)) => DescriptionView::Wiki(id.clone()),
                _ => DescriptionView::Text(card.description_text()),
            },
            homepage: card.homepage.clone(),
            source_code: d.map(|d| d.origin.source_code.clone()).unwrap_or_default(),
            icon_missing: card.icon.is_none() && row.is_some(),
            nothing_line: {
                let caps = self.capability_lines(name);
                crate::words_front::nothing_line(
                    crate::words_front::Lang::current(),
                    crate::words_front::EmptyBlocks {
                        offers: caps.provides.is_empty(),
                        needs: caps.requires.is_empty(),
                        uses: caps.recommends.is_empty(),
                        pictures: shown == 0,
                    },
                )
            },
            placeholder: card.placeholder,
            card,
        })
    }
}
