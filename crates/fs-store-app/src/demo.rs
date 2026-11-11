//! Pattern: Strategy (a second `StoreBackend`, in memory) for tests of the window that need no real store.
//!
//! Why it exists: the typing test (`fs-release/actions/typing-test`) starts the real binary on a virtual screen and
//! types into the field of the sources view. The real engine would read the person's folders and clone the
//! catalog from the network, and a job would install real packages. `fs-store --demo` runs the same window over this
//! backend: its sources and programs live in memory, it reads no catalog from the network, and it installs nothing (a job only
//! pretends and says so). It is never chosen on its own: only the flag `--demo` selects it.
//!
//! Sealed completely: [`DemoUpdates`] keeps settings and findings in memory and its check refuses (the real one starts
//! `fs-store background`, which clones catalogs, writes `updates.toml`, notifies and may install); no check runs at
//! the start; [`DemoLauncher`] starts no process and opens no address; the window puts no menu entry into the
//! person's folders. [`parts`] is the one place that chooses, so a test can prove it.

use std::sync::{Arc, Mutex};

use fs_store_core::{Kind, SourceError, Version};
use fs_store_sources::{Findings, Settings};

use crate::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase, SourceRow,
    StoreBackend,
};
use crate::engine::EngineBackend;
use crate::launcher::{Launcher, ProcessLauncher};
use crate::updates::{FileUpdates, UpdateService};

/// The in-memory store: one reference source, three programs, one installed.
#[derive(Debug)]
pub struct DemoBackend {
    sources: Mutex<Vec<SourceRow>>,
}

impl Default for DemoBackend {
    fn default() -> Self {
        Self {
            sources: Mutex::new(vec![SourceRow {
                id: "freesynergy".into(),
                url: "https://example.invalid/freesynergy".into(),
                enabled: true,
                reference: true,
            }]),
        }
    }
}

fn package(name: &str, title: &str, text: &str) -> PackageRow {
    PackageRow {
        name: name.into(),
        title: title.into(),
        kind: Kind::Program,
        version: Version::new(0, 2, 0),
        source: "freesynergy".into(),
        description: text.into(),
        homepage: "https://example.invalid".into(),
    }
}

impl DemoBackend {
    fn sources_locked(&self) -> std::sync::MutexGuard<'_, Vec<SourceRow>> {
        self.sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl StoreBackend for DemoBackend {
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
        Ok(self.sources_locked().clone())
    }
    fn add_source(&self, url: &str) -> Result<SourceRow, BackendError> {
        let mut sources = self.sources_locked();
        if sources.iter().any(|s| s.url == url) {
            return Err(BackendError::Source(SourceError::Duplicate));
        }
        let row = SourceRow {
            id: format!("demo-{}", sources.len()),
            url: url.to_owned(),
            enabled: true,
            reference: false,
        };
        sources.push(row.clone());
        Ok(row)
    }
    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError> {
        let mut sources = self.sources_locked();
        let row = sources
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or(BackendError::Source(SourceError::NotFound))?;
        row.enabled = enabled;
        Ok(())
    }
    fn remove_source(&self, id: &str) -> Result<(), BackendError> {
        let mut sources = self.sources_locked();
        match sources.iter().position(|s| s.id == id) {
            Some(i) if sources[i].reference => {
                Err(BackendError::Source(SourceError::ReferenceSource))
            }
            Some(i) => {
                sources.remove(i);
                Ok(())
            }
            None => Err(BackendError::Source(SourceError::NotFound)),
        }
    }
    fn available(&self) -> Result<Listing, BackendError> {
        Ok(Listing {
            packages: vec![
                package(
                    "fs-tasks",
                    "FreeSynergy.Tasks",
                    "Tasks for people and groups.",
                ),
                package("fs-people", "FreeSynergy.People", "Contacts and members."),
                package("fs-wiki", "FreeSynergy.Wiki", "Notes that belong together."),
            ],
            ..Listing::default()
        })
    }
    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        Ok(vec![InstalledRow {
            name: "fs-tasks".into(),
            version: Version::new(0, 1, 0),
        }])
    }
    fn uninstall(&self, _name: &str) -> Result<(), String> {
        Err("demo mode: nothing is installed, so nothing can be removed".to_owned())
    }
    fn run(
        &self,
        _kind: JobKind,
        _name: &str,
        _observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError> {
        Err(JobError::Unsupported(
            "demo mode (--demo) installs nothing".to_owned(),
        ))
    }
}

const NO_UPDATES: &str = "demo mode (--demo) looks for no updates";

/// Settings and findings in memory; the check is refused, nothing is read, written or started.
#[derive(Debug, Default)]
pub struct DemoUpdates {
    settings: Mutex<Settings>,
    findings: Mutex<Findings>,
}

fn held<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl UpdateService for DemoUpdates {
    fn settings(&self) -> Result<Settings, String> {
        Ok(*held(&self.settings))
    }
    fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        *held(&self.settings) = *settings;
        Ok(())
    }
    fn findings(&self) -> Result<Findings, String> {
        Ok(held(&self.findings).clone())
    }
    fn save_findings(&self, findings: &Findings) -> Result<(), String> {
        *held(&self.findings) = findings.clone();
        Ok(())
    }
    fn check(&self) -> Result<(), String> {
        Err(NO_UPDATES.to_owned())
    }
    fn now(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

/// Starts nothing: every call says why.
#[derive(Debug, Default, Clone, Copy)]
pub struct DemoLauncher;

impl Launcher for DemoLauncher {
    fn exists(&self, _command: &str) -> bool {
        false
    }
    fn start(&self, _command: &str) -> Result<(), String> {
        Err("demo mode (--demo) starts no program".to_owned())
    }
    fn open_url(&self, _url: &str) -> Result<(), String> {
        Err("demo mode (--demo) opens no address".to_owned())
    }
}

/// What the window runs on.
pub struct Parts {
    /// The store.
    pub backend: Arc<dyn StoreBackend>,
    /// Settings and the check for updates.
    pub updates: Arc<dyn UpdateService>,
    /// Starting other programs.
    pub launcher: Arc<dyn Launcher>,
    /// Whether to look for updates when the window opens (never in demo mode).
    pub check_at_start_allowed: bool,
}

/// The real parts, or with `demo` the sealed in-memory ones.
#[must_use]
pub fn parts(demo: bool) -> Parts {
    if demo {
        Parts {
            backend: Arc::new(DemoBackend::default()),
            updates: Arc::new(DemoUpdates::default()),
            launcher: Arc::new(DemoLauncher),
            check_at_start_allowed: false,
        }
    } else {
        Parts {
            backend: Arc::new(EngineBackend::from_environment()),
            updates: Arc::new(FileUpdates::from_environment()),
            launcher: Arc::new(ProcessLauncher::new(fs_store_engine::default_bin_dir())),
            check_at_start_allowed: true,
        }
    }
}
