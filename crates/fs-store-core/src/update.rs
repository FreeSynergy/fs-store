//! Pattern: State driven by one fixed sequence (verify, install, on failure restore); Strategy for the running check.
//! The undo is `InstallStrategy::restore`; there is no `Command` trait yet.
//! A running program is never replaced; a failed update is undone.

use crate::package::Version;
use crate::signature::VerifiedPackage;
use crate::state::{Event, InstallState};
use crate::strategy::InstallStrategy;

/// Asks the system whether a program is running.
pub trait RunningCheck {
    /// True while the named program runs.
    fn is_running(&self, name: &str) -> bool;
}

/// Why an update did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    /// The program runs and is not replaced.
    Running,
    /// The installer failed; the update was rolled back.
    RolledBack,
    /// The installer failed and the rollback failed too.
    RollbackFailed,
}

/// An update that can be undone.
#[derive(Debug)]
pub struct Update<'a> {
    /// What to install.
    pub package: &'a VerifiedPackage,
    /// What was there before (`None` for a first install).
    pub previous: Option<Version>,
}

/// Runs the update and returns the final state.
///
/// # Errors
/// `UpdateError` when the program runs or the update failed.
pub fn run_update(
    update: &Update<'_>,
    strategy: &mut dyn InstallStrategy,
    running: &dyn RunningCheck,
) -> Result<InstallState, UpdateError> {
    let name = update.package.package().name.as_str();
    if running.is_running(name) {
        return Err(UpdateError::Running);
    }
    // The sequence below is fixed and always valid, so the fallback never applies.
    let step = |s: InstallState, e| s.next(e).unwrap_or(s);
    let state = step(InstallState::Pending, Event::Verify);
    let state = step(state, Event::Start);
    if strategy.install(update.package).is_ok() {
        return Ok(step(state, Event::Finish));
    }
    let failed = step(state, Event::Fail);
    match strategy.restore(name, update.previous) {
        Ok(()) => {
            debug_assert_eq!(step(failed, Event::Rollback), InstallState::RolledBack);
            Err(UpdateError::RolledBack)
        }
        Err(_) => Err(UpdateError::RollbackFailed),
    }
}
