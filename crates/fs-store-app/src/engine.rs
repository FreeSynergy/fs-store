//! Pattern: Adapter. The in-process engine behind [`StoreBackend`]; it does what the CLI `fs-store` does (the
//! CLI keeps that code inside its `main`, so it is repeated here on purpose, not shared). Replaced by a
//! Cap'n Proto client of `store.capnp` later.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use fs_store_catalog::parse_catalog;
use fs_store_core::{SignaturePolicy, SignatureVerifier, SourceId, SourceRepository, Version};
use fs_store_engine::net::{HttpsFetch, clone_catalog};
use fs_store_engine::service::{Fetch, FileFetch};
use fs_store_pgp::PgpVerifier;
use fs_store_sources::FileSources;

use crate::backend::{
    BackendError, InstalledDetails, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase,
    Prepared, SourceRow, StoreBackend,
};

pub(crate) struct AnyFetch;

impl Fetch for AnyFetch {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        if url.starts_with("file://") {
            FileFetch.get(url)
        } else {
            HttpsFetch.get(url)
        }
    }
}

/// The engine, working below `root`.
#[derive(Debug, Clone)]
pub struct EngineBackend {
    root: PathBuf,
    /// Names the sources offered last time: they are looked up in rpm too.
    offered: Arc<Mutex<Vec<String>>>,
}

impl EngineBackend {
    /// Works below `root` (`sources.toml`, `packages/`, `current/`).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            offered: Arc::default(),
        }
    }

    /// `FS_STORE_ROOT`, or `~/.local/share/fs-store`, like the CLI.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::new(std::env::var_os("FS_STORE_ROOT").map_or_else(
            || {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join(".local/share/fs-store")
            },
            PathBuf::from,
        ))
    }

    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub(crate) fn sources_file(&self) -> Result<FileSources, BackendError> {
        self.file()
    }

    fn file(&self) -> Result<FileSources, BackendError> {
        FileSources::load(&self.root.join("sources.toml")).map_err(other)
    }

    /// Clones one source; returns catalog and signature bytes.
    fn fetch_source(&self, url: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        let work = self.root.join("sources").join("work");
        let _ = fs::remove_dir_all(&work);
        fs::create_dir_all(work.parent().unwrap_or(&self.root)).map_err(|e| e.to_string())?;
        clone_catalog(url, &work)?;
        let read = |file: &str| {
            let path = work.join(file);
            fs::read(&path).map_err(|e| format!("{file}: {e}"))
        };
        let catalog = read("catalog.xml")?;
        let sig = read("catalog.xml.asc")?;
        Ok((catalog, sig))
    }

    /// Catalog entries of one source, after its signature was checked.
    pub(crate) fn read_source(
        &self,
        url: &str,
    ) -> Result<(fs_store_catalog::Catalog, Vec<u8>, Vec<u8>), String> {
        let (catalog, sig) = self.fetch_source(url)?;
        let verifier = PgpVerifier::builtin();
        if !SignaturePolicy::default().is_satisfied_by(&verifier.verify(&catalog, Some(&sig))) {
            return Err("the signature of the catalog is not valid".to_owned());
        }
        let parsed = parse_catalog(&String::from_utf8_lossy(&catalog)).map_err(|e| e.0)?;
        fs_store_sources::ensure_fresh(
            &self.root.join("serials.toml"),
            url,
            parsed.serial,
            parsed.expires,
        )?;
        Ok((parsed, catalog, sig))
    }

    /// After a good install: note it in the register and put the commands on the PATH, as the commands do.
    /// Neither may fail the install that already happened.
    pub(crate) fn remember(&self, name: &str, version: Version, source: &str, facets: &[String]) {
        let path = self.root.join("installed.toml");
        if let Ok(mut registry) = fs_store_sources::Registry::load(&path) {
            registry.record(name, &version.to_string(), source);
            let made = fs_store_engine::link_facets(
                &self.root,
                name,
                facets,
                &fs_store_engine::default_bin_dir(),
                &fs_store_engine::default_apps_dir(),
            )
            .unwrap_or_default();
            registry.set_facets(name, made);
            let _ = registry.save();
        }
        let _ =
            fs_store_engine::link_programs(&self.root, name, &fs_store_engine::default_bin_dir());
    }

    pub(crate) fn installed_version(&self, name: &str) -> Option<Version> {
        let known = crate::installed::detect(
            &self.root,
            &fs_store_engine::rpm_installed_version,
            &[name.to_owned()],
        );
        known
            .into_iter()
            .find(|r| r.name == name)
            .map(|r| r.version)
    }
}

