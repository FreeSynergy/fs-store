//! Pattern: none (plain data: the views, the button actions, the rows and the problems the window shows).

use fs_store_core::{SourceError, Version};

use crate::backend::{JobError, PackageRow};

/// The views of the window. The cards are the one page; the others come from the menu at the top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The one page: a card for every program.
    Cards,
    /// The list of sources.
    Sources,
    /// What can be set: the check for updates.
    Settings,
    /// Every message of this session.
    Messages,
    /// Name, version, licence and address of the program.
    About,
}

/// What the button of an available package does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Not installed: Install.
    Install,
    /// A newer version is offered: Update.
    Update,
    /// Nothing to do.
    UpToDate,
}

/// A row of the available view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableRow {
    /// The offer.
    pub package: PackageRow,
    /// The installed version, if any.
    pub installed: Option<Version>,
    /// What the button does.
    pub action: Action,
}

/// Something to tell the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A rule of the source list.
    Source(SourceError),
    /// A call failed; developer text.
    Backend(String),
    /// A source could not be read.
    Unreadable {
        /// Source id.
        source: String,
        /// Detail.
        detail: String,
    },
    /// A catalog entry is not offered.
    Skipped {
        /// Entry id.
        id: String,
        /// Reason.
        reason: String,
    },
    /// A job is running already.
    Busy(String),
    /// "Install" was pressed for a package this store version cannot install (shown locked).
    Locked(String),
    /// A job failed: which package and why.
    JobFailed {
        /// Package name.
        name: String,
        /// The reason.
        error: JobError,
    },
    /// The button of a package was pressed but there is nothing to do (up to date, or no longer offered).
    NothingToDo(String),
    /// A program could not be opened: which command and why.
    LaunchFailed {
        /// The command tried (empty when none was found).
        command: String,
        /// The reason, as a developer text.
        detail: String,
    },
    /// "Uninstall" failed: which program and why.
    UninstallFailed {
        /// Package name.
        name: String,
        /// The reason, as a developer text.
        detail: String,
    },
    /// The card has no usable project page.
    NoHomepage(String),
    /// The project page could not be opened.
    RepositoryFailed {
        /// The address.
        url: String,
        /// The reason, as a developer text.
        detail: String,
    },
    /// The daily timer could not be switched; what the process said.
    TimerFailed(String),
    /// "Add" was pressed with an empty field.
    EmptyAddress,
}
