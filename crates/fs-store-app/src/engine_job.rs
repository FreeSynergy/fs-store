//! Pattern: Unit of Work (the two halves of a job for the in-process engine: `prepare_job` downloads and
//! checks, `commit_jobs` installs a whole batch at once).

use std::collections::HashMap;

use fs_store_core::{SignaturePolicy, SourceRepository, Version};
use fs_store_engine::ProcRunningCheck;
use fs_store_engine::service::Installer;
use fs_store_pgp::PgpVerifier;

use crate::backend::{JobError, JobKind, Phase, Prepared};
use crate::engine::{AnyFetch, EngineBackend};
use crate::engine_map::job_error;

/// What `prepare_job` hands to `commit_jobs`.
struct Staged {
    ready: fs_store_engine::Prepared,
    source: String,
    signer: String,
    facets: Vec<String>,
}

impl EngineBackend {
    fn installer<'a>(
        &self,
        verifier: &'a PgpVerifier,
        running: &'a ProcRunningCheck,
    ) -> Installer<'a> {
        Installer {
            root: self.root().to_owned(),
            verifier,
            fetch: &AnyFetch,
            running,
            prefer: fs_store_engine::preferred_format(),
            policy: SignaturePolicy::default(),
        }
    }

    /// Finds the package in the enabled sources, downloads it and checks sum and signature. Changes nothing.
    pub(crate) fn prepare_job(&self, kind: JobKind, name: &str) -> Result<Prepared, JobError> {
        let current = self.installed_version(name);
        if kind == JobKind::Update && current.is_none() {
            return Err(JobError::NotOffered);
        }
        let verifier = PgpVerifier::builtin();
        let running = ProcRunningCheck::new(self.root());
        let installer = self.installer(&verifier, &running);
        let sources = self
            .sources_file()
            .map_err(|e| JobError::Other(format!("{e:?}")))?;
        let mut last = JobError::NotOffered;
        for source in sources.list().into_iter().filter(|s| s.enabled) {
            let (catalog, bytes, sig) = match self.read_source(&source.url) {
                Ok(x) => x,
                Err(detail) => {
                    last = JobError::Other(format!("{}: {detail}", source.id.0));
                    continue;
                }
            };
            let Some(offered) = catalog.entries.iter().find(|e| e.name.as_str() == name) else {
                continue;
            };
            let plan = installer
                .plan(&bytes, &sig, name, current)
                .map_err(job_error)?;
            let downloaded = installer.download(&plan).map_err(job_error)?;
            let ready = installer.check(&plan, downloaded).map_err(job_error)?;
            let signer = ready.signer().to_owned();
            return Ok(Prepared(Box::new(Staged {
                ready,
                source: source.url,
                signer,
                facets: offered.facets.clone(),
            })));
        }
        Err(last)
    }

    /// Installs all prepared packages in one step (all rpm with one `pkexec dnf`), then notes each good one.
    pub(crate) fn commit_jobs(
        &self,
        batch: Vec<(String, Prepared)>,
        observe: &mut dyn FnMut(&str, Phase),
    ) -> Vec<(String, Result<Version, JobError>)> {
        let mut staged = Vec::new();
        let mut results = Vec::new();
        for (name, p) in batch {
            match p.0.downcast::<Staged>() {
                Ok(s) => staged.push(*s),
                Err(_) => results.push((
                    name,
                    Err(JobError::Other("not prepared by this backend".to_owned())),
                )),
            }
        }
        let verifier = PgpVerifier::builtin();
        let running = ProcRunningCheck::new(self.root());
        let installer = self.installer(&verifier, &running);
        let notes: HashMap<String, (String, String, Vec<String>)> = staged
            .iter()
            .map(|s| {
                (
                    s.ready.name().to_owned(),
                    (s.source.clone(), s.signer.clone(), s.facets.clone()),
                )
            })
            .collect();
        let cell = std::cell::RefCell::new(observe);
        let tell = |name: &str, phase: Phase| (cell.borrow_mut())(name, phase);
        let ready = staged.into_iter().map(|s| s.ready).collect();
        for (name, result) in installer.commit_observed(ready, &tell) {
            let result = result.map_err(job_error);
            // The install happened also when only the old server could not be ended.
            let ok_version = match &result {
                Ok(version) | Err(JobError::ServerNotEnded { version, .. }) => Some(*version),
                Err(_) => None,
            };
            let mut result = result;
            if let (Some(version), Some((source, signer, facets))) = (ok_version, notes.get(&name))
            {
                let problems = self.remember(&name, version, source, signer, facets);
                if !problems.is_empty() {
                    result = Err(incomplete(result, version, &problems));
                }
            }
            results.push((name, result));
        }
        results
    }
}

/// The install worked but a step after it did not: one error that says both (a server that stayed is kept in the text).
fn incomplete(
    result: Result<Version, JobError>,
    version: Version,
    problems: &[String],
) -> JobError {
    let mut reason = problems.join("; ");
    if let Err(JobError::ServerNotEnded { reason: server, .. }) = result {
        reason = format!("the old background server could not be ended: {server}; {reason}");
    }
    JobError::Incomplete { version, reason }
}
