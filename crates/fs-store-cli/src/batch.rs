//! Pattern: Unit of Work over the Command `install`: several packages are one batch. All downloads run at the
//! same time; installing starts when every download is done, and then everything is installed at once (all
//! rpm in ONE `pkexec dnf` call, so the password is asked once). A package that fails does not stop the others.

use std::path::Path;

use fs_store_catalog::CatalogEntry;
use fs_store_engine::ProcRunningCheck;
use fs_store_engine::service::{Fetch, InstallFailure, Installer};

use crate::commands::{AnyFetch, fetch_source, installed, read_catalog, trust};

/// One package to install or update, and where to get it from (`None`: any enabled source).
pub(crate) struct Want {
    pub name: String,
    pub source: Option<String>,
}

/// What the batch found out about one wanted package before downloading.
struct Found {
    name: String,
    url: String,
    entry: CatalogEntry,
    catalog: Vec<u8>,
    sig: Vec<u8>,
}

/// Installs or updates all of `wants` as one batch; `only_newer` makes it an update.
pub(crate) fn install_batch(
    root: &Path,
    wants: &[Want],
    extra: Option<&String>,
    only_newer: bool,
) -> Result<(), String> {
    let (verifier, policy) = trust(extra)?;
    let running = ProcRunningCheck::new(root);
    let installer = Installer {
        root: root.to_owned(),
        verifier: &verifier,
        fetch: &AnyFetch,
        running: &running,
        prefer: fs_store_engine::preferred_format(),
        policy: policy.clone(),
    };
    let mut problems = Vec::new();
    let mut found = Vec::new();
    for want in wants {
        let name = want.name.as_str();
        let current = installed(root, name);
        if only_newer && current.is_none() {
            problems.push(format!("{name} is not installed; use install"));
            continue;
        }
        let mut last = format!("{name} is not offered by any source");
        let mut hit = None;
        for url in crate::commands::addresses(root, want.source.as_ref())? {
            let (catalog, sig) = match fetch_source(root, &url, &verifier, &policy) {
                Ok(x) => x,
                Err(e) => {
                    last = e;
                    continue;
                }
            };
            if let Some(entry) = read_catalog(&catalog)?
                .entries
                .into_iter()
                .find(|e| e.name.as_str() == name)
            {
                hit = Some(Found {
                    name: name.to_owned(),
                    url,
                    entry,
                    catalog,
                    sig,
                });
                break;
            }
        }
        let Some(hit) = hit else {
            problems.push(last);
            continue;
        };
        if only_newer && current.is_some_and(|c| hit.entry.version <= c) {
            println!(
                "{name} is up to date ({})",
                current.map_or_else(String::new, |c| c.to_string())
            );
            continue;
        }
        found.push(hit);
    }
    problems.extend(run_batch(root, &installer, found));
    if problems.is_empty() {
        return Ok(());
    }
    for p in &problems[..problems.len() - 1] {
        eprintln!("{p}");
    }
    Err(problems[problems.len() - 1].clone())
}

/// Plans, downloads (all at once), then installs everything that is ready. Returns what went wrong, per package.
fn run_batch(root: &Path, installer: &Installer<'_>, found: Vec<Found>) -> Vec<String> {
    let mut problems = Vec::new();
    let mut plans = Vec::new();
    let mut by_name = Vec::new();
    for f in found {
        let current = installed(root, &f.name);
        match installer.plan(&f.catalog, &f.sig, &f.name, current) {
            Ok(plan) => {
                plans.push(plan);
                by_name.push(f);
            }
            Err(e) => problems.push(format!("{}: {}: {e}", f.name, f.url)),
        }
    }
    if plans.len() > 1 {
        println!("downloading {} packages at once ...", plans.len());
    }
    let mut ready = Vec::new();
    for (name, result) in installer.prepare_all(plans, &|_, _| {}) {
        match result {
            Ok(p) => ready.push(p),
            Err(e) => problems.push(format!("{name}: {e}")),
        }
    }
    for (name, result) in installer.commit_all(ready) {
        match (result, by_name.iter().find(|f| f.name == name)) {
            (Ok(v), Some(f)) => {
                finish(root, f, v);
                println!("installed {name} {v}");
            }
            (Ok(_), None) => {}
            (Err(e @ InstallFailure::ServerNotEnded { version, .. }), Some(f)) => {
                finish(root, f, version);
                println!("installed {name} {version}");
                problems.push(format!("{name}: {e}"));
            }
            (Err(e), _) => problems.push(format!("{name}: {e}")),
        }
    }
    problems
}

/// After a good install: icon, register, programs on the PATH. None of it may fail the install that happened.
fn finish(root: &Path, f: &Found, v: fs_store_core::Version) {
    // The icon comes with a hash in the signed catalog; one that does not match is not stored.
    if let Some((icon, Ok(bytes))) = f.entry.icon.as_ref().map(|i| (i, AnyFetch.get(&i.url))) {
        let _ = fs_store_engine::store_icon(root, &f.name, &bytes, &icon.sha256);
    }
    if let Ok(mut registry) = fs_store_sources::Registry::load(&root.join("installed.toml")) {
        registry.record(&f.name, &v.to_string(), &f.url);
        registry.set_facets(
            &f.name,
            crate::updates::link_facets(root, &f.name, &f.entry.facets),
        );
        let _ = registry.save();
    }
    crate::updates::link(root, &f.name);
}
