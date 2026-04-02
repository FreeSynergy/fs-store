// wizard/confirm.rs — Confirmation step.
//
// Shows the package that will be installed and, for containers, collects
// environment variable values via TextInputWidgets.

use fs_render::{FsWidget, ListWidget, TextInputWidget};
use fs_store::installer::{InstallKind, InstallRequest};

// ── EnvVarField ───────────────────────────────────────────────────────────────

/// One environment variable input field for a container package.
#[derive(Clone, Debug)]
pub struct EnvVarField {
    /// Variable name (e.g. `DB_HOST`).
    pub key: String,
    /// Current user-entered value.
    pub value: String,
}

impl EnvVarField {
    /// Build the `KEY=value` string for the `.env` file.
    #[must_use]
    pub fn to_env_line(&self) -> String {
        format!("{}={}", self.key, self.value)
    }

    /// `TextInputWidget` for this field (one per variable).
    #[must_use]
    pub fn widget(&self) -> TextInputWidget {
        TextInputWidget {
            id: format!("store-wizard-env-{}", self.key.to_lowercase()),
            placeholder: self.key.clone(),
            value: self.value.clone(),
            enabled: true,
        }
    }
}

// ── ConfirmStep ───────────────────────────────────────────────────────────────

/// State for the confirmation wizard step.
#[derive(Clone, Debug)]
pub struct ConfirmStep {
    /// What will be installed.
    pub request: InstallRequest,
    /// Env var fields — populated for `InstallKind::Container`, empty otherwise.
    pub env_fields: Vec<EnvVarField>,
    /// Whether the user has confirmed the installation.
    pub confirmed: bool,
}

impl ConfirmStep {
    /// Create a new confirm step.
    ///
    /// `env_keys` is the list of variable names extracted from the compose file
    /// (use `fs_store::installer::fetch_container_env_vars` to obtain them).
    #[must_use]
    pub fn new(request: InstallRequest, env_keys: Vec<String>) -> Self {
        let env_fields = env_keys
            .into_iter()
            .map(|key| EnvVarField {
                key,
                value: String::new(),
            })
            .collect();
        Self {
            request,
            env_fields,
            confirmed: false,
        }
    }

    /// Build the `.env` file content from all filled-in fields.
    #[must_use]
    pub fn env_content(&self) -> String {
        self.env_fields
            .iter()
            .map(EnvVarField::to_env_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Whether there are environment variable fields to fill.
    #[must_use]
    pub fn has_env_fields(&self) -> bool {
        !self.env_fields.is_empty()
    }

    /// Summary `ListWidget` showing the package details + env var count.
    #[must_use]
    pub fn widget(&self) -> Box<dyn FsWidget> {
        let req = &self.request;
        let kind_label = match req.kind {
            InstallKind::Bundle => "Bundle",
            InstallKind::Container => "Container",
            InstallKind::Language => "Language pack",
            InstallKind::Theme => "Theme",
            InstallKind::App => "Application",
            InstallKind::Other => "Package",
        };

        let mut items = vec![
            format!("Name:    {}", req.name),
            format!("ID:      {}", req.id),
            format!("Version: {}", req.version),
            format!("Type:    {kind_label}"),
        ];

        if let Some(ref path) = req.store_path {
            items.push(format!("Path:    {path}"));
        }

        if !self.env_fields.is_empty() {
            items.push(String::new());
            items.push(format!("Env vars to configure: {}", self.env_fields.len()));
            for f in &self.env_fields {
                let display = if f.value.is_empty() {
                    format!("  {}  (not set)", f.key)
                } else {
                    format!("  {}  ✓", f.key)
                };
                items.push(display);
            }
        }

        Box::new(ListWidget {
            id: "store-wizard-confirm".into(),
            items,
            selected_index: None,
            enabled: true,
        })
    }

    /// One `TextInputWidget` per environment variable field.
    #[must_use]
    pub fn env_input_widgets(&self) -> Vec<TextInputWidget> {
        self.env_fields.iter().map(EnvVarField::widget).collect()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn req(kind: InstallKind) -> InstallRequest {
        InstallRequest {
            id: "test-pkg".into(),
            name: "Test Package".into(),
            kind,
            version: "1.2.3".into(),
            store_path: None,
            capabilities: vec![],
            icon: None,
        }
    }

    #[test]
    fn no_env_keys_for_non_container() {
        let step = ConfirmStep::new(req(InstallKind::App), vec![]);
        assert!(!step.has_env_fields());
        assert!(step.env_content().is_empty());
    }

    #[test]
    fn env_keys_create_fields() {
        let step = ConfirmStep::new(
            req(InstallKind::Container),
            vec!["DB_HOST".into(), "DB_PORT".into()],
        );
        assert_eq!(step.env_fields.len(), 2);
        assert!(step.has_env_fields());
    }

    #[test]
    fn env_content_formats_correctly() {
        let mut step = ConfirmStep::new(
            req(InstallKind::Container),
            vec!["HOST".into(), "PORT".into()],
        );
        step.env_fields[0].value = "localhost".into();
        step.env_fields[1].value = "5432".into();
        let content = step.env_content();
        assert_eq!(content, "HOST=localhost\nPORT=5432");
    }

    #[test]
    fn widget_produces_confirm_list() {
        let step = ConfirmStep::new(req(InstallKind::Theme), vec![]);
        let w = step.widget();
        assert_eq!(w.widget_id(), "store-wizard-confirm");
    }

    #[test]
    fn env_input_widgets_count_matches_fields() {
        let step = ConfirmStep::new(
            req(InstallKind::Container),
            vec!["A".into(), "B".into(), "C".into()],
        );
        assert_eq!(step.env_input_widgets().len(), 3);
    }
}
