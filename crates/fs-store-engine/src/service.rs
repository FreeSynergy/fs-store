//! The whole path of one install: signed catalog, release file, signature, checksum, install with rollback.
//! Pattern: Facade over the traits of the core; downloading is a Strategy (`Fetch`) so tests need no network.

use std::fs;
use std::path::PathBuf;

use fs_store_catalog::{CatalogEntry, parse_catalog};
use fs_store_core::{
    Format, Package, RunningCheck, SignaturePolicy, SignatureVerifier, SourceId, Update,
    UpdateError, Version, run_update,
};
use sha2::{Digest, Sha256};

use crate::TarGz;

/// Gets the bytes behind an address.
pub trait Fetch {
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
    /// Staging directory for downloads.
    pub staging: PathBuf,
    /// Signature check.
    pub verifier: &'a dyn SignatureVerifier,
    /// Downloads.
    pub fetch: &'a dyn Fetch,
    /// Running check.
    pub running: &'a dyn RunningCheck,
    /// Who may sign; `SignaturePolicy::default()` trusts only the FreeSynergy release key.
    pub policy: SignaturePolicy,
}

impl Installer<'_> {
    /// Verifies the catalog, then installs or updates `name`.
    ///
    /// # Errors
    /// `InstallFailure`; on every error before the update step nothing on disk was changed besides staging.
    pub fn install(
        &self,
        catalog: &[u8],
        catalog_sig: &[u8],
        name: &str,
        previous: Option<Version>,
    ) -> Result<Version, InstallFailure> {
        let policy = &self.policy;
        if !policy.is_satisfied_by(&self.verifier.verify(catalog, Some(catalog_sig))) {
            return Err(InstallFailure::CatalogSignature);
        }
        let text =
            std::str::from_utf8(catalog).map_err(|e| InstallFailure::Catalog(e.to_string()))?;
        let parsed = parse_catalog(text).map_err(|e| InstallFailure::Catalog(e.0))?;
        let entry: &CatalogEntry = parsed
            .entries
            .iter()
            .find(|e| e.name.as_str() == name)
            .ok_or(InstallFailure::NotOffered)?;
        if entry.format != Format::TarGz {
            return Err(InstallFailure::Unsupported(entry.format));
        }
        let file = self
            .fetch
            .get(&entry.location)
            .map_err(InstallFailure::Fetch)?;
        let sig = self
            .fetch
            .get(&entry.signature_url)
            .map_err(InstallFailure::Fetch)?;
        if hex(&Sha256::digest(&file)) != entry.sha256.to_ascii_lowercase() {
            return Err(InstallFailure::Checksum);
        }
        let outcome = self.verifier.verify(&file, Some(&sig));
        let package = Package {
            name: entry.name.clone(),
            kind: entry.kind,
            version: entry.version,
            format: entry.format,
            source: SourceId("catalog".to_owned()),
        };
        let verified = policy
            .admit(package, outcome)
            .map_err(|_| InstallFailure::ReleaseSignature)?;
        fs::create_dir_all(&self.staging).map_err(|e| InstallFailure::Fetch(e.to_string()))?;
        let staged = self
            .staging
            .join(format!("{name}-{}.tar.gz", entry.version));
        fs::write(&staged, &file).map_err(|e| InstallFailure::Fetch(e.to_string()))?;
        let mut strategy = TarGz::new(&self.root, &staged);
        let result = run_update(
            &Update {
                package: &verified,
                previous,
            },
            &mut strategy,
            self.running,
        );
        let _ = fs::remove_file(&staged);
        result
            .map(|_| entry.version)
            .map_err(InstallFailure::Update)
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}
