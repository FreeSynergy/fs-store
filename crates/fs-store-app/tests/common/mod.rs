//! Shared helpers of the view-model tests: a fake backend that keeps its state in memory.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use fs_store_app::backend::{
    BackendError, InstalledDetails, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase,
    SourceRow, StoreBackend,
};
use fs_store_app::model::{Action, StoreModel};
use fs_store_core::{Kind, SourceError, Version};

pub struct Fake {
    pub state: Mutex<State>,
}

pub struct State {
    pub sources: Vec<SourceRow>,
    pub offered: Vec<PackageRow>,
    pub installed: Vec<InstalledRow>,
    pub fail: Option<JobError>,
    pub remove_calls: u32,
    pub uninstalls: Vec<String>,
    pub uninstall_fails: Option<String>,
    pub runs: Vec<(JobKind, String)>,
    pub details: Vec<(String, InstalledDetails)>,
    pub icons: Vec<(String, std::path::PathBuf)>,
}

pub fn v(a: u64, b: u64, c: u64) -> Version {
    Version::new(a, b, c)
}

pub fn package(name: &str, version: Version) -> PackageRow {
    PackageRow {
        name: name.to_owned(),
        title: String::new(),
        kind: Kind::Program,
        version,
        source: "freesynergy".to_owned(),
        description: String::new(),
        homepage: String::new(),
        details: fs_store_core::Details::default(),
        native: Vec::new(),
        locked: None,
    }
}

pub fn fake(installed: Vec<InstalledRow>, fail: Option<JobError>) -> Arc<Fake> {
    Arc::new(Fake {
        state: Mutex::new(State {
            sources: vec![SourceRow {
                id: "freesynergy".into(),
                url: "https://example.org/ref".into(),
                enabled: true,
                reference: true,
            }],
            offered: vec![
                package("fs-tasks", v(0, 6, 1)),
                package("fs-store", v(0, 2, 0)),
                package("fs-theme", v(1, 0, 0)),
            ],
            installed,
            fail,
            remove_calls: 0,
            uninstalls: Vec::new(),
            uninstall_fails: None,
            runs: Vec::new(),
            details: Vec::new(),
            icons: Vec::new(),
        }),
    })
}

impl StoreBackend for Fake {
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
        Ok(self.state.lock().unwrap().sources.clone())
    }
    fn add_source(&self, url: &str) -> Result<SourceRow, BackendError> {
        let mut s = self.state.lock().unwrap();
        if s.sources.iter().any(|x| x.url == url) {
            return Err(BackendError::Source(SourceError::Duplicate));
        }
        let row = SourceRow {
            id: url.rsplit('/').next().unwrap_or("x").to_owned(),
            url: url.to_owned(),
            enabled: true,
            reference: false,
        };
        s.sources.push(row.clone());
        Ok(row)
    }
    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError> {
        let mut s = self.state.lock().unwrap();
        let row = s
            .sources
            .iter_mut()
            .find(|x| x.id == id)
            .ok_or(BackendError::Source(SourceError::NotFound))?;
        row.enabled = enabled;
        Ok(())
    }
    fn remove_source(&self, id: &str) -> Result<(), BackendError> {
        let mut s = self.state.lock().unwrap();
        s.remove_calls += 1;
        s.sources.retain(|x| x.id != id);
        Ok(())
    }
    fn available(&self) -> Result<Listing, BackendError> {
        let s = self.state.lock().unwrap();
        let on = s.sources.iter().any(|x| x.enabled);
        Ok(Listing {
            packages: if on { s.offered.clone() } else { Vec::new() },
            ..Listing::default()
        })
    }
    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        Ok(self.state.lock().unwrap().installed.clone())
    }
    fn details(&self, name: &str) -> InstalledDetails {
        let s = self.state.lock().unwrap();
        s.details
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d.clone())
            .unwrap_or_default()
    }
    fn icon_path(&self, name: &str) -> Option<std::path::PathBuf> {
        let s = self.state.lock().unwrap();
        s.icons
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p.clone())
    }
    fn uninstall(&self, name: &str) -> Result<(), String> {
        let mut s = self.state.lock().unwrap();
        if let Some(e) = s.uninstall_fails.clone() {
            return Err(e);
        }
        s.uninstalls.push(name.to_owned());
        s.installed.retain(|i| i.name != name);
        Ok(())
    }
    fn run(
        &self,
        kind: JobKind,
        name: &str,
        observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError> {
        let mut s = self.state.lock().unwrap();
        s.runs.push((kind, name.to_owned()));
        observe(Phase::Verified);
        observe(Phase::Installing);
        if let Some(e) = s.fail.clone() {
            return Err(e);
        }
        let version = s
            .offered
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.version)
            .ok_or(JobError::NotOffered)?;
        s.installed.retain(|i| i.name != name);
        s.installed.push(InstalledRow {
            name: name.to_owned(),
            version,
        });
        Ok(version)
    }
}

pub fn model(fake: &Arc<Fake>) -> StoreModel {
    let mut m = StoreModel::new(fake.clone());
    m.refresh();
    m
}

pub fn action_of(m: &StoreModel, name: &str) -> Option<Action> {
    m.available()
        .iter()
        .find(|r| r.package.name == name)
        .map(|r| r.action)
}
