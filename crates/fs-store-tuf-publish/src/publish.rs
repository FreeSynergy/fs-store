//! Pattern: Adapter (`tough`'s editor behind the publishing functions); see the crate documentation.
use std::fmt;
use std::future::Future;
use std::num::NonZeroU64;
use std::path::Path;
use std::time::Duration;

use jiff::Timestamp;
use tough::editor::RepositoryEditor;
use tough::editor::signed::PathExists;
use tough::key_source::{KeySource, LocalKeySource};

/// Why publishing failed (the text says which step).
#[derive(Debug)]
pub struct PublishError(pub String);

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PublishError {}

pub(crate) fn fail<E: fmt::Display>(step: &'static str) -> impl FnOnce(E) -> PublishError {
    move |e| PublishError(format!("{step}: {e}"))
}

/// How long each signed role stays valid.
#[derive(Debug, Clone, Copy)]
pub struct Expiries {
    /// Targets role, default 90 days.
    pub targets: Duration,
    /// Snapshot role, default 7 days.
    pub snapshot: Duration,
    /// Timestamp role, default 1 day.
    pub timestamp: Duration,
}

impl Default for Expiries {
    fn default() -> Self {
        let days = |d: u64| Duration::from_secs(d * 86_400);
        Self {
            targets: days(90),
            snapshot: days(7),
            timestamp: days(1),
        }
    }
}

/// The versions that were written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Versions {
    /// Version of `targets.json`.
    pub targets: u64,
    /// Version of `snapshot.json`.
    pub snapshot: u64,
    /// Version of `timestamp.json`.
    pub timestamp: u64,
}

pub(crate) fn block_on<F: Future>(f: F) -> Result<F::Output, PublishError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(fail("runtime"))?;
    Ok(rt.block_on(f))
}

/// The version after the one in `metadata/<role>.json` (1 when there is none yet).
fn next_version(metadata: &Path, role: &str) -> Result<NonZeroU64, PublishError> {
    let file = metadata.join(format!("{role}.json"));
    if !file.exists() {
        return Ok(NonZeroU64::MIN);
    }
    let bytes = std::fs::read(&file).map_err(fail("read existing metadata"))?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(fail("parse existing metadata"))?;
    let old = json["signed"]["version"]
        .as_u64()
        .ok_or_else(|| PublishError(format!("{}: no signed.version", file.display())))?;
    old.checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| PublishError(format!("{}: version overflow", file.display())))
}

fn expiry(from_now: Duration) -> Result<Timestamp, PublishError> {
    Timestamp::now()
        .checked_add(from_now)
        .map_err(fail("expiry"))
}

/// Signs and writes the TUF repository for `dir/catalog.xml` into `dir/tuf/`, with the trusted `dir/root.json` and the
/// keys `dir/{targets,snapshot,timestamp}.pk8`. Nothing in `dir/tuf` is changed when signing fails.
///
/// # Errors
/// `PublishError` when an input is missing, a key does not match the root, or a file cannot be written.
pub fn publish(dir: &Path, expiries: &Expiries) -> Result<Versions, PublishError> {
    let metadata = dir.join("tuf/metadata");
    let versions = Versions {
        targets: next_version(&metadata, "targets")?.get(),
        snapshot: next_version(&metadata, "snapshot")?.get(),
        timestamp: next_version(&metadata, "timestamp")?.get(),
    };
    let (t, s, ts) = (
        NonZeroU64::new(versions.targets),
        NonZeroU64::new(versions.snapshot),
        NonZeroU64::new(versions.timestamp),
    );
    let (Some(t), Some(s), Some(ts)) = (t, s, ts) else {
        return Err(PublishError("version zero".to_owned()));
    };
    let sources: Vec<Box<dyn KeySource>> = ["targets", "snapshot", "timestamp"]
        .iter()
        .map(|r| {
            Box::new(LocalKeySource {
                path: dir.join(format!("{r}.pk8")),
            }) as Box<dyn KeySource>
        })
        .collect();
    let (catalog, targets_out) = (dir.join("catalog.xml"), dir.join("tuf/targets"));
    let (te, se, tse) = (
        expiry(expiries.targets)?,
        expiry(expiries.snapshot)?,
        expiry(expiries.timestamp)?,
    );
    block_on(async {
        let mut editor = RepositoryEditor::new(dir.join("root.json"))
            .await
            .map_err(fail("root.json"))?;
        editor
            .targets_version(t)
            .map_err(fail("targets"))?
            .targets_expires(te)
            .map_err(fail("targets"))?
            .snapshot_version(s)
            .snapshot_expires(se)
            .timestamp_version(ts)
            .timestamp_expires(tse)
            .add_target_path(&catalog)
            .await
            .map_err(fail("catalog.xml"))?;
        let signed = editor.sign(&sources).await.map_err(fail("sign"))?;
        tokio::fs::create_dir_all(&targets_out)
            .await
            .map_err(fail("tuf/targets"))?;
        // The old catalog has another hash; remove it so the copy is not refused (a client refuses the gap until the
        // metadata below is written, which is the safe direction).
        match tokio::fs::remove_file(targets_out.join("catalog.xml")).await {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(fail("remove old catalog.xml")(e));
            }
            _ => {}
        }
        signed
            .copy_target(&catalog, &targets_out, PathExists::Replace, None)
            .await
            .map_err(fail("copy catalog.xml"))?;
        signed
            .write(&metadata)
            .await
            .map_err(fail("write metadata"))
    })??;
    Ok(versions)
}
