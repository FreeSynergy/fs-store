// init.rs — BootableInstaller trait for the FreeSynergy Init package.
//
// The `init/` directory in the Store is NOT a regular package. It is the
// bootstrap installer meant to be written to a USB drive for first-time
// installation. It gets its own trait, separate from Package.

use std::path::Path;

// ── InitFile ──────────────────────────────────────────────────────────────────

/// A single file included in the bootable installer image.
#[derive(Debug, Clone)]
pub struct InitFile {
    /// Store-relative path of the source file.
    pub store_path: String,

    /// Target path within the installer image.
    pub target_path: String,
}

// ── BootableInstaller trait ───────────────────────────────────────────────────

/// The bootable USB installer for FreeSynergy.
///
/// Not a `Package` — has no catalog entry in the normal namespace. It is
/// fetched via `installer::init` Bus namespace and built locally.
pub trait BootableInstaller: Send + Sync {
    /// Current version of the installer.
    fn version(&self) -> &str;

    /// Files included in the installer image.
    fn files(&self) -> &[InitFile];

    /// Direct download URL for the pre-built image.
    fn download_url(&self) -> &str;

    /// Build a bootable image from the local Store data at `target_path`.
    ///
    /// `target_path` may point to a USB device (e.g. `/dev/sdb`) or a file
    /// (e.g. `/tmp/freesynergy-init.img`).
    fn build_image(&self, target: &Path) -> anyhow::Result<()>;
}
