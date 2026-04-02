// installer/pipeline.rs — Install-Pipeline (Pipeline Pattern).
//
// Design: Pipeline Pattern
//   InstallStep (trait)  — one discrete phase of an installation
//   InstallContext        — mutable context threaded through the pipeline
//   StepOutcome           — Continue | Skip (a step may skip itself gracefully)
//   Pipeline              — ordered list of steps; stops on first Err
//
// Concrete steps:
//   DownloadStep          — fetch compose/binary/artifact from Store
//   ValidateStep          — (placeholder) checksum / signature check
//   InstallFileStep       — write files to disk, call system installer
//   AdapterInstallStep    — auto-install the adapter accompanying a program package
//   InventoryStep         — notify fs-inventory via bus event (best-effort)
//   RegistryStep          — notify fs-registry via bus event (best-effort)
//   PublishEventStep      — publish install::completed on the bus (best-effort)
//
// All steps receive an `&mut InstallContext` and return `Result<StepOutcome, String>`.
// An `Err` aborts the pipeline; `Ok(Skip)` skips without aborting.

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, warn};

use super::{
    fetch_icon_content, install_app_binary, install_container, install_language_pack,
    install_theme_file, InstallKind, InstallRequest,
};

// ── InstallTarget ─────────────────────────────────────────────────────────────

/// Where / how the package is installed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallTarget {
    /// Podman / Docker container (compose + Quadlet).
    Container,
    /// RPM system package.
    Rpm,
    /// Debian/Ubuntu system package.
    Deb,
    /// Flatpak sandbox.
    Flatpak,
    /// Portable `AppImage` binary.
    AppImage,
}

impl InstallTarget {
    /// Human-readable label used in UI and log messages.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Container => "Container",
            Self::Rpm => "RPM",
            Self::Deb => "DEB",
            Self::Flatpak => "Flatpak",
            Self::AppImage => "AppImage",
        }
    }
}

// ── StepOutcome ───────────────────────────────────────────────────────────────

/// What a pipeline step produced.
#[derive(Debug)]
pub enum StepOutcome {
    /// Continue to the next step.
    Continue,
    /// Skip gracefully — not an error, just not applicable.
    Skip { reason: &'static str },
}

// ── PipelineEvent ─────────────────────────────────────────────────────────────

/// Progress event published on the pipeline channel.
///
/// The UI subscribes to a `Receiver<PipelineEvent>` to show progress.
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    /// A step started.
    StepStarted { step: String },
    /// A step was skipped.
    StepSkipped { step: String, reason: String },
    /// A step finished successfully.
    StepCompleted { step: String },
    /// The entire pipeline finished successfully.
    Done,
    /// The pipeline failed.
    Failed { step: String, reason: String },
}

// ── InstallContext ─────────────────────────────────────────────────────────────

/// Mutable context passed through every pipeline step.
pub struct InstallContext {
    /// The install request (id, name, kind, version, …).
    pub request: InstallRequest,

    /// Where to install.
    pub target: InstallTarget,

    /// Base directory for `FreeSynergy` data (`~/.local/share/fsn` by default).
    pub fs_dir: std::path::PathBuf,

    /// Path to the installed artifact produced by `InstallFileStep`, if any.
    pub artifact_path: Option<String>,

    /// Environment variables entered by the user in the configure step.
    pub env_vars: String,

    /// Optional URL of the registry service for capability registration.
    /// If `None`, the registry step is skipped.
    pub registry_url: Option<String>,

    /// Optional URL of the inventory service for package registration.
    /// If `None`, the inventory step is skipped.
    pub inventory_url: Option<String>,

    /// Optional URL of the bus service for event publishing.
    /// If `None`, the event step is skipped.
    pub bus_url: Option<String>,

    /// Progress channel — send events so the UI can display them.
    pub progress: Option<mpsc::UnboundedSender<PipelineEvent>>,
}

