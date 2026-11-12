//! Pattern: none (the words of the storefront: tabs, columns, badges, the package page and the preview). Sentences are
//! constants with named placeholders; `words::fill` puts the values in.

/// Heading of the nav entry for the start page.
pub const NAV_START: &str = "Start";
/// Nav entry: programs.
pub const NAV_PROGRAMS: &str = "Programs";
/// Nav entry: frontends.
pub const NAV_FRONTENDS: &str = "Frontends";
/// Nav entry: components.
pub const NAV_COMPONENTS: &str = "Components";
/// Nav entry: themes.
pub const NAV_THEMES: &str = "Themes";
/// Nav entry: servers (no kind of that name exists in the catalog yet).
pub const NAV_SERVERS: &str = "Servers";
/// Nav entry: icon sets (no kind of that name exists in the catalog yet).
pub const NAV_ICON_SETS: &str = "Icon sets";
/// Note on a nav entry whose kind has no package.
pub const NAV_NONE_YET: &str = "No package of this kind yet";
/// Note on a nav entry the catalog cannot describe yet.
pub const COMING_SOON: &str = "Coming soon";

/// Filter tab: everything.
pub const TAB_ALL: &str = "All";
/// Filter tab: installed.
pub const TAB_INSTALLED: &str = "Installed";
/// Filter tab: updates.
pub const TAB_UPDATES: &str = "Updates";
/// Filter tab: not installed.
pub const TAB_NOT_INSTALLED: &str = "Not installed";

/// Sort choice: by name.
pub const SORT_NAME: &str = "Name";
/// Sort choice: newest release first.
pub const SORT_NEWEST: &str = "Newest";
/// Sort choice: smallest first.
pub const SORT_SIZE: &str = "Smallest";
/// Label before the sort choices.
pub const SORT_LABEL: &str = "Sort";
/// Placeholder of the search field.
pub const SEARCH_PLACEHOLDER: &str = "Search packages";
/// Tooltip: grid view.
pub const VIEW_GRID: &str = "Grid";
/// Tooltip: table view.
pub const VIEW_TABLE: &str = "Table";

/// Column heads of the table.
pub const COL_PACKAGE: &str = "Package";
/// Column: installed version.
pub const COL_INSTALLED: &str = "Installed";
/// Column: latest version.
pub const COL_LATEST: &str = "Latest";
/// Column: release date.
pub const COL_RELEASED: &str = "Released";
/// Column: download size.
pub const COL_SIZE: &str = "Size";
/// Column: signature level.
pub const COL_SIGNATURE: &str = "Signature";
/// Column: capabilities.
pub const COL_CAPABILITIES: &str = "Needs";

/// What is not said by the catalog.
pub const UNKNOWN: &str = "unknown";
/// Nothing to show in a table cell.
pub const NONE: &str = "-";

/// Start page: block of the updates.
pub const START_UPDATES: &str = "Updates";
/// Start page: block of new packages.
pub const START_NEW: &str = "New since the last check";
/// Start page: the recommended block is not possible yet.
pub const START_RECOMMENDED: &str = "Recommended";
/// Why the recommended block is locked.
pub const START_RECOMMENDED_WHY: &str =
    "Coming soon: the catalog has no field for recommended packages yet.";
/// Link of a block on the start page. Placeholder: `{count}`.
pub const SHOW_ALL: &str = "Show all {count}";
/// The list is empty because of a filter or a search.
pub const NOTHING_MATCHES: &str = "Nothing matches. Clear the search or choose another filter.";
/// Footer: where the packages come from. Placeholder: `{sources}`.
pub const FOOTER_SOURCES: &str = "Sources (from the settings): {sources}";
/// Footer: no source switched on.
pub const FOOTER_NO_SOURCE: &str = "No source is switched on. Choose one in the settings.";
/// Footer: number of packages. Placeholder: `{count}`.
pub const FOOTER_COUNT: &str = "{count} packages";

/// Trust level: the maker and the mirror both signed.
pub const TRUST_BOTH: &str = "Maker and mirror";
/// Trust level: only our key.
pub const TRUST_OURS: &str = "Our key only";
/// Trust level: only a checksum.
pub const TRUST_SUM: &str = "Checksum only";
/// What the level means: both.
pub const TRUST_BOTH_TEXT: &str =
    "The maker's own signature was checked, then the package was signed with our key.";
/// What the level means: ours.
pub const TRUST_OURS_TEXT: &str = "Signed with our key. The maker gives no signature, so only the checksum of the maker is known.";
/// What the level means: checksum.
pub const TRUST_SUM_TEXT: &str =
    "Only a checksum, no signature. Such a package is never installed from the reference source.";
/// What the level means: nothing said.
pub const TRUST_UNKNOWN_TEXT: &str = "The catalog does not say how this package was checked.";

/// Needs state: nothing needed.
pub const NEEDS_NONE: &str = "none";
/// Needs state: all met.
pub const NEEDS_MET: &str = "met";
/// Needs state: a choice of providers.
pub const NEEDS_CHOICE: &str = "choose";
/// Needs state: something missing.
pub const NEEDS_MISSING: &str = "missing";

