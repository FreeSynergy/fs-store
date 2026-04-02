// wizard/progress.rs — Installation-progress step.
//
// Tracks which phase the install is in and accumulates log messages.
// The FsWidget output is a live-updated list widget.

use fs_render::{FsWidget, ListWidget};

use crate::wizard::done::InstallResult;

// ── InstallPhase ──────────────────────────────────────────────────────────────

/// Current phase of an in-progress (or completed) install operation.
#[derive(Clone, Debug, PartialEq)]
pub enum InstallPhase {
    /// Waiting to start.
    Queued,
    /// Downloading files from the store.
    FetchingFiles,
    /// Writing files to the local filesystem.
    WritingFiles,
    /// Recording the install in the package registry.
    RegisteringPackage,
    /// The install has finished. Carries the outcome.
    Done(InstallResult),
}

impl InstallPhase {
    /// Short human-readable label for the current phase.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::FetchingFiles => "Fetching files…",
            Self::WritingFiles => "Writing files…",
            Self::RegisteringPackage => "Registering package…",
            Self::Done(InstallResult::Success) => "Done — installed successfully",
            Self::Done(InstallResult::Failed(_)) => "Done — installation failed",
        }
    }

    /// Whether the phase represents a completed operation.
    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }

    /// Whether this is a terminal success.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Done(InstallResult::Success))
    }
}

// ── ProgressStep ──────────────────────────────────────────────────────────────

/// State for the progress wizard step.
#[derive(Clone, Debug)]
pub struct ProgressStep {
    /// Name of the package being installed (for display).
    pub package_name: String,
    /// Current install phase.
    pub phase: InstallPhase,
    /// Chronological log of status messages.
    pub log: Vec<String>,
}

impl ProgressStep {
    /// Create a new progress step in the `Queued` phase.
    #[must_use]
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            phase: InstallPhase::Queued,
            log: Vec::new(),
        }
    }

    /// Advance to the next phase and append a log message.
    pub fn advance(&mut self, phase: InstallPhase, message: impl Into<String>) {
        self.phase = phase;
        self.log.push(message.into());
    }

    /// Whether the step is finished (success or failure).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.phase.is_done()
    }

    /// The final `InstallResult`, if available.
    #[must_use]
    pub fn result(&self) -> Option<&InstallResult> {
        if let InstallPhase::Done(ref r) = self.phase {
            Some(r)
        } else {
            None
        }
    }

    /// Progress `ListWidget` showing the current phase + log.
    #[must_use]
    pub fn widget(&self) -> Box<dyn FsWidget> {
        let mut items = vec![
            format!("Installing: {}", self.package_name),
            format!("Status:     {}", self.phase.label()),
            String::new(),
        ];

        if self.log.is_empty() {
            items.push("(waiting…)".into());
        } else {
            for msg in &self.log {
                items.push(format!("  {msg}"));
            }
        }

        // Show error detail at the bottom when failed.
        if let InstallPhase::Done(InstallResult::Failed(ref err)) = self.phase {
            items.push(String::new());
            items.push(format!("Error: {err}"));
        }

        Box::new(ListWidget {
            id: "store-wizard-progress".into(),
            items,
            selected_index: None,
            enabled: false,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_step_is_queued() {
        let step = ProgressStep::new("Test Package");
        assert_eq!(step.phase, InstallPhase::Queued);
        assert!(!step.is_complete());
    }

    #[test]
    fn advance_changes_phase() {
        let mut step = ProgressStep::new("Test");
        step.advance(InstallPhase::FetchingFiles, "Downloading compose.yml");
        assert_eq!(step.phase, InstallPhase::FetchingFiles);
        assert_eq!(step.log.len(), 1);
    }

    #[test]
    fn done_success_is_complete() {
        let mut step = ProgressStep::new("Test");
        step.advance(InstallPhase::Done(InstallResult::Success), "OK");
        assert!(step.is_complete());
        assert!(step.phase.is_success());
        assert!(matches!(step.result(), Some(InstallResult::Success)));
    }

    #[test]
    fn done_failure_is_complete_and_not_success() {
        let mut step = ProgressStep::new("Test");
        step.advance(
            InstallPhase::Done(InstallResult::Failed("disk full".into())),
            "Failed",
        );
        assert!(step.is_complete());
        assert!(!step.phase.is_success());
    }

    #[test]
    fn result_is_none_while_in_progress() {
        let step = ProgressStep::new("Test");
        assert!(step.result().is_none());
    }

    #[test]
    fn widget_id_is_stable() {
        let step = ProgressStep::new("Pkg");
        assert_eq!(step.widget().widget_id(), "store-wizard-progress");
    }

    #[test]
    fn phase_labels_are_nonempty() {
        let phases = [
            InstallPhase::Queued,
            InstallPhase::FetchingFiles,
            InstallPhase::WritingFiles,
            InstallPhase::RegisteringPackage,
            InstallPhase::Done(InstallResult::Success),
            InstallPhase::Done(InstallResult::Failed("x".into())),
        ];
        for p in &phases {
            assert!(!p.label().is_empty());
        }
    }
}
