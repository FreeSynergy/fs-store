//! Pattern: none (sentences as constants, plus the pure `fill` of the core).
//! Every text a person reads in this window comes from here (coding rules 2a). Whole sentences with named
//! placeholders in `{braces}`; translation comes later (ADR 0053). The code never compares these texts.

use fs_store_core::{Kind, SourceError};

use crate::backend::{JobError, Phase};

pub use fs_store_core::words::fill;

/// Title of the window.
pub const TITLE: &str = "FreeSynergy Store";
/// Name of the sources view.
pub const VIEW_SOURCES: &str = "Sources";
/// Name of the available view.
pub const VIEW_AVAILABLE: &str = "Available";
/// Name of the installed view.
pub const VIEW_INSTALLED: &str = "Installed";

/// Button: install a package.
pub const BUTTON_INSTALL: &str = "Install";
/// Button: update a package.
pub const BUTTON_UPDATE: &str = "Update";
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
/// Shown instead of a button when the installed version is the newest. Placeholder: `{version}`.
pub const UP_TO_DATE: &str = "Up to date ({version})";
/// Shown with a package that has a newer version. Placeholders: `{offered}`, `{installed}`.
pub const NEWER_OFFERED: &str = "{offered} (installed: {installed})";

/// Nothing in the list of sources.
pub const EMPTY_SOURCES: &str = "There are no sources.";
/// Nothing offered.
pub const EMPTY_AVAILABLE: &str = "No source offers a package. Switch on or add a source.";
/// Nothing installed.
pub const EMPTY_INSTALLED: &str = "Nothing is installed yet.";
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
/// Another job is running. Placeholder: `{name}`.
pub const JOB_BUSY: &str = "{name} is being installed. Wait until it is done.";

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
        Phase::Verified => "Checked",
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
        JobError::Unsupported => "{name} comes in a form that this store cannot install yet.",
        JobError::RollbackFailed => {
            "The update of {name} failed and the previous version could not be restored."
        }
        JobError::Other(_) => "{name} was not installed: {detail}",
    }
}