impl InstallContext {
    /// Create a minimal context for testing (no service URLs, no progress channel).
    #[must_use]
    pub fn new(request: InstallRequest, target: InstallTarget, fs_dir: std::path::PathBuf) -> Self {
        Self {
            request,
            target,
            fs_dir,
            artifact_path: None,
            env_vars: String::new(),
            registry_url: None,
            inventory_url: None,
            bus_url: None,
            progress: None,
        }
    }

    fn emit(&self, event: PipelineEvent) {
        if let Some(tx) = &self.progress {
            let _ = tx.send(event);
        }
    }
}

// ── InstallStep trait ─────────────────────────────────────────────────────────

/// One discrete phase of the installation pipeline.
///
/// Each step receives a mutable `InstallContext`, may mutate it (e.g. set
/// `artifact_path`), and returns either `Continue` or `Skip`.  An `Err`
/// aborts the entire pipeline.
#[async_trait]
pub trait InstallStep: Send + Sync {
    /// Short name used in log messages and progress events.
    fn name(&self) -> &'static str;

    /// Execute this step.
    ///
    /// # Errors
    ///
    /// Returns an error string that stops the pipeline.
    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String>;
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

/// Ordered sequence of [`InstallStep`]s.
///
/// `run()` threads the `InstallContext` through each step.  A step may skip
/// itself; an error aborts the entire run and emits `PipelineEvent::Failed`.
pub struct Pipeline {
    steps: Vec<Box<dyn InstallStep>>,
}

impl Pipeline {
    /// Build the standard pipeline for the given request and target.
    #[must_use]
    pub fn for_request(req: &InstallRequest, target: &InstallTarget) -> Self {
        let mut steps: Vec<Box<dyn InstallStep>> = vec![
            Box::new(DownloadStep),
            Box::new(ValidateStep),
            Box::new(InstallFileStep),
        ];

        // Auto-install adapter for program/container packages.
        if matches!(req.kind, InstallKind::Container | InstallKind::App) {
            steps.push(Box::new(AdapterInstallStep));
        }

        // Service integration (best-effort).
        steps.push(Box::new(InventoryStep));
        steps.push(Box::new(RegistryStep));
        steps.push(Box::new(PublishEventStep));

        let _ = target; // target influences InstallFileStep behaviour via ctx
        Self { steps }
    }

    /// Run all steps in order.
    ///
    /// # Errors
    ///
    /// Returns the error from the first step that fails.
    pub async fn run(self, ctx: &mut InstallContext) -> Result<(), String> {
        for step in &self.steps {
            let name = step.name().to_string();
            ctx.emit(PipelineEvent::StepStarted { step: name.clone() });
            info!(step = %name, "install step started");

            match step.execute(ctx).await {
                Ok(StepOutcome::Continue) => {
                    info!(step = %name, "install step completed");
                    ctx.emit(PipelineEvent::StepCompleted { step: name });
                }
                Ok(StepOutcome::Skip { reason }) => {
                    info!(step = %name, %reason, "install step skipped");
                    ctx.emit(PipelineEvent::StepSkipped {
                        step: name,
                        reason: reason.to_string(),
                    });
                }
                Err(e) => {
                    warn!(step = %name, error = %e, "install step failed");
                    ctx.emit(PipelineEvent::Failed {
                        step: name,
                        reason: e.clone(),
                    });
                    return Err(e);
                }
            }
        }

        ctx.emit(PipelineEvent::Done);
        Ok(())
    }
}

// ── DownloadStep ──────────────────────────────────────────────────────────────

/// Fetch the artifact from the Store and resolve the icon.
///
/// Sets `ctx.artifact_path` for file-based packages (container compose,
/// language pack, theme). For app binaries and adapter packages this step
/// is a no-op (the binary is fetched in `InstallFileStep`).
struct DownloadStep;

#[async_trait]
impl InstallStep for DownloadStep {
    fn name(&self) -> &'static str {
        "download"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        // Resolve icon content (non-fatal).
        let icon = fetch_icon_content(ctx.request.icon.clone()).await;
        if !icon.is_empty() {
            ctx.request.icon = Some(icon);
        }
        Ok(StepOutcome::Continue)
    }
}

