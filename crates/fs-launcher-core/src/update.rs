//! Pattern: Command (`apply` / `invert`) driving the State; Strategy for the running check.
//! A running program is never replaced; a failed update is undone.

use crate::package::Version;
use crate::signature::VerifiedPackage;
use crate::state::InstallState;
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
    let _ = (update, strategy, running);
    todo!()
}
