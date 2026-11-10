//! Pattern: Strategy (what runs the check for updates by itself, and how the person is told: a systemd user timer and
//! desktop notifications on Linux, a scheduled task and a balloon on Windows).

use std::path::Path;

/// How often the timer runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Every {
    /// Once an hour.
    Hourly,
    /// Once a day.
    Daily,
}

/// The timer of the background check, and the tell-the-person part of it.
pub trait Background: Send + Sync {
    /// Makes `<exe> background` run every `every`, for this person, starting at login or soon after.
    ///
    /// # Errors
    /// A sentence when the system refused.
    fn timer_enable(&self, exe: &Path, every: Every) -> Result<String, String>;
    /// Stops the timer and takes it away.
    ///
    /// # Errors
    /// A sentence when the system refused.
    fn timer_disable(&self) -> Result<String, String>;
    /// Says what the system knows about the timer.
    ///
    /// # Errors
    /// A sentence when the system refused.
    fn timer_status(&self) -> Result<String, String>;
    /// True when the timer is switched on.
    fn timer_enabled(&self) -> bool;
    /// Tells the person something, best effort.
    fn notify(&self, text: &str);
    /// Shows how many things wait (a badge on the launcher), best effort; a system without one does nothing.
    fn badge(&self, count: usize);
}
