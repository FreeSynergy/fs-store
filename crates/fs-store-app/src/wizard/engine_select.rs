// wizard/engine_select.rs — Render-Engine selection step.
//
// Design Pattern: State Machine step (same as other wizard steps).
//
// Used during bundle install when `bundle.render_engines` is declared.
// The user selects which GUI engine to install:
//   - iced (default, Wayland/X11)
//   - bevy (3D-capable, experimental)
//   - tui  (terminal only)
//   - none (headless / API only)
//
// The step is skipped when the bundle has no engine choices.

use fs_render::{FsWidget, ListWidget};
use fs_store::installer::InstallRequest;

// ── RenderEngineOption ────────────────────────────────────────────────────────

/// One available render engine choice.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderEngineOption {
    /// Engine id, e.g. `"gui-engine-iced"`.
    pub id: String,
    /// Display label, e.g. `"iced (recommended — Wayland/X11)"`.
    pub label: String,
    /// Whether this option is the recommended default.
    pub is_default: bool,
}

impl RenderEngineOption {
    /// Build an `InstallRequest` for this engine option.
    #[must_use]
    pub fn to_install_request(&self, version: &str) -> InstallRequest {
        InstallRequest {
            id: self.id.clone(),
            name: self.label.clone(),
            kind: fs_store::InstallKind::App,
            version: version.to_owned(),
            store_path: Some(format!("packages/apps/{}", self.id)),
            capabilities: vec![format!("render-engine.{}", self.id)],
            icon: None,
        }
    }
}

// ── EngineSelectStep ──────────────────────────────────────────────────────────

/// State for the render-engine selection wizard step.
///
/// Populated from the bundle's `[[bundle.render_engines]]` entries.
/// `selected_index` starts at the default option, if any.
#[derive(Clone, Debug)]
pub struct EngineSelectStep {
    /// Available engine options.
    pub options: Vec<RenderEngineOption>,
    /// Index of the currently selected option.
    pub selected_index: Option<usize>,
}

impl EngineSelectStep {
    /// Create a new step from a list of engine options.
    ///
    /// Pre-selects the first option marked `is_default = true`.
    #[must_use]
    pub fn new(options: Vec<RenderEngineOption>) -> Self {
        let selected_index = options.iter().position(|o| o.is_default);
        Self {
            options,
            selected_index,
        }
    }

    /// Built-in default options — used when no bundle catalog is available.
    #[must_use]
    pub fn defaults() -> Self {
        Self::new(vec![
            RenderEngineOption {
                id: "gui-engine-iced".into(),
                label: "iced (recommended — Wayland/X11)".into(),
                is_default: true,
            },
            RenderEngineOption {
                id: "gui-engine-bevy".into(),
                label: "Bevy (3D-capable, experimental)".into(),
                is_default: false,
            },
            RenderEngineOption {
                id: "tui-engine".into(),
                label: "TUI (terminal only — no display server required)".into(),
                is_default: false,
            },
            RenderEngineOption {
                id: "none".into(),
                label: "None (headless / API only)".into(),
                is_default: false,
            },
        ])
    }

    /// The currently selected option, if any.
    #[must_use]
    pub fn selected(&self) -> Option<&RenderEngineOption> {
        self.selected_index.and_then(|i| self.options.get(i))
    }

    /// Whether the step is complete — i.e. an engine has been selected.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.selected_index.is_some()
    }

    /// Whether this step should be shown — i.e. there are choices to make.
    #[must_use]
    pub fn is_applicable(&self) -> bool {
        !self.options.is_empty()
    }

    /// Select engine by index. No-op if out of bounds.
    pub fn select(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
    }

    /// `ListWidget` for the engine selection step.
    #[must_use]
    pub fn widget(&self) -> Box<dyn FsWidget> {
        let items: Vec<String> = self
            .options
            .iter()
            .map(|o| {
                if o.is_default {
                    format!("{}  [default]", o.label)
                } else {
                    o.label.clone()
                }
            })
            .collect();

        Box::new(ListWidget {
            id: "store-wizard-engine-select".into(),
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

    fn iced() -> RenderEngineOption {
        RenderEngineOption {
            id: "gui-engine-iced".into(),
            label: "iced".into(),
            is_default: true,
        }
    }

    fn bevy() -> RenderEngineOption {
        RenderEngineOption {
            id: "gui-engine-bevy".into(),
            label: "Bevy".into(),
            is_default: false,
        }
    }

    #[test]
    fn default_option_pre_selected() {
        let step = EngineSelectStep::new(vec![bevy(), iced()]);
        // iced is at index 1 and is_default=true
        assert_eq!(step.selected_index, Some(1));
        assert_eq!(step.selected().unwrap().id, "gui-engine-iced");
    }

    #[test]
    fn no_default_means_no_preselection() {
        let step = EngineSelectStep::new(vec![bevy()]);
        assert_eq!(step.selected_index, None);
        assert!(!step.is_complete());
    }

    #[test]
    fn select_changes_selection() {
        let mut step = EngineSelectStep::new(vec![iced(), bevy()]);
        step.select(1);
        assert_eq!(step.selected().unwrap().id, "gui-engine-bevy");
        assert!(step.is_complete());
    }

    #[test]
    fn select_out_of_bounds_is_noop() {
        let mut step = EngineSelectStep::new(vec![iced()]);
        step.select(99);
        assert_eq!(step.selected_index, Some(0)); // iced was pre-selected
    }

    #[test]
    fn defaults_has_four_options() {
        let step = EngineSelectStep::defaults();
        assert_eq!(step.options.len(), 4);
        assert!(step.is_applicable());
    }

    #[test]
    fn widget_produces_list() {
        let step = EngineSelectStep::defaults();
        let w = step.widget();
        assert_eq!(w.widget_id(), "store-wizard-engine-select");
    }

    #[test]
    fn to_install_request_sets_kind() {
        let opt = iced();
        let req = opt.to_install_request("0.13.0");
        assert_eq!(req.id, "gui-engine-iced");
        assert!(matches!(req.kind, fs_store::InstallKind::App));
        assert_eq!(req.version, "0.13.0");
    }
}
