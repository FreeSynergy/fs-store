//! Pattern: State (enum plus one transition function).
//! `Failed` has exactly one way out: `RolledBack`.

/// Where an installation is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    /// Chosen, not yet checked.
    Pending,
    /// Signature accepted.
    Verified,
    /// Being installed.
    Installing,
    /// Done.
    Installed,
    /// Went wrong; must be rolled back.
    Failed,
    /// Undone; the previous state is back.
    RolledBack,
}

/// What happens to an installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The signature was accepted.
    Verify,
    /// The installer starts.
    Start,
    /// The installer finished.
    Finish,
    /// Something went wrong.
    Fail,
    /// The undo finished.
    Rollback,
}

/// An event that does not fit the current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    /// State the event hit.
    pub from: InstallState,
    /// The event.
    pub event: Event,
}

impl InstallState {
    /// The next state.
    ///
    /// # Errors
    /// `TransitionError` when the event is not allowed here.
    pub fn next(self, event: Event) -> Result<Self, TransitionError> {
        use Event::{Fail, Finish, Rollback, Start, Verify};
        use InstallState::{Failed, Installed, Installing, Pending, RolledBack, Verified};
        match (self, event) {
            (Pending, Verify) => Ok(Verified),
            (Verified, Start) => Ok(Installing),
            (Installing, Finish) => Ok(Installed),
            (Pending | Verified | Installing, Fail) => Ok(Failed),
            (Failed, Rollback) => Ok(RolledBack),
            (from, event) => Err(TransitionError { from, event }),
        }
    }
}
