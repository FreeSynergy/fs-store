// container.rs — ContainerPackage: server-side Podman/Quadlet services.

use serde::{Deserialize, Serialize};

use crate::category::{ContainerCategory, PackageCategory};
use crate::package::mod_prelude::*;

// ── Domain types ──────────────────────────────────────────────────────────────

/// An exposed service port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    /// Port number (1–65535).
    pub port: u16,

    /// Protocol, e.g. `"tcp"`, `"udp"`.
    #[serde(default = "default_tcp")]
    pub protocol: String,

    /// Optional human-readable label, e.g. `"HTTP"`, `"HTTPS"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn default_tcp() -> String {
    "tcp".to_owned()
}

/// A configurable variable the user must or can set before deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Environment variable name, e.g. `"FORGEJO_SECRET_KEY"`.
    pub name: String,

    /// i18n key for the field label.
    pub label_key: String,

    /// Whether this variable must be set.
    #[serde(default)]
    pub required: bool,

    /// Whether the value is a secret (masked in the UI).
    #[serde(default)]
    pub secret: bool,
}

/// An optional feature toggle for a container service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Feature slug, e.g. `"ldap"`, `"webauthn"`.
    pub id: String,

    /// i18n key for the feature label.
    pub label_key: String,

    /// Whether this feature is enabled by default.
    #[serde(default)]
    pub default: bool,
}

// ── ContainerPackage ──────────────────────────────────────────────────────────

/// A server-side container service installable via Podman/Quadlet.
///
/// Extends [`Package`] with ports, deployment variables, and feature toggles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPackage {
    #[serde(flatten)]
    pub(crate) data: PackageData,

    /// Ports exposed by this service.
    #[serde(default)]
    pub ports: Vec<ServicePort>,

    /// User-configurable deployment variables.
    #[serde(default)]
    pub variables: Vec<Variable>,

    /// Optional feature flags the user can enable or disable.
    #[serde(default)]
    pub features: Vec<Feature>,
}

impl Package for ContainerPackage {
    fn id(&self) -> &str {
        &self.data.id
    }
    fn name(&self) -> &str {
        &self.data.name
    }
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: ContainerCategory = ContainerCategory;
        &CAT
    }
    fn summary(&self) -> &str {
        &self.data.summary
    }
    fn icon_path(&self) -> Option<&str> {
        self.data.icon_path.as_deref()
    }
    fn tags(&self) -> &[String] {
        &self.data.tags
    }
    fn releases(&self) -> &[PackageRelease] {
        &self.data.releases
    }
    fn help(&self) -> &PackageHelp {
        &self.data.help
    }
}

/// Container-specific extension methods.
pub trait ContainerExt: Package {
    /// Ports exposed by this service.
    fn ports(&self) -> &[ServicePort];

    /// User-configurable deployment variables.
    fn variables(&self) -> &[Variable];

    /// Optional feature flags.
    fn features(&self) -> &[Feature];

    /// `true` when the user must provide at least one required variable.
    fn requires_configuration(&self) -> bool {
        self.variables().iter().any(|v| v.required)
    }
}

impl ContainerExt for ContainerPackage {
    fn ports(&self) -> &[ServicePort] {
        &self.ports
    }
    fn variables(&self) -> &[Variable] {
        &self.variables
    }
    fn features(&self) -> &[Feature] {
        &self.features
    }
}
