// container.rs — ContainerPackage: server-side Podman/Quadlet services.

use serde::{Deserialize, Serialize};

use crate::category::{ContainerCategory, PackageCategory};
use crate::package::mod_prelude::*;

// ── Storage + API types ───────────────────────────────────────────────────────

/// Filesystem paths reserved by a container package.
///
/// Mirrors the `[storage]` section of the package `catalog.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StoragePaths {
    /// Per-user private data directory, e.g. `"~/.local/share/freesynergy/{pkg}"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Global/system-wide data directory, e.g. `"/var/lib/freesynergy/{pkg}"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<String>,
    /// Configuration directory, e.g. `"/etc/freesynergy/{pkg}"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    /// Cache directory, e.g. `"/var/cache/freesynergy/{pkg}"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache: Option<String>,
}

impl StoragePaths {
    /// `true` when at least one path is declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.user.is_none()
            && self.global.is_none()
            && self.config.is_none()
            && self.cache.is_none()
    }
}

/// A REST API endpoint exposed by a container package.
///
/// Mirrors one entry of `[[api.rest]]` in the package `catalog.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiEndpoint {
    /// URL path prefix, e.g. `"/api/v1"`.
    pub base: String,
    /// TCP port (defaults to 80 / 443 if absent).
    pub port: Option<u16>,
    /// Protocol: `"http"` or `"https"`.
    pub proto: String,
    /// Human-readable description of this endpoint.
    pub description: String,
}

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
/// Extends [`Package`] with ports, deployment variables, feature toggles,
/// storage paths, and API endpoint metadata.
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

    /// Filesystem paths reserved by this package (from `[storage]`).
    #[serde(default)]
    pub storage: StoragePaths,

    /// REST API endpoints exposed by this package (from `[[api.rest]]`).
    #[serde(default)]
    pub api_endpoints: Vec<ApiEndpoint>,
}

impl Package for ContainerPackage {
    impl_package_data!();
    fn category(&self) -> &'static dyn PackageCategory {
        static CAT: ContainerCategory = ContainerCategory;
        &CAT
    }
    fn storage(&self) -> Option<&StoragePaths> {
        if self.storage.is_empty() {
            None
        } else {
            Some(&self.storage)
        }
    }
    fn api_endpoints(&self) -> &[ApiEndpoint] {
        &self.api_endpoints
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

    /// Filesystem paths reserved by this package.
    fn container_storage(&self) -> &StoragePaths;

    /// REST API endpoints exposed by this package.
    fn container_api_endpoints(&self) -> &[ApiEndpoint];

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
    fn container_storage(&self) -> &StoragePaths {
        &self.storage
    }
    fn container_api_endpoints(&self) -> &[ApiEndpoint] {
        &self.api_endpoints
    }
}
