//! Pattern: Facade (`owner`): which of our packages a path belongs to, from the store's own lists, with the level of proof.
//! `rpm -qf` / `dpkg -S` are only an additional cross-check (`system_owner`), never required.

use std::fs;
use std::path::Path;

use fs_store_sources::{Basis, FileRecord, Manifest, ManifestStore};

use crate::own_files::{Verdict, package_dir};

/// The package a path belongs to, according to our own lists.
#[derive(Debug)]
pub struct Owner {
    /// The list (package, version, source, signer, basis).
    pub manifest: Manifest,
    /// The file entry.
    pub file: FileRecord,
    /// What the file is now.
    pub verdict: Verdict,
}

impl Owner {
    /// The level of proof, in words: recorded at install, made afterwards, changed, missing.
    #[must_use]
    pub fn proof(&self) -> &'static str {
        match (self.verdict, self.manifest.basis) {
            (Verdict::Changed, _) => "changed since it was listed",
            (Verdict::Missing, _) => "listed, but the file is gone",
            (Verdict::Mode, _) => "the permission bits changed since it was listed",
            (Verdict::Type, _) => "no longer a regular file (a link or a folder is there)",
            (Verdict::Unreadable, _) => "listed, but the file cannot be read",
            (Verdict::Intact, Basis::Installed) => {
                "recorded by the store when it installed the file"
            }
            (Verdict::Intact, Basis::Retroactive) => {
                "listed afterwards from the tree found on disk (not the original)"
            }
        }
    }
}

/// Finds which of our packages `path` belongs to (a link into a package counts: `~/.local/bin/fs-x` finds `fs-x`).
/// `None` when it is none of ours.
///
/// # Errors
/// A sentence when a list cannot be read.
pub fn owner(root: &Path, path: &Path) -> Result<Option<Owner>, String> {
    let Ok(target) = fs::canonicalize(path) else {
        return Ok(None);
    };
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_owned());
    let store = ManifestStore::new(&root);
    for name in store.names().map_err(|e| format!("{e:?}"))? {
        let Some(manifest) = store.load(&name).map_err(|e| format!("{name}: {e:?}"))? else {
            continue;
        };
        let dir = package_dir(&root, &manifest.name, &manifest.version);
        let Ok(rel) = target.strip_prefix(&dir) else {
            continue;
        };
        let rel = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if let Some(file) = manifest.files.iter().find(|f| f.path == rel).cloned() {
            let verdict = check_one(&target, &file);
            return Ok(Some(Owner {
                manifest,
                file,
                verdict,
            }));
        }
    }
    Ok(None)
}

fn check_one(path: &Path, file: &FileRecord) -> Verdict {
    crate::own_judge::judge(path, file)
}

/// Additional cross-check on Linux for a file of a system package: `rpm -qf`, else `dpkg -S`. `None` when neither knows it
/// or the tool is not there. Never required, never believed over our own list.
#[must_use]
pub fn system_owner(path: &Path) -> Option<String> {
    let run = |program: &str, args: &[&str]| {
        let out = std::process::Command::new(program)
            .args(args)
            .arg(path)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            .filter(|t| !t.is_empty())
    };
    run("rpm", &["-qf", "--qf", "%{NAME} %{VERSION}-%{RELEASE}\\n"])
        .map(|t| format!("rpm: {t}"))
        .or_else(|| run("dpkg", &["-S"]).map(|t| format!("dpkg: {t}")))
}
