//! Pattern: Strategy (a second `StoreBackend`, in memory) for tests of the window that need no real store.
//!
//! Why it exists: the typing test (`fs-release/actions/typing-test`) starts the real binary on a virtual screen and
//! types into the field of the sources view. The real engine would read the person's folders and clone the
//! catalog from the network, and a job would install real packages. `fs-store --demo` runs the same window over this
//! backend: its sources and programs live in memory, it reads no catalog from the network, and it installs nothing (a job only
//! pretends and says so). It is never chosen on its own: only the flag `--demo` selects it.

use std::sync::Mutex;

use fs_store_core::{Kind, SourceError, Version};

use crate::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase, SourceRow,
    StoreBackend,
};

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