// ── ValidateStep ──────────────────────────────────────────────────────────────

/// Verify the downloaded artifact (checksum / signature).
///
/// Currently a no-op placeholder.  Once the Store publishes checksums this
/// step will compare SHA-256 hashes and reject tampered artifacts.
struct ValidateStep;

#[async_trait]
impl InstallStep for ValidateStep {
    fn name(&self) -> &'static str {
        "validate"
    }

    async fn execute(&self, _ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        // TODO(phase-3): verify SHA-256 / Ed25519 signature from Store catalog.
        Ok(StepOutcome::Skip {
            reason: "validation not yet implemented",
        })
    }
}

// ── InstallFileStep ───────────────────────────────────────────────────────────

/// Write files to disk and invoke the system installer.
///
/// Delegates to the existing per-kind install helpers.  Sets
/// `ctx.artifact_path` on success.
struct InstallFileStep;

#[async_trait]
impl InstallStep for InstallFileStep {
    fn name(&self) -> &'static str {
        "install"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        let path = match (&ctx.request.kind, &ctx.target) {
            (InstallKind::Container, InstallTarget::Container) => {
                install_container(&ctx.request, &ctx.fs_dir, &ctx.env_vars).await?
            }
            (InstallKind::Language, _) => install_language_pack(&ctx.request, &ctx.fs_dir).await?,
            (InstallKind::Theme, _) => install_theme_file(&ctx.request, &ctx.fs_dir).await?,
            (InstallKind::App, _) => {
                let installed_by = ctx.request.store_path.as_deref();
                install_app_binary(&ctx.request, installed_by).await?
            }
            (InstallKind::Bundle, _) => {
                // Bundle install iterates components — handled at a higher level.
                return Ok(StepOutcome::Skip {
                    reason: "bundle components installed separately",
                });
            }
            (InstallKind::Other, _) => {
                return Ok(StepOutcome::Skip {
                    reason: "register-only package, no file to install",
                });
            }
            _ => {
                return Ok(StepOutcome::Skip {
                    reason: "target/kind combination not implemented yet",
                });
            }
        };

        ctx.artifact_path = path;
        Ok(StepOutcome::Continue)
    }
}

// ── AdapterInstallStep ────────────────────────────────────────────────────────

/// Auto-install the adapter package that accompanies a program/container.
///
/// The catalog declares `[adapter] id = "fs-channel-matrix"` etc.  When a
/// program package is installed its adapter is installed automatically so the
/// capability shows up in fs-registry without manual user action.
///
/// Currently a best-effort step — skips if no adapter is declared or if the
/// adapter install fails (the program itself is already installed).
struct AdapterInstallStep;

#[async_trait]
impl InstallStep for AdapterInstallStep {
    fn name(&self) -> &'static str {
        "adapter-install"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        // Adapter id is carried in `capabilities` as `"adapter:<id>"` by convention.
        // The catalog reader sets this when it parses the `[adapter]` table.
        let adapter_cap = ctx
            .request
            .capabilities
            .iter()
            .find(|c| c.starts_with("adapter:"))
            .cloned();

        let Some(cap) = adapter_cap else {
            return Ok(StepOutcome::Skip {
                reason: "no adapter declared for this package",
            });
        };

        let adapter_id = cap.trim_start_matches("adapter:").to_string();
        info!(adapter = %adapter_id, "auto-installing adapter");

        let adapter_req = InstallRequest {
            id: adapter_id.clone(),
            name: adapter_id.clone(),
            kind: InstallKind::Other,
            version: "latest".to_string(),
            store_path: Some(format!("packages/adapters/{adapter_id}")),
            capabilities: vec![],
            icon: None,
        };

