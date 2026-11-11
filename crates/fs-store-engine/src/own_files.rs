//! Pattern: Facade over the store's own list of files (`ManifestStore`): write it at install, check it (`verify`), find the
//! package of a path (`owner`), and make a missing list from the tree that is there (migration). The list is the truth for
//! "what we installed"; `rpm -qf` and `dpkg -S` are only an additional cross-check for files of system packages
//! (`system_owner` in `owner.rs`), never required.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs_store_sources::{Basis, FileRecord, Manifest, ManifestStore};
use sha2::{Digest, Sha256};

const SCHEMA: u32 = 1;

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

pub(crate) fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut sum = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sum.update(&buf[..n]);
    }
    Ok(format!("sha256:{}", hex(&sum.finalize())))
}

#[cfg(unix)]
fn mode_of(meta: &fs::Metadata) -> u32 {
    std::os::unix::fs::PermissionsExt::mode(&meta.permissions()) & 0o7777
}

#[cfg(not(unix))]
fn mode_of(_meta: &fs::Metadata) -> u32 {
    0
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<FileRecord>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        if meta.is_dir() {
            walk(base, &entry.path(), out)?;
        } else if meta.is_file() {
            let rel = entry
                .path()
                .strip_prefix(base)
                .map_err(std::io::Error::other)?
                .to_owned();
            out.push(FileRecord {
                path: rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/"),
                hash: hash_file(&entry.path())?,
                size: meta.len(),
                mode: mode_of(&meta),
            });
        }
    }
    Ok(())
}

/// Every regular file below `dir` with its checksum, sorted by path. Links and special files are not part of a package.
///
/// # Errors
/// `io::Error` when a file cannot be read.
pub fn collect(dir: &Path) -> std::io::Result<Vec<FileRecord>> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub(crate) fn package_dir(root: &Path, name: &str, version: &str) -> PathBuf {
    root.join("packages").join(name).join(version)
}

fn make(
    root: &Path,
    name: &str,
    version: &str,
    source: &str,
    signer: &str,
    basis: Basis,
) -> Result<Manifest, String> {
    let files = collect(&package_dir(root, name, version))
        .map_err(|e| format!("{name}: files not listed: {e}"))?;
    Ok(Manifest {
        schema: SCHEMA,
        name: name.to_owned(),
        version: version.to_owned(),
        source: source.to_owned(),
        signer: signer.to_owned(),
        basis,
        recorded_at: now(),
        files,
    })
}

/// Writes the list of the files the store just unpacked for `name` `version`. A package that has no unpacked directory
/// (rpm, deb: the package manager owns the files) gets no list and the answer is `Ok(false)`.
///
/// # Errors
/// A sentence when the files cannot be read or the list cannot be written.
pub fn record_install(
    root: &Path,
    name: &str,
    version: &str,
    source: &str,
    signer: &str,
) -> Result<bool, String> {
    if !package_dir(root, name, version).is_dir() {
        return Ok(false);
    }
    let manifest = make(root, name, version, source, signer, Basis::Installed)?;
    ManifestStore::new(root)
        .save(&manifest)
        .map_err(|e| format!("{name}: the list of files was not saved: {e:?}"))?;
    Ok(true)
}

/// What a file is now, against the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Same checksum.
    Intact,
    /// Another checksum: the file was changed after the install.
    Changed,
    /// The file is gone.
    Missing,
}

/// The check of one package.
#[derive(Debug)]
pub struct Report {
    /// The list that was checked.
    pub manifest: Manifest,
    /// Each file with its verdict, in the order of the list.
    pub files: Vec<(String, Verdict)>,
    /// The list was made just now from the tree (migration): changes before now cannot be seen.
    pub made_now: bool,
}

impl Report {
    /// Nothing changed and nothing is missing.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.files.iter().all(|(_, v)| *v == Verdict::Intact)
    }
}

fn current_versions(root: &Path) -> Vec<(String, String)> {
    let Ok(dir) = fs::read_dir(root.join("current")) else {
        return Vec::new();
    };
    let mut out: Vec<(String, String)> = dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_owned();
            let target = fs::read_link(e.path()).ok()?;
            let version = target.file_name()?.to_str()?.to_owned();
            (!name.starts_with('.')).then_some((name, version))
        })
        .collect();
    out.sort();
    out
}

/// Makes the list of every installed package that has none (an installation older than the list), from the tree on disk,
/// marked `Retroactive`. Returns the names made.
///
/// # Errors
/// A sentence for the first package that could not be listed.
pub fn adopt_missing(
    root: &Path,
    source_of: &dyn Fn(&str) -> String,
) -> Result<Vec<String>, String> {
    let store = ManifestStore::new(root);
    let mut made = Vec::new();
    for (name, version) in current_versions(root) {
        let known = store.load(&name).map_err(|e| format!("{name}: {e:?}"))?;
        if known.is_some_and(|m| m.version == version) {
            continue;
        }
        let manifest = make(
            root,
            &name,
            &version,
            &source_of(&name),
            "",
            Basis::Retroactive,
        )?;
        store
            .save(&manifest)
            .map_err(|e| format!("{name}: {e:?}"))?;
        made.push(name);
    }
    Ok(made)
}

fn check(root: &Path, manifest: &Manifest) -> Vec<(String, Verdict)> {
    let dir = package_dir(root, &manifest.name, &manifest.version);
    manifest
        .files
        .iter()
        .map(|f| {
            let verdict = match hash_file(&dir.join(&f.path)) {
                Ok(h) if h == f.hash => Verdict::Intact,
                Ok(_) => Verdict::Changed,
                Err(_) => Verdict::Missing,
            };
            (f.path.clone(), verdict)
        })
        .collect()
}

/// Checks our own installations against their lists (`only` = one package). A package without a list gets one first,
/// marked as made afterwards. Only what the store installed is checked.
///
/// # Errors
/// A sentence when a list cannot be read or made.
pub fn verify(
    root: &Path,
    only: Option<&str>,
    source_of: &dyn Fn(&str) -> String,
) -> Result<Vec<Report>, String> {
    let made = adopt_missing(root, source_of)?;
    let store = ManifestStore::new(root);
    let mut reports = Vec::new();
    for name in store.names().map_err(|e| format!("{e:?}"))? {
        if only.is_some_and(|o| o != name) {
            continue;
        }
        let Some(manifest) = store.load(&name).map_err(|e| format!("{name}: {e:?}"))? else {
            continue;
        };
        reports.push(Report {
            files: check(root, &manifest),
            made_now: made.contains(&name),
            manifest,
        });
    }
    if let Some(o) = only.filter(|_| reports.is_empty()) {
        return Err(format!("{o} is not installed by this store"));
    }
    Ok(reports)
}
