// wizard/select.rs — Package-selection step.
//
// Holds the list of browsable packages and the user's current selection.
// Rendering is engine-agnostic via FsWidget.

use fs_render::{FsWidget, ListWidget};
use fs_store::installer::InstallKind;

// ── SelectablePackage ─────────────────────────────────────────────────────────

/// One row in the selection list.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectablePackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: InstallKind,
    pub summary: String,
}

impl SelectablePackage {
    /// Short kind label for display in the list row.
    #[must_use]
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            InstallKind::Bundle => "bundle",
            InstallKind::Container => "container",
            InstallKind::Language => "language",
            InstallKind::Theme => "theme",
            InstallKind::App => "app",
            InstallKind::Other => "other",
        }
    }
}

// ── SelectStep ────────────────────────────────────────────────────────────────

/// State for the package-selection wizard step.
#[derive(Clone, Debug, Default)]
pub struct SelectStep {
    /// All packages available for installation.
    pub packages: Vec<SelectablePackage>,
    /// Index into the *visible* (filtered) list.
    pub selected_index: Option<usize>,
    /// Case-insensitive substring filter applied to name and summary.
    pub filter: String,
}

impl SelectStep {
    /// Create a new step with the given package list.
    #[must_use]
    pub fn new(packages: Vec<SelectablePackage>) -> Self {
        Self {
            packages,
            selected_index: None,
            filter: String::new(),
        }
    }

    /// Packages that match the current filter.
    #[must_use]
    pub fn visible(&self) -> Vec<&SelectablePackage> {
        if self.filter.is_empty() {
            return self.packages.iter().collect();
        }
        let q = self.filter.to_lowercase();
        self.packages
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&q) || p.summary.to_lowercase().contains(&q))
            .collect()
    }

    /// The currently selected package, if any.
    #[must_use]
    pub fn selected(&self) -> Option<&SelectablePackage> {
        self.selected_index
            .and_then(|i| self.visible().into_iter().nth(i))
    }

    /// Whether the step is complete — i.e. a package has been selected.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.selected_index.is_some()
    }

    /// List widget for the selection step.
    #[must_use]
    pub fn widget(&self) -> Box<dyn FsWidget> {
        let items: Vec<String> = self
            .visible()
            .iter()
            .map(|p| {
                format!(
                    "[{}]  {}  v{}  — {}",
                    p.kind_label(),
                    p.name,
                    p.version,
                    p.summary
                )
            })
            .collect();

        Box::new(ListWidget {
            id: "store-wizard-select".into(),
            items,
            selected_index: self.selected_index,
            enabled: true,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(id: &str, name: &str, kind: InstallKind) -> SelectablePackage {
        SelectablePackage {
            id: id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            kind,
            summary: format!("{name} package"),
        }
    }

    #[test]
    fn empty_filter_shows_all() {
        let step = SelectStep::new(vec![
            pkg("a", "Alpha", InstallKind::App),
            pkg("b", "Beta", InstallKind::Theme),
        ]);
        assert_eq!(step.visible().len(), 2);
    }

    #[test]
    fn filter_narrows_visible_list() {
        let step = SelectStep {
            packages: vec![
                pkg("a", "Alpha", InstallKind::App),
                pkg("b", "Beta", InstallKind::Theme),
            ],
            filter: "alp".into(),
            selected_index: None,
        };
        assert_eq!(step.visible().len(), 1);
        assert_eq!(step.visible()[0].id, "a");
    }

    #[test]
    fn no_selection_is_incomplete() {
        let step = SelectStep::new(vec![pkg("a", "Alpha", InstallKind::App)]);
        assert!(!step.is_complete());
    }

    #[test]
    fn selection_is_complete() {
        let step = SelectStep {
            packages: vec![pkg("a", "Alpha", InstallKind::App)],
            selected_index: Some(0),
            filter: String::new(),
        };
        assert!(step.is_complete());
        assert_eq!(step.selected().unwrap().id, "a");
    }

    #[test]
    fn widget_produces_list() {
        let step = SelectStep::new(vec![pkg("a", "Alpha", InstallKind::App)]);
        let w = step.widget();
        assert_eq!(w.widget_id(), "store-wizard-select");
    }
}