        // Adapters are register-only — InstallKind::Other skips file install.
        let mut adapter_ctx =
            InstallContext::new(adapter_req, InstallTarget::Container, ctx.fs_dir.clone());
        adapter_ctx.registry_url = ctx.registry_url.clone();
        adapter_ctx.inventory_url = ctx.inventory_url.clone();
        adapter_ctx.bus_url = ctx.bus_url.clone();

        let pipeline = Pipeline::for_request(&adapter_ctx.request, &adapter_ctx.target);
        if let Err(e) = pipeline.run(&mut adapter_ctx).await {
            warn!(adapter = %adapter_id, error = %e, "adapter install failed — continuing");
        }

        Ok(StepOutcome::Continue)
    }
}

// ── InventoryStep ─────────────────────────────────────────────────────────────

/// Notify fs-inventory that a new package was installed.
///
/// Sends a POST to `{inventory_url}/api/v1/inventory/upsert` with the package
/// id and version.  Best-effort — skips if no inventory URL is configured or
/// if the call fails.
struct InventoryStep;

#[async_trait]
impl InstallStep for InventoryStep {
    fn name(&self) -> &'static str {
        "inventory"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        let Some(base_url) = &ctx.inventory_url else {
            return Ok(StepOutcome::Skip {
                reason: "inventory URL not configured",
            });
        };

        let url = format!("{base_url}/api/v1/inventory/upsert");
        let body = serde_json::json!({
            "id":      ctx.request.id,
            "name":    ctx.request.name,
            "version": ctx.request.version,
            "path":    ctx.artifact_path,
        });

        match reqwest::Client::new()
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!(id = %ctx.request.id, "inventory updated");
                Ok(StepOutcome::Continue)
            }
            Ok(resp) => {
                warn!(id = %ctx.request.id, status = %resp.status(), "inventory update returned non-2xx");
                Ok(StepOutcome::Skip {
                    reason: "inventory returned non-2xx",
                })
            }
            Err(e) => {
                warn!(id = %ctx.request.id, error = %e, "inventory update failed (best-effort)");
                Ok(StepOutcome::Skip {
                    reason: "inventory unreachable",
                })
            }
        }
    }
}

// ── RegistryStep ──────────────────────────────────────────────────────────────

/// Register the installed package's capabilities in fs-registry.
///
/// Sends a POST to `{registry_url}/api/v1/registry/register` with the
/// capability list.  Best-effort — skips if no registry URL is configured.
struct RegistryStep;

#[async_trait]
impl InstallStep for RegistryStep {
    fn name(&self) -> &'static str {
        "registry"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        if ctx.request.capabilities.is_empty() {
            return Ok(StepOutcome::Skip {
                reason: "package declares no capabilities",
            });
        }

        let Some(base_url) = &ctx.registry_url else {
            return Ok(StepOutcome::Skip {
                reason: "registry URL not configured",
            });
        };

        let url = format!("{base_url}/api/v1/registry/register");
        let body = serde_json::json!({
            "package_id":   ctx.request.id,
            "capabilities": ctx.request.capabilities,
        });

        match reqwest::Client::new()
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!(id = %ctx.request.id, "capabilities registered");
                Ok(StepOutcome::Continue)
            }
            Ok(resp) => {
                warn!(status = %resp.status(), "registry register returned non-2xx");
                Ok(StepOutcome::Skip {
                    reason: "registry returned non-2xx",
                })
            }
            Err(e) => {
                warn!(error = %e, "registry unreachable (best-effort)");
                Ok(StepOutcome::Skip {
                    reason: "registry unreachable",
                })
            }
        }
    }
}

// ── PublishEventStep ──────────────────────────────────────────────────────────

/// Publish `install::completed` on the `FreeSynergy` Bus.
///
/// Other services (Desktop, Managers, Lenses) subscribe to this topic to
/// refresh their state without polling.  Best-effort — skips if no bus URL
/// is configured.
struct PublishEventStep;

