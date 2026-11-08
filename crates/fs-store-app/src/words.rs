//! Pattern: none (sentences as constants, plus the pure `fill` of the core).
//! Every text a person reads in this window comes from here (coding rules 2a). Whole sentences with named
//! placeholders in `{braces}`; translation comes later (ADR 0053). The code never compares these texts.

use fs_store_core::{Kind, SourceError};

use crate::backend::{JobError, Phase};

pub use fs_store_core::words::fill;

/// Title of the window.
pub const TITLE: &str = "FreeSynergy.Store";
/// Name of the sources view.
pub const VIEW_SOURCES: &str = "Sources";
/// Title of the window with a count. Placeholder: `{n}`.
pub const TITLE_COUNT: &str = "FreeSynergy.Store ({n})";
/// Name of the settings page.
pub const VIEW_SETTINGS: &str = "Settings";
/// Name of the About page.
pub const VIEW_ABOUT: &str = "About";
/// Title of the menu with the store's own entries.
pub const MENU_STORE: &str = "Store";
/// Title of the help menu.
pub const MENU_HELP: &str = "Help";
/// Menu entry: quit (with its shortcut).
pub const MENU_QUIT: &str = "Quit (Ctrl+Q)";
/// Menu entry: back to the cards.
pub const MENU_CARDS: &str = "Programs";
/// Button: back to the cards.
pub const BUTTON_BACK: &str = "Back to the programs";
/// Menu entry: sources.
pub const MENU_SOURCES: &str = "Sources";
/// Menu entry: settings.
pub const MENU_SETTINGS: &str = "Settings";
/// Menu entry: about.
pub const MENU_ABOUT: &str = "About";
/// Menu entry that opens the help page of the current view (F1).
pub const MENU_HELP_PAGE: &str = "Help (F1)";

/// About: name and version. Placeholder: `{version}`.
pub const ABOUT_NAME: &str = "FreeSynergy.Store {version}";
/// About: licence.
pub const ABOUT_LICENCE: &str = "Licence: MIT OR Apache-2.0";
/// About: project address. Placeholder: `{url}`.
pub const ABOUT_URL: &str = "Project: {url}";
/// The project address.
pub const PROJECT_URL: &str = "https://github.com/FreeSynergy/fs-store";

/// Button: install a package.
pub const BUTTON_INSTALL: &str = "Install";
/// Button: update a package.
pub const BUTTON_UPDATE: &str = "Update";
/// Button on top: updates every program that has an update. Placeholder `{count}`.
pub const BUTTON_UPDATE_ALL: &str = "Update all ({count})";
/// Button: switch a source on.
pub const BUTTON_SWITCH_ON: &str = "Switch on";
/// Button: switch a source off.
pub const BUTTON_SWITCH_OFF: &str = "Switch off";
/// Button: remove a source.
pub const BUTTON_REMOVE: &str = "Remove";
/// Button: add a source.
pub const BUTTON_ADD: &str = "Add";
/// Button: read the sources again.
pub const BUTTON_REFRESH: &str = "Refresh";

/// Placeholder of the field for a new source.
pub const ADD_PLACEHOLDER: &str =
    "Git address of a source, for example https://example.org/packages";
/// Mark on the built-in source.
pub const MARK_BUILT_IN: &str = "built in";
/// Mark on a source that is switched off.
pub const MARK_OFF: &str = "off";
/// Nothing in the list of sources.
pub const EMPTY_SOURCES: &str = "There are no sources.";
/// Nothing offered.
pub const EMPTY_AVAILABLE: &str = "No source offers a package. Switch on or add a source.";
/// No job has run.
pub const NO_JOB: &str = "Nothing is being installed.";

/// Sentence for a source problem. Placeholder: none.
#[must_use]
pub fn source_problem(error: &SourceError) -> &'static str {
    fs_store_core::words::source_problem(error)
}

/// A call to the store failed. Placeholder: `{detail}`.
pub const STORE_PROBLEM: &str = "The store could not do this: {detail}";
/// A source could not be read. Placeholders: `{source}`, `{detail}`.
pub const SOURCE_UNREADABLE: &str = "The source {source} could not be read: {detail}";
/// An entry of a catalog is not offered. Placeholders: `{id}`, `{reason}`.
pub const ENTRY_SKIPPED: &str = "{id} is not offered: {reason}";
/// The button of a package did nothing. Placeholder: `{name}`.
pub const NOTHING_TO_DO: &str =
    "{name} needs nothing: it is up to date, or no source offers it any more.";
