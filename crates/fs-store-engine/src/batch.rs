//! Many installs as one batch. Pattern: Unit of Work (all downloads first, in parallel; one commit for the batch)
//! over the Command-like `Plan`. All rpm of a batch go into ONE `pkexec dnf install` (all deb into ONE `pkexec apt-get install`), so the person types the
//! password once; tar.gz packages are unpacked one after the other. A package whose download failed does not
//! stop the others.

use fs_store_core::{
    Format, InstallError, InstallState, InstallStrategy, Update, UpdateError, Version, run_update,
};

use crate::ending::{END_WAIT, end_old_servers};
use crate::prepare::Plan;
use crate::service::{InstallFailure, Installer, timing};
use crate::{
    Commander, Deb, Prepared, Rpm, SystemCommander, TarGz, end_servers, find_servers, is_alive,
    program_of, start_server,
};

/// Something to tell while a batch runs: which package reached which phase.
pub type Observer<'a> = &'a (dyn Fn(&str, InstallState) + Sync);
/// The same for the install step, which runs on one thread.
pub type CommitObserver<'a> = &'a dyn Fn(&str, InstallState);

impl Installer<'_> {
    /// Downloads and checks all plans at the same time (one thread each). The answers keep the order of `plans`.
    /// `observe` hears `Pending` when a download starts and `Verified` when the package is ready.
    #[must_use]
    pub fn prepare_all(
        &self,
        plans: Vec<Plan>,
        observe: Observer<'_>,
    ) -> Vec<(String, Result<Prepared, InstallFailure>)> {
        std::thread::scope(|scope| {
            let handles: Vec<_> = plans
                .into_iter()
                .map(|plan| {
                    scope.spawn(move || {
                        let name = plan.name().to_owned();
                        observe(&name, InstallState::Pending);
                        let result = self
                            .download(&plan)
                            .and_then(|downloaded| self.check(&plan, downloaded));
                        if result.is_ok() {
                            observe(&name, InstallState::Verified);
                        }
                        (name, result)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join().unwrap_or_else(|_| {
                        (
                            String::new(),
                            Err(InstallFailure::Fetch("a download crashed".to_owned())),
                        )
                    })
                })
                .collect()
        })
    }

    /// Installs everything prepared: all rpm with one `dnf` call, the rest one by one.
    #[must_use]
    pub fn commit_all(
        &self,
        prepared: Vec<Prepared>,
    ) -> Vec<(String, Result<Version, InstallFailure>)> {
        self.commit_observed(prepared, &|_, _| {})
    }

    /// Like [`Installer::commit_all`], and tells `observe` `Installing` for each package just before.
    #[must_use]
    pub fn commit_observed(
        &self,
        prepared: Vec<Prepared>,
        observe: CommitObserver<'_>,
    ) -> Vec<(String, Result<Version, InstallFailure>)> {
        // A background server (`fs-<name> serve`) never blocks an update: it is ended in order first (`Control.stop`,
        // SIGTERM for an old one). When the install failed, the old version starts again. When it worked, the new
        // one is not started here: the window of the program starts it when it needs it (`connect_matching`),
        // and the last job step ends what is still running of the old one.
        let mut results = Vec::new();
        let mut stopped = Vec::new();
        let mut go = Vec::new();
        for p in prepared {
            let servers = find_servers(p.name(), std::process::id());
            if servers.is_empty() {
                go.push(p);
            } else if end_servers(&servers, END_WAIT).is_ok() {
                stopped.push((p.name().to_owned(), servers));
                go.push(p);
            } else {
                // Not installed. A server that did end meanwhile is started again (the old version), so the update
                // that did not happen does not also take the server away.
                for s in servers.iter().filter(|s| !is_alive(s)) {
                    let _ = start_server(&program_of(&self.root, p.name(), s), s);
                }
                results.push((
                    p.name().to_owned(),
                    Err(InstallFailure::Update(UpdateError::Running)),
                ));
            }
        }
        let mut done = self.commit_stopped(go, observe);
        for (name, servers) in stopped {
            if done.iter().any(|(n, r)| *n == name && r.is_err()) {
                for s in &servers {
                    let _ = start_server(&program_of(&self.root, &name, s), s);
                }
            }
        }
        for (name, result) in &mut done {
            end_old_servers(name, END_WAIT, result);
        }
        results.extend(done);
        results
    }

    fn commit_stopped(
        &self,
        prepared: Vec<Prepared>,
        observe: CommitObserver<'_>,
    ) -> Vec<(String, Result<Version, InstallFailure>)> {
        let mut results = Vec::new();
        let (rpm, rest): (Vec<_>, Vec<_>) = prepared
            .into_iter()
            .partition(|p| p.verified.package().format == Format::Rpm);
        let (deb, rest): (Vec<_>, Vec<_>) = rest
            .into_iter()
            .partition(|p| p.verified.package().format == Format::Deb);
        // A program that runs is never replaced, also not inside a batch.
        let mut idle = |group: Vec<Prepared>| {
            let (idle, running): (Vec<_>, Vec<_>) = group
                .into_iter()
                .partition(|p| !self.running.is_running(p.name()));
            for p in running {
                results.push((
                    p.name().to_owned(),
                    Err(InstallFailure::Update(UpdateError::Running)),
                ));
            }
            idle
        };
        let (rpm, deb) = (idle(rpm), idle(deb));
        results.extend(self.commit_rpm_with(rpm, observe, SystemCommander));
        results.extend(self.commit_deb_with(deb, observe, SystemCommander));
        for p in rest {
            let name = p.name().to_owned();
            observe(&name, InstallState::Installing);
            let version = p.version();
            let mut files = p.files;
            let mut strategy: Box<dyn InstallStrategy> =
                Box::new(TarGz::from_bytes(&self.root, files.remove(0)));
            let t = std::time::Instant::now();
            let result = run_update(
                &Update {
                    package: &p.verified,
                    previous: p.previous,
                },
                strategy.as_mut(),
                self.running,
            );
            timing("install", &name, t);
            results.push((
                name,
                result.map(|_| version).map_err(InstallFailure::Update),
            ));
        }
        results
    }

    /// One `pkexec dnf install` for all rpm files of all packages: one transaction, one password.
    /// If it fails nothing was installed, so every package of the group gets the same cause.
    ///
    /// `commander` runs the command; tests give one that does not start anything.
    pub fn commit_rpm_with<C: Commander>(
        &self,
        group: Vec<Prepared>,
        observe: CommitObserver<'_>,
        commander: C,
    ) -> Vec<(String, Result<Version, InstallFailure>)> {
        let dir = self.root.join("rpm");
        commit_native(group, observe, "rpm batch", |files| {
            Box::new(Rpm::new(&dir, files, commander))
        })
    }

    /// The same for deb: one `pkexec apt-get install` for all deb files of all packages.
    pub fn commit_deb_with<C: Commander>(
        &self,
        group: Vec<Prepared>,
        observe: CommitObserver<'_>,
        commander: C,
    ) -> Vec<(String, Result<Version, InstallFailure>)> {
        let dir = self.root.join("deb");
        commit_native(group, observe, "deb batch", |files| {
            Box::new(Deb::new(&dir, files, commander))
        })
    }
}

/// Pattern: Template method for the native formats. One strategy installs the files of the whole group; its answer
/// is the answer of every package in it.
fn commit_native<'a>(
    group: Vec<Prepared>,
    observe: CommitObserver<'_>,
    label: &str,
    strategy: impl FnOnce(Vec<Vec<u8>>) -> Box<dyn InstallStrategy + 'a>,
) -> Vec<(String, Result<Version, InstallFailure>)> {
    let Some(first) = group.first().map(|p| p.verified.clone()) else {
        return Vec::new();
    };
    for p in &group {
        observe(p.name(), InstallState::Installing);
    }
    let names: Vec<(String, Version)> = group
        .iter()
        .map(|p| (p.name().to_owned(), p.version()))
        .collect();
    let files = group.into_iter().flat_map(|p| p.files).collect();
    let mut strategy = strategy(files);
    let t = std::time::Instant::now();
    let outcome = strategy.install(&first);
    timing("install", label, t);
    names
        .into_iter()
        .map(|(name, version)| {
            let r = match &outcome {
                Ok(()) => Ok(version),
                Err(InstallError(cause)) => Err(InstallFailure::Update(UpdateError::RolledBack(
                    InstallError(cause.clone()),
                ))),
            };
            (name, r)
        })
        .collect()
}
