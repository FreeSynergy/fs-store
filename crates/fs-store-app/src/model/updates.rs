//! Pattern: State (what the last check found, the person's settings, and the check that may be running).

use std::sync::Arc;

use fs_store_sources::{CheckEvery, InstallMode, Novelty, Update};

use crate::updates::UpdateService;

use super::store::StoreModel;
use super::types::Problem;

/// Where the check for updates is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckState {
    /// No check was started in this window.
    Idle,
    /// The check runs now.
    Running,
    /// The last check in this window ended well.
    Done,
    /// The last check in this window did not work; developer text.
    Failed(String),
}

/// True if a check by itself is due: `Start` always (the window just opened), `Daily` and `Hourly` when the
/// last check (`checked`, Unix seconds, 0 = never) is older than the interval, `Off` never.
#[must_use]
pub fn check_due(every: CheckEvery, checked: u64, now: u64) -> bool {
    let interval = match every {
        CheckEvery::Off => return false,
        CheckEvery::Start => return true,
        CheckEvery::Daily => 24 * 60 * 60,
        CheckEvery::Hourly => 60 * 60,
    };
    checked == 0 || now.saturating_sub(checked) >= interval
}

impl StoreModel {
    /// Uses `service` for settings, findings and the check.
    #[must_use]
    pub fn with_updates(mut self, service: Arc<dyn UpdateService>) -> Self {
        self.updates = service;
        self.load_updates();
        self
    }

    /// The service, for a worker thread.
    #[must_use]
    pub fn update_service(&self) -> Arc<dyn UpdateService> {
        Arc::clone(&self.updates)
    }

    /// Reads settings and findings again; a failure is told as a problem.
    pub fn load_updates(&mut self) {
        match self.updates.settings() {
            Ok(settings) => self.settings = settings,
            Err(detail) => self.problems.push(Problem::Backend(detail)),
        }
        match self.updates.findings() {
            Ok(findings) => self.findings = findings,
            Err(detail) => self.problems.push(Problem::Backend(detail)),
        }
        self.load_admin();
    }

    /// The settings.
    #[must_use]
    pub fn settings(&self) -> fs_store_sources::Settings {
        self.settings
    }

    /// Updates the last check found.
    #[must_use]
    pub fn update_list(&self) -> &[Update] {
        &self.findings.update
    }

    /// Programs the last check found that the person has not looked at yet.
    #[must_use]
    pub fn new_programs(&self) -> &[Novelty] {
        &self.findings.new
    }

    /// Unix seconds of the last check; 0 = never.
    #[must_use]
    pub fn last_checked(&self) -> u64 {
        self.findings.checked
    }

    /// The number for the badge: the updates found, or 0 when the person switched "Tell me" off.
    #[must_use]
    pub fn badge(&self) -> usize {
        if self.settings.notify {
            self.findings.update.len()
        } else {
            0
        }
    }

    /// True if `name` is marked "New" in the available list (only while that list is shown).
    #[must_use]
    pub fn is_fresh(&self, name: &str) -> bool {
        self.fresh.iter().any(|n| n == name)
    }

    /// Where the check is.
    #[must_use]
    pub fn check_state(&self) -> &CheckState {
        &self.check
    }

    /// The person opened the available list: new programs are marked there, and not new any more afterwards.
    pub(super) fn enter_available(&mut self) {
        if self.findings.new.is_empty() {
            self.fresh.clear();
            return;
        }
        self.fresh = self.findings.new.iter().map(|n| n.name.clone()).collect();
        self.findings.mark_seen();
        self.save_findings();
    }

    /// An update of `name` was installed: it is no longer an update to do.
    pub(super) fn updated(&mut self, name: &str) {
        let before = self.findings.update.len();
        self.findings.update.retain(|u| u.name != name);
        if self.findings.update.len() != before {
            self.save_findings();
        }
    }

    fn save_findings(&mut self) {
        if let Err(detail) = self.updates.save_findings(&self.findings) {
            self.problems.push(Problem::Backend(detail));
        }
    }

    pub(super) fn save_settings(&mut self) {
        if let Err(detail) = self.updates.save_settings(&self.settings) {
            self.problems.push(Problem::Backend(detail));
        }
    }

    /// The person chose how often to look.
    pub fn set_check_every(&mut self, every: CheckEvery) {
        self.problems.clear();
        self.settings.check = every;
        self.save_settings();
    }

    /// The person chose what happens with updates.
    pub fn set_install_mode(&mut self, mode: InstallMode) {
        self.problems.clear();
        self.settings.install = mode;
        self.save_settings();
    }

    /// The person switched "Tell me" on or off.
    pub fn set_notify(&mut self, notify: bool) {
        self.problems.clear();
        self.settings.notify = notify;
        self.save_settings();
    }

    /// True if the window should look for updates now, once, because it just opened.
    #[must_use]
    pub fn check_wanted_at_start(&self) -> bool {
        check_due(
            self.settings.check,
            self.findings.checked,
            self.updates.now(),
        )
    }

    /// Marks the check as running. `false` (and nothing changes) if it runs already.
    pub fn begin_check(&mut self) -> bool {
        if self.check == CheckState::Running {
            return false;
        }
        self.check = CheckState::Running;
        true
    }

    /// The check ended: reads what it found, and says how it went.
    pub fn finish_check(&mut self, result: Result<(), String>) {
        self.check = match result {
            Ok(()) => CheckState::Done,
            Err(detail) => CheckState::Failed(detail),
        };
        match self.updates.findings() {
            Ok(findings) => {
                // What the person has seen in this list stays marked until the list is left.
                self.findings = findings;
            }
            Err(detail) => self.problems.push(Problem::Backend(detail)),
        }
    }
}
