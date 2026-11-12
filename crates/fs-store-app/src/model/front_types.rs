//! Pattern: none (plain data of the storefront: layout, filter, sort, navigation, the rows of the table).

use fs_store_core::Kind;

use super::cards::Card;
use super::front_steps::Preview;

/// How the packages are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// Cards.
    #[default]
    Grid,
    /// A table with a column for every fact.
    Table,
}

/// The filter tabs above the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Everything.
    #[default]
    All,
    /// Installed packages.
    Installed,
    /// Installed packages with a newer version offered.
    Updates,
    /// Offered, not installed.
    NotInstalled,
}

impl Tab {
    /// All tabs in the order shown.
    pub const ALL: [Tab; 4] = [Tab::All, Tab::Installed, Tab::Updates, Tab::NotInstalled];
}

/// The order of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// By title.
    #[default]
    Name,
    /// Newest release first; packages without a date last.
    Newest,
    /// Smallest download first; packages without a size last.
    Size,
}

impl Sort {
    /// All choices in the order shown.
    pub const ALL: [Sort; 3] = [Sort::Name, Sort::Newest, Sort::Size];
}

/// Where in the catalog the person is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    /// The start page.
    #[default]
    Start,
    /// All packages of one kind.
    Kind(Kind),
}

/// One entry of the navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavEntry {
    /// What it opens; `None` for an entry that is not possible yet.
    pub section: Option<Section>,
    /// The words.
    pub label: &'static str,
    /// How many packages are behind it.
    pub count: usize,
    /// It can be chosen.
    pub enabled: bool,
    /// It is the one shown.
    pub current: bool,
    /// Why it is locked (shown as a tooltip), when it is.
    pub note: Option<&'static str>,
}

/// What a package says about its needs, in one word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedsState {
    /// It needs nothing.
    None,
    /// Everything it needs has one provider.
    Met,
    /// Something it needs has several providers: the person chooses.
    Choice,
    /// Something it needs has no provider.
    Missing,
}

/// One row of the table: the card and the facts of its columns, already as words ("unknown" where the catalog is silent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    /// The card (name, buttons, state).
    pub card: Card,
    /// A short line under the name.
    pub summary: String,
    /// Kind in words.
    pub kind: String,
    /// Installed version or `-`.
    pub installed: String,
    /// Offered version or `-`.
    pub latest: String,
    /// Release date or `unknown`.
    pub released: String,
    /// Download size or `unknown`.
    pub size: String,
    /// Signature level in words.
    pub signature: String,
    /// Needs in a word.
    pub needs: String,
    /// Needs as a state, for the colour.
    pub needs_state: NeedsState,
}

/// A block of the start page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartBlock {
    /// Heading.
    pub title: String,
    /// The section "Show all" opens; `None` for the blocks that are not a section.
    pub more: Option<Section>,
    /// How many packages are in the block in all.
    pub total: usize,
    /// The first few.
    pub cards: Vec<Card>,
}

/// The state of the storefront the person changes.
#[derive(Debug, Clone, Default)]
pub struct FrontUi {
    pub(super) layout: Layout,
    pub(super) tab: Tab,
    pub(super) sort: Sort,
    pub(super) section: Section,
    pub(super) search: String,
    pub(super) page: Option<String>,
    pub(super) preview: Option<Preview>,
}

impl FrontUi {
    /// Grid or table.
    #[must_use]
    pub fn layout(&self) -> Layout {
        self.layout
    }
    /// The filter tab.
    #[must_use]
    pub fn tab(&self) -> Tab {
        self.tab
    }
    /// The order.
    #[must_use]
    pub fn sort(&self) -> Sort {
        self.sort
    }
    /// Where in the catalog the person is.
    #[must_use]
    pub fn section(&self) -> Section {
        self.section
    }
    /// The search text.
    #[must_use]
    pub fn search(&self) -> &str {
        &self.search
    }
}
