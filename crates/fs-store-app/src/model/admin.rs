//! Pattern: Command + State (the admin page beyond the update choices: the daily timer, the switch for
//! automatic updates of every program). The service does the work; the model keeps what the page shows.

use crate::updates::Program;
use crate::words;

use super::store::StoreModel;
use super::types::Problem;

/// What the admin page shows besides the settings.
#[derive(Debug, Clone, Default)]
pub struct AdminState {
    /// The daily update timer is on.
    pub(super) timer_on: bool,
    /// The programs of the register.
    pub(super) programs: Vec<Program>,
    /// What the last timer command printed; gone when the view changes.
    pub(super) note: Option<String>,
}

impl StoreModel {
    /// Reads the timer state and the register again; a failure is told as a problem.
    pub(super) fn load_admin(&mut self) {
        self.admin.timer_on = self.updates.timer_enabled();
        match self.updates.programs() {
            Ok(list) => self.admin.programs = list,
            Err(detail) => self.problems.push(Problem::Backend(detail)),
        }
    }

    /// True if the daily update timer is on.
    #[must_use]
    pub fn timer_on(&self) -> bool {
        self.admin.timer_on
    }

    /// The programs of the register with their switch for automatic updates, by name.
    #[must_use]
    pub fn programs(&self) -> &[Program] {
        &self.admin.programs
    }

    /// What the last timer command printed (a success; it goes when the view changes).
    #[must_use]
    pub fn admin_note(&self) -> Option<&str> {
        self.admin.note.as_deref()
    }

    /// The person switched the daily timer on or off. The output is kept as a note, a failure is a problem.
    pub fn set_timer(&mut self, on: bool) {
        self.problems.clear();
        self.admin.note = None;
        match self.updates.timer_set(on) {
            Ok(out) if out.is_empty() => {
                let done = if on {
                    words::TIMER_DONE_ON
                } else {
                    words::TIMER_DONE_OFF
                };
                self.admin.note = Some(done.to_owned());
            }
            Ok(out) => self.admin.note = Some(out),
            Err(detail) => self.problems.push(Problem::TimerFailed(detail)),
        }
        self.admin.timer_on = self.updates.timer_enabled();
    }

    /// The person switched automatic updates of `name` on or off; saved in the register.
    pub fn set_program_auto(&mut self, name: &str, on: bool) {
        self.problems.clear();
        if let Err(detail) = self.updates.set_auto_update(name, on) {
            self.problems.push(Problem::Backend(detail));
        }
        match self.updates.programs() {
            Ok(list) => self.admin.programs = list,
            Err(detail) => self.problems.push(Problem::Backend(detail)),
        }
    }
}
