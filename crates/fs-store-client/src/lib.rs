//! Pattern: Facade (one question: "is there an update for me?"). Decision (provisional, 2026-11-08): programs read the
//! result of the store's background check (`updates.toml`, written by `fs-store background`) instead of calling the
//! server `fs-store serve`: no process has to run, nothing is downloaded, and the one place that checks and downloads
//! stays the store. When `fs-store serve` can know its callers (ADR 0054) the same question can go through `store.capnp`.
//!
//! A program shows the hint, for example "A new version of this program is available (0.0.2 -> 0.0.3). Open the
//! store to update." and never downloads by itself. The person can switch hints off in the store (`notify off`).
#![deny(missing_docs)]

use std::path::{Path, PathBuf};

use fs_store_sources::{Findings, Settings};

/// A newer version of the asking program is on offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateHint {
    /// The program's package name.
    pub program: String,
    /// The installed version.
    pub from: String,
    /// The offered version.
    pub to: String,
}

impl UpdateHint {
    /// One plain sentence for a status line or notice.
    #[must_use]
    pub fn sentence(&self) -> String {
        format!(
            "A new version of {} is available ({} -> {}). Open the store to update.",
            self.program, self.from, self.to
        )
    }
}

/// The store's folder: `FS_STORE_ROOT`, else `~/.local/share/fs-store` (as for the store itself).
#[must_use]
pub fn store_root() -> PathBuf {
    std::env::var_os("FS_STORE_ROOT").map_or_else(
        || {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/share/fs-store")
        },
        PathBuf::from,
    )
}

/// Is there an update for `program`? `None` when there is none, the person switched hints off, or the store has
/// not checked yet (a missing or unreadable file never makes a program fail).
#[must_use]
pub fn update_hint(program: &str) -> Option<UpdateHint> {
    update_hint_in(&store_root(), program)
}

/// Like [`update_hint`], for a given store folder.
#[must_use]
pub fn update_hint_in(root: &Path, program: &str) -> Option<UpdateHint> {
    let settings = Settings::load(&root.join("settings.toml")).ok()?;
    if !settings.notify {
        return None;
    }
    let findings = Findings::load(&root.join("updates.toml")).ok()?;
    findings
        .update
        .iter()
        .find(|u| u.name == program)
        .map(|u| UpdateHint {
            program: u.name.clone(),
            from: u.from.clone(),
            to: u.to.clone(),
        })
}
