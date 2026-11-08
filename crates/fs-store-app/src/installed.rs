//! Pattern: Facade (one answer to "what is installed, and where" from the three places that know: the register
//! `installed.toml`, the `current/<name>` links and the rpm database). The rpm question is passed in, so tests
//! need no rpm.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use fs_store_core::Version;
use fs_store_sources::Registry;

use crate::backend::{InstallKind, InstalledDetails, InstalledRow};

/// The version `current/<name>` points to.
#[must_use]
pub fn linked_version(root: &Path, name: &str) -> Option<Version> {
    let target = fs::read_link(root.join("current").join(name)).ok()?;
    Version::parse(target.file_name()?.to_str()?)
}

/// Every installed package, once, with its newest known version. `candidates` are names that are looked up in
/// rpm too (the names the sources offer).
pub fn detect(
    root: &Path,
    rpm: &dyn Fn(&str) -> Option<Version>,
    candidates: &[String],
) -> Vec<InstalledRow> {
    let registry = Registry::load(&root.join("installed.toml")).ok();
    let mut names: BTreeSet<String> = candidates.iter().cloned().collect();
    if let Some(r) = &registry {
        names.extend(r.list().iter().map(|e| e.name.clone()));
    }
    if let Ok(dir) = fs::read_dir(root.join("current")) {
        names.extend(
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.')),
        );
    }
    names
        .into_iter()
        .filter_map(|name| {
            let registered = registry
                .as_ref()
                .and_then(|r| r.get(&name))
                .and_then(|e| Version::parse(&e.version));
            let version = [linked_version(root, &name), registered, rpm(&name)]
                .into_iter()
                .flatten()
                .max()?;
            Some(InstalledRow { name, version })
        })
        .collect()
}

/// Where the package `name` is, as far as that can be found out.
pub fn details(
    root: &Path,
    bin_dir: &Path,
    rpm: &dyn Fn(&str) -> Option<Version>,
    name: &str,
) -> InstalledDetails {
    let mut out = InstalledDetails::default();
    let folder = root.join("current").join(name);
    if linked_version(root, name).is_some() {
        out.kinds.push(InstallKind::Archive);
        out.folder = fs::canonicalize(&folder)
            .ok()
            .map(|p| p.display().to_string());
    }
    let from_rpm = rpm(name).is_some();
    if from_rpm {
        out.kinds.push(InstallKind::Rpm);
    }
    let mut commands: Vec<String> = vec![name.to_owned()];
    if let Some(entry) = Registry::load(&root.join("installed.toml"))
        .ok()
        .and_then(|r| r.get(name).cloned())
    {
        out.source = Some(entry.source);
        commands.extend(entry.facets.iter().cloned());
        out.facets = entry.facets;
    }
    if let Ok(dir) = fs::read_dir(&folder) {
        commands.extend(
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| !n.starts_with('.')),
        );
    }
    let mut seen = BTreeSet::new();
    for command in commands {
        if !seen.insert(command.clone()) {
            continue;
        }
        let places = [Some(bin_dir), from_rpm.then_some(Path::new("/usr/bin"))];
        if let Some(path) = places
            .into_iter()
            .flatten()
            .map(|dir| dir.join(&command))
            .find(|p| fs::symlink_metadata(p).is_ok())
        {
            out.commands.push((command, path.display().to_string()));
        }
    }
    out
}
