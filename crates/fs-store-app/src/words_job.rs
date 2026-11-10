//! Pattern: none (the four words of a job state; `words` re-exports them).

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
