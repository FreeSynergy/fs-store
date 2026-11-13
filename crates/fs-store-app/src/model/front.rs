//! Pattern: Command + Query (the storefront: layout, filter tabs, sort, search, navigation, the start page and the
//! table). Pure Rust, no GUI types; the window draws what these return and calls the commands.

use fs_store_core::Kind;

use crate::words_front as w;

use super::cards::{Card, CardState};
use super::front_start::KINDS;
use super::front_text::{kind_text, or_unknown, size_text, summary, trust_word};
use super::front_types::{FrontUi, Layout, NavEntry, Section, Sort, Tab, TableRow};
use super::store::StoreModel;

impl StoreModel {
    /// The state of the storefront.
    #[must_use]
    pub fn front(&self) -> &FrontUi {
        &self.front
    }

    /// Grid or table.
    pub fn set_layout(&mut self, layout: Layout) {
        self.front.layout = layout;
    }

    /// The other layout.
    pub fn toggle_layout(&mut self) {
        self.front.layout = match self.front.layout {
            Layout::Grid => Layout::Table,
            Layout::Table => Layout::Grid,
        };
    }

    /// The filter tab.
    pub fn set_tab(&mut self, tab: Tab) {
        self.front.tab = tab;
    }

    /// The order.
    pub fn set_sort(&mut self, sort: Sort) {
        self.front.sort = sort;
    }

    /// The search text; it narrows the list at once.
    pub fn set_search(&mut self, text: &str) {
        text.clone_into(&mut self.front.search);
    }

    /// Opens a section of the catalog (closes a package page).
    pub fn open_section(&mut self, section: Section) {
        self.front.section = section;
        self.front.page = None;
    }

    /// Opens the page of a package.
    pub fn open_page(&mut self, name: &str) {
        self.front.page = Some(name.to_owned());
    }

    /// Leaves the package page.
    pub fn close_page(&mut self) {
        self.front.page = None;
    }

    /// The package whose page is open.
    #[must_use]
    pub fn page_name(&self) -> Option<&str> {
        self.front.page.as_deref()
    }

    /// What Escape does: closes the question, else the package page, else goes to the start (the search field keeps its text).
    /// True when something changed.
    pub fn escape(&mut self) -> bool {
        if self.front.preview.take().is_some() || self.front.page.take().is_some() {
            return true;
        }
        if self.front.section != Section::Start {
            self.front.section = Section::Start;
            return true;
        }
        false
    }

    fn kind_count(&self, kind: Kind) -> usize {
        self.cards().iter().filter(|c| c.kind == Some(kind)).count()
    }

    /// The navigation: the start page and a kind each (locked while it has no package; servers and icon sets say "Coming soon").
    #[must_use]
    pub fn nav(&self) -> Vec<NavEntry> {
        let section = self.front.section;
        let mut out = vec![NavEntry {
            section: Some(Section::Start),
            label: w::NAV_START,
            // the overview is no list, so it has no number; every other entry counts what its list shows
            count: None,
            enabled: true,
            current: section == Section::Start,
            note: None,
        }];
        for (kind, label) in KINDS {
            let count = self.kind_count(kind);
            // A kind this store does not install yet has a section when the catalog lists some; it says "Coming soon" either way.
            let note = if kind.is_installable() {
                (count == 0).then_some(w::NAV_NONE_YET)
            } else {
                Some(w::COMING_SOON)
            };
            out.push(NavEntry {
                section: (count > 0).then_some(Section::Kind(kind)),
                label,
                count: Some(count),
                enabled: count > 0,
                current: section == Section::Kind(kind),
                note,
            });
        }
        out
    }

    fn in_section(&self, card: &Card) -> bool {
        match self.front.section {
            Section::Start => true,
            Section::Kind(kind) => card.kind == Some(kind),
        }
    }

    fn matches_search(&self, card: &Card) -> bool {
        let needle = self.front.search.trim().to_lowercase();
        needle.is_empty()
            || [&card.title, &card.name, &card.description]
                .iter()
                .any(|t| t.to_lowercase().contains(&needle))
    }

    fn in_tab(tab: Tab, card: &Card) -> bool {
        match tab {
            Tab::All => true,
            Tab::Installed => card.installed.is_some(),
            Tab::Updates => card.state == CardState::UpdateAvailable,
            Tab::NotInstalled => card.installed.is_none(),
        }
    }

    /// How many packages each tab would show (with the section and the search, not the tab).
    #[must_use]
    pub fn tab_counts(&self) -> Vec<(Tab, usize)> {
        let base: Vec<Card> = self
            .cards()
            .into_iter()
            .filter(|c| self.in_section(c) && self.matches_search(c))
            .collect();
        Tab::ALL
            .into_iter()
            .map(|t| (t, base.iter().filter(|c| Self::in_tab(t, c)).count()))
            .collect()
    }

    fn row_of(&self, name: &str) -> Option<&super::types::AvailableRow> {
        self.available.iter().find(|r| r.package.name == name)
    }

    /// The cards of the section, narrowed by the tab and the search, in the chosen order.
    #[must_use]
    pub fn listing(&self) -> Vec<Card> {
        let mut cards: Vec<Card> = self
            .cards()
            .into_iter()
            .filter(|c| {
                self.in_section(c) && self.matches_search(c) && Self::in_tab(self.front.tab, c)
            })
            .collect();
        let key = |c: &Card| {
            let d = self.row_of(&c.name).map(|r| &r.package.details);
            (
                d.map(|d| d.released.clone()).unwrap_or_default(),
                d.and_then(|d| d.size),
                c.title.to_lowercase(),
            )
        };
        match self.front.sort {
            Sort::Name => cards.sort_by_key(|c| c.title.to_lowercase()),
            Sort::Newest => cards.sort_by(|a, b| {
                let (ka, kb) = (key(a), key(b));
                // a date first (newest on top); no date last, then by name
                ka.0.is_empty()
                    .cmp(&kb.0.is_empty())
                    .then_with(|| kb.0.cmp(&ka.0))
                    .then_with(|| ka.2.cmp(&kb.2))
            }),
            Sort::Size => cards.sort_by(|a, b| {
                let (ka, kb) = (key(a), key(b));
                ka.1.is_none()
                    .cmp(&kb.1.is_none())
                    .then_with(|| ka.1.cmp(&kb.1))
                    .then_with(|| ka.2.cmp(&kb.2))
            }),
        }
        cards
    }

    /// The table rows of the same list.
    #[must_use]
    pub fn table_rows(&self) -> Vec<TableRow> {
        self.listing()
            .into_iter()
            .map(|card| {
                let row = self.row_of(&card.name);
                let d = row.map(|r| &r.package.details);
                let (needs, needs_state) = self.needs_of(&card.name);
                TableRow {
                    summary: summary(&card.description, 60),
                    kind: kind_text(card.kind).to_owned(),
                    installed: card
                        .installed
                        .map_or_else(|| w::NONE.to_owned(), |v| v.to_string()),
                    latest: card
                        .offered
                        .map_or_else(|| w::NONE.to_owned(), |v| v.to_string()),
                    released: or_unknown(d.map_or("", |d| d.released.as_str())),
                    size: d
                        .and_then(|d| d.size)
                        .map_or_else(|| w::UNKNOWN.to_owned(), size_text),
                    signature: trust_word(d.and_then(|d| d.trust)).to_owned(),
                    needs,
                    needs_state,
                    card,
                }
            })
            .collect()
    }
}
