//! The whole path of one install: signed catalog, release file, signature, checksum, install with rollback.
//! Pattern: Facade over the traits of the core; downloading is a Strategy (`Fetch`) so tests need no network.

use std::fs;
use std::path::PathBuf;

use fs_store_core::{
    Format, InstallState, RunningCheck, SignaturePolicy, SignatureVerifier, UpdateError, Version,
};

/// Gets the bytes behind an address.
pub trait Fetch: Sync {
    /// # Errors
    /// Text describing why the address could not be read.
    fn get(&self, url: &str) -> Result<Vec<u8>, String>;
}

/// Reads `file://` addresses and plain paths (local catalogs and draft releases).
#[derive(Debug, Default)]
pub struct FileFetch;

impl Fetch for FileFetch {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        let path = url.strip_prefix("file://").unwrap_or(url);
        fs::read(path).map_err(|e| format!("{url}: {e}"))
    }
}

/// Why an install did not happen.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallFailure {
    /// Fetching failed.
    Fetch(String),
    /// The catalog signature is not valid for the trusted key: nothing in the catalog is believed.
    CatalogSignature,
    /// The catalog text is unusable.
    Catalog(String),
    /// The catalog offers an older version than the installed one (replayed catalog).
    Downgrade,
    /// No such package offered.
    NotOffered,
    /// The format has no installer yet.
    Unsupported(Format),
    /// Checksum differs from the catalog.
    Checksum,
    /// Signature of the release file is not valid for the trusted key.
    ReleaseSignature,
    /// The update step failed or was rolled back.
    Update(UpdateError),
}

/// Installs from a catalog that is already on disk or reachable through `Fetch`.
pub struct Installer<'a> {
    /// Where packages live.
    pub root: PathBuf,
    /// Signature check.
    pub verifier: &'a dyn SignatureVerifier,
    /// Downloads.
    pub fetch: &'a dyn Fetch,
    /// Running check.
    pub running: &'a dyn RunningCheck,
    /// Which format to prefer when a release has several (the store's own install mode, [`crate::preferred_format`]):
    /// `Rpm` or `Deb` for the package of this system, `Exe` on Windows (answered with `Unsupported`), else `TarGz`.
    pub prefer: Format,
    /// Who may sign; `SignaturePolicy::default()` trusts only the FreeSynergy release key.
    pub policy: SignaturePolicy,
}

impl Installer<'_> {
    /// Verifies the catalog, then installs or updates `name`.
    ///
    /// # Errors
    /// `InstallFailure`; on every error before the update step nothing on disk was changed.
    pub fn install(
        &self,
        catalog: &[u8],
        catalog_sig: &[u8],
        name: &str,
        previous: Option<Version>,
    ) -> Result<Version, InstallFailure> {
        self.install_observed(catalog, catalog_sig, name, previous, &mut |_| {})
    }

    /// Like [`Installer::install`], and tells `observe` the stages passed: `Verified` once the catalog,
    /// checksum and release signature are good, `Installing` just before the update step. The end
    /// (`Installed`, or `Failed`) is the return value.
    ///
    /// # Errors
    /// `InstallFailure`, as for `install`.
    pub fn install_observed(
        &self,
        catalog: &[u8],
        catalog_sig: &[u8],
        name: &str,
        previous: Option<Version>,
        observe: &mut dyn FnMut(InstallState),
    ) -> Result<Version, InstallFailure> {
        let plan = self.plan(catalog, catalog_sig, name, previous)?;
        let downloaded = self.download(&plan)?;
        let prepared = self.check(&plan, downloaded)?;
        observe(InstallState::Verified);
        observe(InstallState::Installing);
        let mut done = self.commit_all(vec![prepared]);
        done.pop()
            .map_or(Err(InstallFailure::NotOffered), |(_, r)| r)
    }
}

/// With `FS_STORE_TIMING=1` the time of each step goes to stderr, to find out where a slow install spends it.
pub(crate) fn timing(step: &str, what: &str, since: std::time::Instant) {
    if std::env::var_os("FS_STORE_TIMING").is_some() {
        eprintln!(
            "timing {step:<9} {:>7.2}s  {what}",
            since.elapsed().as_secs_f64()
        );
    }
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

impl std::fmt::Display for InstallFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(what) => write!(f, "could not download: {what}"),
            Self::CatalogSignature => {
                f.write_str("the catalog is not signed by a trusted key; nothing in it is believed")
            }
            Self::Catalog(what) => write!(f, "the catalog is not usable: {what}"),
            Self::Downgrade => {
                f.write_str("the catalog offers an older version than the installed one (refused)")
            }
            Self::NotOffered => f.write_str("not offered by this source"),
            Self::Unsupported(Format::Exe) => f.write_str(
                "Unsupported: the Windows setup.exe cannot be installed by the store yet (the store itself does not run on Windows)",
            ),
            Self::Unsupported(format) => write!(f, "no installer for the format {format:?} yet"),
            Self::Checksum => {
                f.write_str("the downloaded file does not match the checksum in the catalog")
            }
            Self::ReleaseSignature => {
                f.write_str("the downloaded file is not signed by a trusted key")
            }
            Self::Update(UpdateError::RolledBack(cause)) => {
                write!(f, "installing failed and was rolled back: {}", cause.0)
            }
            Self::Update(e) => write!(f, "installing failed: {e:?}"),
        }
    }
}