/// "Add" with an empty field.
pub const EMPTY_ADDRESS: &str = "Type the Git address of a source first.";
/// A program could not be started. Placeholders: `{command}`, `{detail}`.
pub const OPEN_FAILED: &str = "{command} could not be opened: {detail}";
/// No command was found to open a program. Placeholder: `{detail}`.
pub const OPEN_NO_COMMAND: &str =
    "Nothing to open was found for {detail}: no command of it exists on this machine.";
/// Button: open a program or facet.
pub const BUTTON_OPEN: &str = "Open";
/// Detail: how it was installed. Placeholder: `{kind}`.
pub const DETAIL_KIND: &str = "Installed as: {kind}";
/// Detail: a command. Placeholders: `{command}`, `{path}`.
pub const DETAIL_COMMAND: &str = "Command {command}: {path}";
/// Detail: the folder. Placeholder: `{folder}`.
pub const DETAIL_FOLDER: &str = "Folder: {folder}";
/// Detail: the source. Placeholder: `{source}`.
pub const DETAIL_SOURCE: &str = "Source: {source}";
/// The worker of a job broke down.
pub const JOB_PANICKED: &str = "the installer broke down unexpectedly";
/// Another job is running. Placeholder: `{name}`.
pub const JOB_BUSY: &str = "{name} is being installed. Wait until it is done.";

/// Name of a way of installing.
#[must_use]
pub fn install_kind(kind: crate::backend::InstallKind) -> &'static str {
    match kind {
        crate::backend::InstallKind::Rpm => "rpm",
        crate::backend::InstallKind::Archive => "tar.gz",
    }
}

/// Name of a kind of package.
#[must_use]
pub fn kind(kind: Kind) -> &'static str {
    match kind {
        Kind::Program => "Program",
        Kind::Frontend => "Frontend",
        Kind::Component => "Component",
        Kind::Theme => "Theme",
    }
}

/// Short name of a phase, for the progress marks.
#[must_use]
pub fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Pending => "Waiting",
        Phase::Verified => "Ready, waiting for the others",
        Phase::Installing => "Installing",
        Phase::Installed => "Installed",
        Phase::Failed => "Failed",
        Phase::RolledBack => "Rolled back",
    }
}

/// Sentence for the job in this phase. Placeholders: `{name}`, `{version}` (the new version, once known).
#[must_use]
pub fn job_phase(phase: Phase) -> &'static str {
    match phase {
        Phase::Pending => "{name}: waiting to start.",
        Phase::Verified => "{name}: signature and checksum are good.",
        Phase::Installing => "{name}: installing.",
        Phase::Installed => "{name} {version} is installed.",
        Phase::Failed => "{name}: something went wrong. Going back.",
        Phase::RolledBack => "{name}: the previous state is back.",
    }
}

/// Sentence for the reason a job failed. Placeholders: `{name}`, `{detail}`.
#[must_use]
pub fn job_problem(error: &JobError) -> &'static str {
    match error {
        JobError::Running => "{name} is running and was not replaced. Close it and try again.",
        JobError::CatalogSignature => {
            "The list of the source could not be trusted: its signature could not be verified."
        }
        JobError::ReleaseSignature => {
            "{name} was not installed: its signature could not be verified."
        }
        JobError::Checksum => {
            "{name} was not installed: the download is not the one that was announced."
        }
        JobError::Downgrade => {
            "{name} was not installed: the source offers an older version than the one you have."
        }
        JobError::NotOffered => "No switched-on source offers {name}.",
        JobError::Unsupported(_) => {
            "{name} comes in a form that this store cannot install yet ({detail})."
        }
        JobError::RollbackFailed => {
            "The update of {name} failed and the previous version could not be restored."
        }
        JobError::Other(_) => "{name} was not installed: {detail}",
    }
}

/// Settings: heading of the look-for-updates choice.
pub const SETTINGS_CHECK: &str = "Look for updates";
/// Choice: never by itself.
pub const CHECK_OFF: &str = "Off";
/// Choice: when the window opens.
pub const CHECK_START: &str = "At start";
/// Choice: once a day.
pub const CHECK_DAILY: &str = "Daily";
/// Choice: every hour.
pub const CHECK_HOURLY: &str = "Hourly";
/// Settings: heading of the install choice.
pub const SETTINGS_INSTALL: &str = "Install updates";
/// Choice: wait for the person.
pub const INSTALL_ASK: &str = "Ask me";
/// Choice: by itself.
pub const INSTALL_AUTO: &str = "Automatically";
/// Hint under the install choice.
pub const INSTALL_AUTO_HINT: &str =
    "Automatically only applies to programs that have automatic updates switched on.";
