//! Pattern: none (plain data: the views, the button actions, the rows and the problems the window shows).

use fs_store_core::{SourceError, Version};

use crate::backend::{JobError, PackageRow};

/// The views of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The list of sources.
    Sources,
    /// Packages of the enabled sources.
    Available,
    /// Packages on this machine.
    Installed,
    /// What can be set (nothing yet; says so).
    Settings,
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
    /// "Add" was pressed with an empty field.
    EmptyAddress,
}
