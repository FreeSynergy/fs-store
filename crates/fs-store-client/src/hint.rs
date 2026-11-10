//! Pattern: Facade (the one question). See the crate documentation.

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

/// The store's folder: `FS_STORE_ROOT`, else the folder of this system (as for the store itself).
#[must_use]
pub fn store_root() -> PathBuf {
    fs_store_platform::current().paths.store_root()
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