fn other(error: impl std::fmt::Debug) -> BackendError {
    BackendError::Other(format!("{error:?}"))
}

use crate::engine_map::{id_from_url, row};

impl StoreBackend for EngineBackend {
    fn sources(&self) -> Result<Vec<SourceRow>, BackendError> {
        Ok(self.file()?.list().into_iter().map(row).collect())
    }

    fn add_source(&self, url: &str) -> Result<SourceRow, BackendError> {
        let mut list = self.file()?;
        fs::create_dir_all(&self.root).map_err(other)?;
        let id = id_from_url(url);
        list.add(url, SourceId(id.clone()))
            .map_err(BackendError::Source)?;
        list.save().map_err(other)?;
        list.list()
            .into_iter()
            .find(|s| s.id.0 == id)
            .map(row)
            .ok_or_else(|| BackendError::Other("the source was not saved".to_owned()))
    }

    fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError> {
        let mut list = self.file()?;
        fs::create_dir_all(&self.root).map_err(other)?;
        list.set_enabled(&SourceId(id.to_owned()), enabled)
            .map_err(BackendError::Source)?;
        list.save().map_err(other)
    }

    fn remove_source(&self, id: &str) -> Result<(), BackendError> {
        let mut list = self.file()?;
        list.remove(&SourceId(id.to_owned()))
            .map_err(BackendError::Source)?;
        list.save().map_err(other)
    }

    fn available(&self) -> Result<Listing, BackendError> {
        let mut listing = Listing::default();
        for source in self.file()?.list().into_iter().filter(|s| s.enabled) {
            match self.read_source(&source.url) {
                Ok((catalog, _, _)) => {
                    listing
                        .packages
                        .extend(catalog.entries.into_iter().map(|e| PackageRow {
                            name: e.name.as_str().to_owned(),
                            kind: e.kind,
                            version: e.version,
                            source: source.id.0.clone(),
                            description: e.description,
                            homepage: e.homepage,
                        }));
                    listing
                        .skipped
                        .extend(catalog.skipped.into_iter().map(|s| (s.id, s.reason)));
                }
                Err(detail) => listing.unreadable.push((source.id.0, detail)),
            }
        }
        *self.offered.lock().unwrap_or_else(PoisonError::into_inner) =
            listing.packages.iter().map(|p| p.name.clone()).collect();
        Ok(listing)
    }

    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        let candidates = self
            .offered
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        Ok(crate::installed::detect(
            &self.root,
            &fs_store_engine::rpm_installed_version,
            &candidates,
        ))
    }

    fn icon_path(&self, name: &str) -> Option<PathBuf> {
        if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
            return None;
        }
        let path = self.root.join("icons").join(format!("{name}.png"));
        path.is_file().then_some(path)
    }

    fn details(&self, name: &str) -> InstalledDetails {
        crate::installed::details(
            &self.root,
            &fs_store_engine::default_bin_dir(),
            &fs_store_engine::rpm_installed_version,
            name,
        )
    }

    fn uninstall(&self, name: &str) -> Result<(), String> {
        crate::engine_remove::uninstall(&self.root, name)
    }

    fn run(
        &self,
        kind: JobKind,
        name: &str,
        observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError> {
        let prepared = self.prepare(kind, name)?;
        observe(Phase::Verified);
        observe(Phase::Installing);
        self.commit(vec![(name.to_owned(), prepared)], &mut |_, _| {})
            .pop()
            .map_or(Err(JobError::NotOffered), |(_, r)| r)
    }

    fn prepare(&self, kind: JobKind, name: &str) -> Result<Prepared, JobError> {
        self.prepare_job(kind, name)
    }

    fn commit(
        &self,
        batch: Vec<(String, Prepared)>,
        observe: &mut dyn FnMut(&str, Phase),
    ) -> Vec<(String, Result<Version, JobError>)> {
        self.commit_jobs(batch, observe)
    }
}
