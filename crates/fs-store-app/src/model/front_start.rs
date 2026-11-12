//! Pattern: Query (the start page, the footer and the one-word answer about needs; flags that put the storefront into a state for pictures
//! and tests). Pure Rust, no GUI types.

use fs_store_core::Kind;

use crate::words;
use crate::words_front as w;

use super::cards::{Card, CardState};
use super::front_types::{Layout, NeedsState, Section, StartBlock, Tab};
use super::store::StoreModel;

/// How many cards a block of the start page shows before "Show all".
const START_CARDS: usize = 4;

/// The kinds with a nav entry, in the order shown.
pub(super) const KINDS: [(Kind, &str); 4] = [
    (Kind::Program, w::NAV_PROGRAMS),
    (Kind::Frontend, w::NAV_FRONTENDS),
    (Kind::Component, w::NAV_COMPONENTS),
    (Kind::Theme, w::NAV_THEMES),
];

impl StoreModel {
    /// The blocks of the start page: updates, new packages, the recommended block (locked) and one block per kind.
    #[must_use]
    pub fn start_blocks(&self) -> Vec<StartBlock> {
        let cards = self.cards();
        let block = |title: &str, more: Option<Section>, list: Vec<Card>| StartBlock {
            title: title.to_owned(),
            more,
            total: list.len(),
            cards: list.into_iter().take(START_CARDS).collect(),
        };
        let mut out = Vec::new();
        let updates: Vec<Card> = cards
            .iter()
            .filter(|c| c.state == CardState::UpdateAvailable)
            .cloned()
            .collect();
        if !updates.is_empty() {
            out.push(block(w::START_UPDATES, None, updates));
        }
        let fresh: Vec<Card> = cards.iter().filter(|c| c.is_new).cloned().collect();
        if !fresh.is_empty() {
            out.push(block(w::START_NEW, None, fresh));
        }
        for (kind, label) in KINDS {
            let list: Vec<Card> = cards
                .iter()
                .filter(|c| c.kind == Some(kind))
                .cloned()
                .collect();
            if !list.is_empty() {
                out.push(block(label, Some(Section::Kind(kind)), list));
            }
        }
        out
    }

    /// The footer: which sources the list comes from (they are chosen in the settings).
    #[must_use]
    pub fn footer_sources(&self) -> String {
        let on: Vec<&str> = self
            .sources
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.id.as_str())
            .collect();
        if on.is_empty() {
            w::FOOTER_NO_SOURCE.to_owned()
        } else {
            words::fill(w::FOOTER_SOURCES, &[("sources", &on.join(", "))])
        }
    }

    /// The footer: how many packages the list shows.
    #[must_use]
    pub fn footer_count(&self) -> String {
        words::fill(
            w::FOOTER_COUNT,
            &[("count", &self.listing().len().to_string())],
        )
    }

    /// `(word, state)` of what the package needs.
    pub(super) fn needs_of(&self, name: &str) -> (String, NeedsState) {
        let lines = self.capability_lines(name);
        let state = if lines.requires.is_empty() {
            NeedsState::None
        } else if lines
            .requires
            .iter()
            .any(|l| l.state == NeedsState::Missing)
        {
            NeedsState::Missing
        } else if lines.requires.iter().any(|l| l.state == NeedsState::Choice) {
            NeedsState::Choice
        } else {
            NeedsState::Met
        };
        let word = match state {
            NeedsState::None => w::NEEDS_NONE,
            NeedsState::Met => w::NEEDS_MET,
            NeedsState::Choice => w::NEEDS_CHOICE,
            NeedsState::Missing => w::NEEDS_MISSING,
        };
        (word.to_owned(), state)
    }
}

/// Puts the storefront into the state a start flag asks for (`--layout`, `--section`, `--tab`, `--search`, `--open`,
/// `--preview`): for pictures and tests of the window.
///
/// # Errors
/// A sentence when the value is not one of the choices.
pub fn apply_show_flag(model: &mut StoreModel, flag: &str, value: &str) -> Result<(), String> {
    match (flag, value) {
        ("--layout", "grid") => model.set_layout(Layout::Grid),
        ("--layout", "table") => model.set_layout(Layout::Table),
        ("--section", "start") => model.open_section(Section::Start),
        ("--section", "program") => model.open_section(Section::Kind(Kind::Program)),
        ("--section", "frontend") => model.open_section(Section::Kind(Kind::Frontend)),
        ("--section", "component") => model.open_section(Section::Kind(Kind::Component)),
        ("--section", "theme") => model.open_section(Section::Kind(Kind::Theme)),
        ("--tab", "all") => model.set_tab(Tab::All),
        ("--tab", "installed") => model.set_tab(Tab::Installed),
        ("--tab", "updates") => model.set_tab(Tab::Updates),
        ("--tab", "new") => model.set_tab(Tab::NotInstalled),
        ("--search", text) => model.set_search(text),
        ("--open", name) => model.open_page(name),
        ("--preview", name) => model.ask_install(name),
        _ => return Err(format!("{flag}: `{value}` is not a choice")),
    }
    Ok(())
}
