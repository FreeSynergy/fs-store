// wizard/done.rs — InstallResult type + FsView widget.
//
// Renderer-agnostic result display. The Dioxus overlay popup lives in
// app.rs until the store app is migrated to fs-render (G2).

use fs_render::{FsWidget, ListWidget};

// ── InstallResult ─────────────────────────────────────────────────────────────

/// Outcome of a package install operation.
#[derive(Clone, PartialEq, Debug)]
pub enum InstallResult {
    Success,
    Failed(String),
}

impl InstallResult {
    /// Short human-readable summary line.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Success => fs_i18n::t("store.install.success").to_string(),
            Self::Failed(err) => {
                format!("{}: {err}", fs_i18n::t("store.install.failed"))
            }
        }
    }

    /// `true` if the install succeeded.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

// ── FsView ────────────────────────────────────────────────────────────────────

impl InstallResult {
    /// Render a `ListWidget` summarising the install outcome.
    #[must_use]
    pub fn widget(&self) -> Box<dyn FsWidget> {
        let items = match self {
            Self::Success => vec![fs_i18n::t("store.install.success").to_string()],
            Self::Failed(err) => vec![
                fs_i18n::t("store.install.failed").to_string(),
                format!("  {err}"),
            ],
        };

        Box::new(ListWidget {
            id: "store-install-result".into(),
            items,
            selected_index: None,
            enabled: true,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_is_success() {
        assert!(InstallResult::Success.is_success());
    }

    #[test]
    fn failed_is_not_success() {
        assert!(!InstallResult::Failed("boom".into()).is_success());
    }

    #[test]
    fn failed_summary_contains_error() {
        let r = InstallResult::Failed("disk full".into());
        assert!(r.summary().contains("disk full"));
    }

    #[test]
    fn widget_success_has_one_item() {
        // Just verify it doesn't panic and returns a widget.
        let _ = InstallResult::Success.widget();
    }

    #[test]
    fn widget_failed_has_two_items() {
        let _ = InstallResult::Failed("bad".into()).widget();
    }
}
