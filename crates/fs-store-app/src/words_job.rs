//! Pattern: none (the words of a job state and of a job problem; `words` re-exports them).

use crate::backend::JobError;

/// The state of a job in the job list.
#[must_use]
pub fn job_state(state: crate::model::JobState) -> &'static str {
    use crate::model::JobState;
    match state {
        JobState::Waiting => "Waiting",
        JobState::Running => "Running",
        JobState::Done => "Done",
        JobState::Failed => "Failed",
    }
}

/// The installer did not report on a job it was given.
pub const JOB_NO_ANSWER: &str = "the installer gave no answer for it";
/// The refresh button while the catalogs are being read.
pub const REFRESHING: &str = "Reading...";

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
        JobError::Incomplete { .. } => {
            "{name} is installed, but a step after it did not work: {detail}"
        }
        JobError::ServerNotEnded { .. } => {
            "{name} is installed, but its old background server could not be ended: {detail}. End it by hand so that the new version starts."
        }
    }
}