#[async_trait]
impl InstallStep for PublishEventStep {
    fn name(&self) -> &'static str {
        "publish-event"
    }

    async fn execute(&self, ctx: &mut InstallContext) -> Result<StepOutcome, String> {
        let Some(base_url) = &ctx.bus_url else {
            return Ok(StepOutcome::Skip {
                reason: "bus URL not configured",
            });
        };

        let url = format!("{base_url}/api/v1/bus/publish");
        let body = serde_json::json!({
            "topic": "install::completed",
            "payload": {
                "id":      ctx.request.id,
                "name":    ctx.request.name,
                "version": ctx.request.version,
                "target":  ctx.target.label(),
                "path":    ctx.artifact_path,
            },
        });

        match reqwest::Client::new()
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!(id = %ctx.request.id, "install::completed event published");
                Ok(StepOutcome::Continue)
            }
            Ok(resp) => {
                warn!(status = %resp.status(), "bus publish returned non-2xx");
                Ok(StepOutcome::Skip {
                    reason: "bus returned non-2xx",
                })
            }
            Err(e) => {
                warn!(error = %e, "bus unreachable (best-effort)");
                Ok(StepOutcome::Skip {
                    reason: "bus unreachable",
                })
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_request(kind: InstallKind) -> InstallRequest {
        InstallRequest {
            id: "test-pkg".to_string(),
            name: "Test Package".to_string(),
            kind,
            version: "1.0.0".to_string(),
            store_path: None,
            capabilities: vec![],
            icon: None,
        }
    }

    #[test]
    fn install_target_label() {
        assert_eq!(InstallTarget::Container.label(), "Container");
        assert_eq!(InstallTarget::Rpm.label(), "RPM");
        assert_eq!(InstallTarget::Deb.label(), "DEB");
        assert_eq!(InstallTarget::Flatpak.label(), "Flatpak");
        assert_eq!(InstallTarget::AppImage.label(), "AppImage");
    }

    #[test]
    fn pipeline_builds_for_bundle() {
        let req = dummy_request(InstallKind::Bundle);
        let pipeline = Pipeline::for_request(&req, &InstallTarget::Container);
        // Bundle: no AdapterInstallStep (no Container/App kind).
        assert!(!pipeline.steps.is_empty());
    }

    #[test]
    fn pipeline_builds_for_container() {
        let req = dummy_request(InstallKind::Container);
        let pipeline = Pipeline::for_request(&req, &InstallTarget::Container);
        // Container: AdapterInstallStep is included.
        let has_adapter = pipeline.steps.iter().any(|s| s.name() == "adapter-install");
        assert!(has_adapter);
    }

    #[tokio::test]
    async fn pipeline_runs_bundle_skips_install() {
        let req = dummy_request(InstallKind::Bundle);
        let tmp = std::env::temp_dir().join("fs-store-test-pipeline");
        let _ = std::fs::create_dir_all(&tmp);
        let mut ctx = InstallContext::new(req.clone(), InstallTarget::Container, tmp);

        let pipeline = Pipeline::for_request(&req, &InstallTarget::Container);
        // Must not fail — bundle install step skips itself.
        let result = pipeline.run(&mut ctx).await;
        assert!(result.is_ok(), "pipeline failed: {result:?}");
    }

    #[tokio::test]
    async fn pipeline_runs_other_skips_install() {
        let req = dummy_request(InstallKind::Other);
        let tmp = std::env::temp_dir().join("fs-store-test-pipeline-other");
        let _ = std::fs::create_dir_all(&tmp);
        let mut ctx = InstallContext::new(req.clone(), InstallTarget::Container, tmp);

        let pipeline = Pipeline::for_request(&req, &InstallTarget::Container);
        let result = pipeline.run(&mut ctx).await;
        assert!(result.is_ok(), "pipeline failed: {result:?}");
    }
}
