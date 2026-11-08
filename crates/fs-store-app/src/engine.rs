//! Pattern: Adapter. The in-process engine behind [`StoreBackend`]; it does what the CLI `fs-store` does (the
//! CLI keeps that code inside its `main`, so it is repeated here on purpose, not shared). Replaced by a
//! Cap'n Proto client of `store.capnp` later.

use std::fs;
use std::path::PathBuf;

use fs_store_catalog::{CatalogEntry, parse_catalog};
use fs_store_core::{
    SignaturePolicy, SignatureVerifier, SourceId, SourceRepository, UpdateError, Version,
};
use fs_store_engine::ProcRunningCheck;
use fs_store_engine::net::{HttpsFetch, clone_catalog};
use fs_store_engine::service::{Fetch, FileFetch, InstallFailure, Installer};
use fs_store_pgp::PgpVerifier;
use fs_store_sources::FileSources;

use crate::backend::{
    BackendError, InstalledRow, JobError, JobKind, Listing, PackageRow, Phase, SourceRow,
    StoreBackend,
};

struct AnyFetch;

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
}

impl EngineBackend {
    /// Works below `root` (`sources.toml`, `packages/`, `current/`).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
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

    fn file(&self) -> Result<FileSources, BackendError> {
        FileSources::load(&self.root.join("sources.toml")).map_err(other)
    }

    /// Clones one source; returns catalog and signature bytes.
    fn fetch_source(&self, url: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        let work = self.root.join("sources").join("work");
        let _ = fs::remove_dir_all(&work);
        fs::create_dir_all(work.parent().unwrap_or(&self.root)).map_err(|e| e.to_string())?;
        clone_catalog(url, &work)?;
        let catalog = fs::read(work.join("catalog.xml")).map_err(|e| e.to_string())?;
        let sig = fs::read(work.join("catalog.xml.asc")).map_err(|e| e.to_string())?;
        Ok((catalog, sig))
    }

    /// Catalog entries of one source, after its signature was checked.
    fn read_source(
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
    fn remember(&self, name: &str, version: Version, source: &str, facets: &[String]) {
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

    fn installed_version(&self, name: &str) -> Option<Version> {
        let target = fs::read_link(self.root.join("current").join(name)).ok()?;
        Version::parse(target.file_name()?.to_str()?)
    }
}

fn other(error: impl std::fmt::Debug) -> BackendError {
    BackendError::Other(format!("{error:?}"))
}

fn row(s: fs_store_core::Source) -> SourceRow {
    SourceRow {
        id: s.id.0,
        url: s.url,
        enabled: s.enabled,
        reference: s.reference,
    }
}

fn id_from_url(url: &str) -> String {
    url.trim_end_matches(['/', '.'])
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("source")
        .trim_end_matches(".git")
        .to_owned()
}

fn job_error(failure: InstallFailure) -> JobError {
    match failure {
        InstallFailure::Fetch(text) | InstallFailure::Catalog(text) => JobError::Other(text),
        InstallFailure::CatalogSignature => JobError::CatalogSignature,
        InstallFailure::NotOffered => JobError::NotOffered,
        InstallFailure::Downgrade => JobError::Downgrade,
        InstallFailure::Unsupported(_) => JobError::Unsupported,
        InstallFailure::Checksum => JobError::Checksum,
        InstallFailure::ReleaseSignature => JobError::ReleaseSignature,
        InstallFailure::Update(UpdateError::Running) => JobError::Running,
        InstallFailure::Update(UpdateError::RolledBack) => {
            JobError::Other("the installer failed".to_owned())
        }
        InstallFailure::Update(UpdateError::RollbackFailed) => JobError::RollbackFailed,
    }
}

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
                        }));
                    listing
                        .skipped
                        .extend(catalog.skipped.into_iter().map(|s| (s.id, s.reason)));
                }
                Err(detail) => listing.unreadable.push((source.id.0, detail)),
            }
        }
        Ok(listing)
    }

    fn installed(&self) -> Result<Vec<InstalledRow>, BackendError> {
        let mut rows = Vec::new();
        let Ok(dir) = fs::read_dir(self.root.join("current")) else {
            return Ok(rows);
        };
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let (false, Some(version)) = (name.starts_with('.'), self.installed_version(&name)) {
                rows.push(InstalledRow { name, version });
            }
        }
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(rows)
    }

    fn run(
        &self,
        kind: JobKind,
        name: &str,
        observe: &mut dyn FnMut(Phase),
    ) -> Result<Version, JobError> {
        let current = self.installed_version(name);
        if kind == JobKind::Update && current.is_none() {
            return Err(JobError::NotOffered);
        }
        let verifier = PgpVerifier::builtin();
        let running = ProcRunningCheck::new(&self.root);
        let installer = Installer {
            root: self.root.clone(),
            verifier: &verifier,
            fetch: &AnyFetch,
            running: &running,
            policy: SignaturePolicy::default(),
        };
        let sources = self.file().map_err(|e| JobError::Other(format!("{e:?}")))?;
        let mut last = JobError::NotOffered;
        for source in sources.list().into_iter().filter(|s| s.enabled) {
            let (catalog, bytes, sig) = match self.read_source(&source.url) {
                Ok(x) => x,
                Err(detail) => {
                    last = JobError::Other(format!("{}: {detail}", source.id.0));
                    continue;
                }
            };
            let offered: Option<&CatalogEntry> =
                catalog.entries.iter().find(|e| e.name.as_str() == name);
            if offered.is_none() {
                continue;
            }
            let version = installer
                .install_observed(&bytes, &sig, name, current, observe)
                .map_err(job_error)?;
            self.remember(
                name,
                version,
                &source.url,
                &offered.map(|o| o.facets.clone()).unwrap_or_default(),
            );
            return Ok(version);
        }
        Err(last)
    }
}