/// Package page: back.
pub const PAGE_BACK: &str = "Back";
/// Package page: section heads.
pub const PAGE_VERSION: &str = "Version";
/// Head: size.
pub const PAGE_SIZE: &str = "Size";
/// Head: origin.
pub const PAGE_ORIGIN: &str = "Origin";
/// Head: signature.
pub const PAGE_SIGNATURE: &str = "Signature";
/// Head: provides.
pub const PAGE_PROVIDES: &str = "Offers";
/// Head: requires.
pub const PAGE_REQUIRES: &str = "Needs";
/// Head: recommends.
pub const PAGE_RECOMMENDS: &str = "Can use";
/// Head: pictures.
pub const PAGE_PICTURES: &str = "Pictures";
/// Head: changes.
pub const PAGE_CHANGES: &str = "Changes";
/// Head: description.
pub const PAGE_DESCRIPTION: &str = "Description";
/// Nothing offered or needed.
pub const PAGE_NOTHING: &str = "nothing";
/// Capability met by a package. Placeholder: `{by}`.
pub const CAP_MET_BY: &str = "provided by {by}";
/// Capability with several providers. Placeholder: `{by}`.
pub const CAP_CHOICE: &str = "choose one of {by}";
/// Capability nobody offers.
pub const CAP_MISSING: &str = "nobody offers it";
/// Origin: maker.
pub const ORIGIN_MAKER: &str = "Maker";
/// Origin: source of the catalog.
pub const ORIGIN_SOURCE: &str = "Source";
/// Origin: maker signature.
pub const ORIGIN_MAKER_SIG: &str = "Maker's signature";
/// Origin: source code.
pub const ORIGIN_CODE: &str = "Source code of this version";
/// Origin: a package of ours, not mirrored.
pub const ORIGIN_OURS: &str = "made by FreeSynergy";
/// Pictures: how many are shown. Placeholder: `{count}`.
pub const PICTURES_SHOWN: &str =
    "{count} with a clear licence (loading them comes with the picture mirror)";
/// Pictures: none.
pub const PICTURES_NONE: &str = "No pictures from the maker.";
/// Pictures: how many are withheld. Placeholders: `{count}`.
pub const PICTURES_WITHHELD: &str =
    "{count} not shown: no clear licence, no hash or no https address";
/// The description is a block of the wiki that cannot be loaded yet.
pub const DESCRIPTION_WIKI: &str =
    "The description lives in the wiki (block {id}). Loading it comes later.";
/// No release has a text.
pub const CHANGES_NONE: &str = "The catalog lists no changes.";
/// Icon missing.
pub const ICON_MISSING: &str = "The catalog has no icon for this package.";

/// Preview: heading. Placeholder: `{title}`.
pub const PREVIEW_INSTALL: &str = "Install {title}: what will happen";
/// Preview: heading of an update.
pub const PREVIEW_UPDATE: &str = "Update {title}: what will happen";
/// Preview: heading of an uninstall.
pub const PREVIEW_REMOVE: &str = "Remove {title}: what will happen";
/// Preview: several packages. Placeholder: `{count}`.
pub const PREVIEW_MANY: &str = "Update {count} programs: what will happen to each";
/// Preview: highest risk. Placeholder: `{risk}`.
pub const PREVIEW_RISK: &str = "Highest risk: {risk}";
/// Preview: who acts. Words.
pub const ACTOR_PROGRAM: &str = "the store";
/// Actor: the system.
pub const ACTOR_MACHINE: &str = "the system";
/// Actor: you.
pub const ACTOR_PERSON: &str = "you";
/// Risk words, lowest first.
pub const RISK_LOOK: &str = "look";
/// Risk: open.
pub const RISK_OPEN: &str = "open";
/// Risk: start.
pub const RISK_START: &str = "start";
/// Risk: install.
pub const RISK_INSTALL: &str = "install";
/// Risk: system-wide.
pub const RISK_SYSTEM: &str = "system-wide";
/// Confirm install.
pub const BUTTON_DO_INSTALL: &str = "Install";
/// Confirm update.
pub const BUTTON_DO_UPDATE: &str = "Update";
/// Confirm removal.
pub const BUTTON_DO_REMOVE: &str = "Remove";
/// Cancel the preview.
pub const BUTTON_NOT_NOW: &str = "Not now";
/// Open the project page.
pub const BUTTON_WEBSITE: &str = "Website";

/// Footer: when the last look for updates was. Placeholder: `{ago}`.
pub const FOOTER_CHECKED: &str = "Checked {ago}";
/// Footer: never looked.
pub const FOOTER_NEVER_CHECKED: &str = "Never checked";
/// Footer: what the store installs. Placeholder: `{format}`.
pub const FOOTER_FORMAT: &str = "Installing as {format}";
/// Number of packages in a section. Placeholder: `{count}`.
pub const SECTION_COUNT: &str = "{count} packages";
/// Tile link: open the project page.
pub const LINK_WEBSITE: &str = "Website";
/// Tile link: open the page of the package.
pub const LINK_DETAILS: &str = "Details";
/// Locked pause button of a running job.
pub const PAUSE_LOCKED: &str = "Pause: coming soon";
/// Locked cancel button of a running job.
pub const CANCEL_LOCKED: &str = "Cancel: coming soon";
/// Button: update to a version. Placeholder: `{version}`.
pub const BUTTON_UPDATE_TO: &str = "Update to {version}";

/// Title of the section that lists everything (a section, a search or a filter shows it).
pub const SECTION_ALL: &str = "Packages";