/// Settings: heading of the notification choice.
pub const SETTINGS_NOTIFY: &str = "Tell me (notification, badge)";
/// Choice: on.
pub const NOTIFY_ON: &str = "On";
/// Choice: off.
pub const NOTIFY_OFF: &str = "Off";
/// The last check. Placeholder: `{time}`.
pub const LAST_CHECK: &str = "Last look for updates: {time}";
/// No check yet.
pub const LAST_CHECK_NEVER: &str = "Last look for updates: never";
/// Button: look for updates now.
pub const BUTTON_LOOK_NOW: &str = "Look now";
/// The check runs.
pub const CHECK_RUNNING: &str = "Looking for updates now. This can take a moment.";
/// The check ended well.
pub const CHECK_DONE: &str = "Done looking for updates.";
/// The check did not work. Placeholder: `{detail}`.
pub const CHECK_FAILED: &str = "Looking for updates did not work: {detail}";
/// Updates page: line of an update. Placeholders: `{name}`, `{from}`, `{to}`.
pub const UPDATE_LINE: &str = "{name}: {from} -> {to}";
/// Line above the cards when updates are known. Placeholder: `{n}`.
pub const UPDATES_BANNER: &str = "{n} update(s) found. Press Update on the cards.";
/// Mark on a program that is new in the available list.
pub const MARK_NEW: &str = "New";

/// Nothing offered and nothing installed.
pub const EMPTY_CARDS: &str = "No programs yet. Switch on or add a source in the Store menu.";
/// Shown when a program has no description.
pub const NO_DESCRIPTION: &str = "There is no description of this program yet.";
/// Entry of the three-dot menu: show the description.
pub const MORE_DESCRIPTION: &str = "Show description";
/// Entry of the three-dot menu: the Git repository.
pub const MORE_REPOSITORY: &str = "Go to the Git repository";
/// Entry of the three-dot menu: uninstall.
pub const MORE_UNINSTALL: &str = "Uninstall";
/// Button: close the description.
pub const BUTTON_CLOSE: &str = "Close";
/// Question before an uninstall. Placeholder: `{name}`.
pub const UNINSTALL_ASK: &str = "Uninstall {name}? Your data stays.";
/// Button: yes, uninstall.
pub const BUTTON_UNINSTALL_YES: &str = "Uninstall";
/// Button: no, keep it.
pub const BUTTON_CANCEL: &str = "Cancel";
/// The program is being taken away.
pub const REMOVING: &str = "Uninstalling...";
/// Uninstall failed. Placeholders: `{name}`, `{detail}`.
pub const UNINSTALL_FAILED: &str = "{name} was not uninstalled: {detail}";
/// The card has no usable project page. Placeholder: `{name}`.
pub const NO_HOMEPAGE: &str = "{name} has no project page (web address) in its catalog entry.";
/// The project page could not be opened. Placeholders: `{url}`, `{detail}`.
pub const REPOSITORY_FAILED: &str = "{url} could not be opened: {detail}";
/// Heading of the installed details in the description panel.
pub const DETAILS_HEADING: &str = "On this machine";
/// Version line of a card that is installed. Placeholder: `{version}`.
pub const VERSION_LINE: &str = "Version {version}";
/// The three dots one above the other (the "more" menu of a card).
pub const MORE_DOTS: &str = "\u{22EE}";

/// Admin page: heading of the daily timer.
pub const TIMER_HEADING: &str = "Look for updates every day, even when the window is closed";
/// The timer is on.
pub const TIMER_IS_ON: &str = "The daily timer is on.";
/// The timer is off.
pub const TIMER_IS_OFF: &str = "The daily timer is off.";
/// Button: switch the timer on.
pub const BUTTON_TIMER_ON: &str = "Switch timer on";
/// Button: switch the timer off.
pub const BUTTON_TIMER_OFF: &str = "Switch timer off";
/// Said when the timer command printed nothing and worked (on).
pub const TIMER_DONE_ON: &str = "The daily timer is switched on.";
/// Said when the timer command printed nothing and worked (off).
pub const TIMER_DONE_OFF: &str = "The daily timer is switched off.";
/// The timer could not be switched. Placeholder: `{detail}`.
pub const TIMER_FAILED: &str = "The daily timer could not be switched: {detail}";
/// Admin page: heading of the per-program switches.
pub const PROGRAMS_HEADING: &str = "Update these programs by themselves";
/// No program is installed yet.
pub const PROGRAMS_NONE: &str = "No program is installed yet.";
/// Admin page: heading of the source list.
pub const SOURCES_HEADING: &str = "Sources";
