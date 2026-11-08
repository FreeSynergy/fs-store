//! The whole path of one install: signed catalog, release file, signature, checksum, install with rollback.
//! Pattern: Facade over the traits of the core; downloading is a Strategy (`Fetch`) so tests need no network.

use std::fs;
use std::path::PathBuf;

use fs_store_catalog::{CatalogEntry, parse_catalog};
use fs_store_core::{
    Format, InstallState, InstallStrategy, Package, RunningCheck, SignatureOutcome,
    SignaturePolicy, SignatureVerifier, SourceId, Update, UpdateError, Version, run_update,
};
use sha2::{Digest, Sha256};

use crate::{Rpm, SystemCommander, TarGz};

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
    /// Which format to prefer when a release has several (the store's own install mode, [`crate::preferred_format`]).
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
        if !matches!(entry.format, Format::TarGz | Format::Rpm) {
            return Err(InstallFailure::Unsupported(entry.format));
        }
        if previous.is_some_and(|p| entry.version < p) {
            return Err(InstallFailure::Downgrade);
        }
        // rpm when the store itself came as rpm and the release has rpm files; else the `tar.gz`.
        let use_rpm = self.prefer == Format::Rpm && !entry.rpms.is_empty();
        let (format, wanted): (Format, Vec<(&str, &str, &str)>) = if use_rpm {
            let all = entry.rpms.iter();
            (
                Format::Rpm,
                all.map(|r| {
                    (
                        r.location.as_str(),
                        r.signature_url.as_str(),
                        r.sha256.as_str(),
                    )
                })
                .collect(),
            )
        } else {
            (
                entry.format,
                vec![(
                    entry.location.as_str(),
                    entry.signature_url.as_str(),
                    entry.sha256.as_str(),
                )],
            )
        };
        let mut files = Vec::new();
        let mut outcome = SignatureOutcome::Missing;
        for (location, signature_url, sha256) in wanted {
            let t = std::time::Instant::now();
            let file = self.fetch.get(location).map_err(InstallFailure::Fetch)?;
            let sig = self
                .fetch
                .get(signature_url)
                .map_err(InstallFailure::Fetch)?;
            timing("download", location, t);
            let t = std::time::Instant::now();
            if hex(&Sha256::digest(&file)) != sha256.to_ascii_lowercase() {
                return Err(InstallFailure::Checksum);
            }
            timing("sha256", location, t);
            let t = std::time::Instant::now();
            outcome = self.verifier.verify(&file, Some(&sig));
            timing("signature", location, t);
            if !policy.is_satisfied_by(&outcome) {
                return Err(InstallFailure::ReleaseSignature);
            }
            files.push(file);
        }
        let package = Package {
            name: entry.name.clone(),
            kind: entry.kind,
            version: entry.version,
            format,
            source: SourceId("catalog".to_owned()),
        };
        let verified = policy
            .admit(package, outcome)
            .map_err(|_| InstallFailure::ReleaseSignature)?;
        observe(InstallState::Verified);
        // The verified bytes go straight to the unpacker; no file others could swap in between.
        let mut strategy: Box<dyn InstallStrategy> = if format == Format::Rpm {
            Box::new(Rpm::new(&self.root.join("rpm"), files, SystemCommander))
        } else {
            Box::new(TarGz::from_bytes(&self.root, files.remove(0)))
        };
        observe(InstallState::Installing);
        let t = std::time::Instant::now();
        let result = run_update(
            &Update {
                package: &verified,
                previous,
            },
            strategy.as_mut(),
            self.running,
        );
        timing("install", name, t);
        result
            .map(|_| entry.version)
            .map_err(InstallFailure::Update)
    }
}

/// With `FS_STORE_TIMING=1` the time of each step goes to stderr, to find out where a slow install spends it.
fn timing(step: &str, what: &str, since: std::time::Instant) {
    if std::env::var_os("FS_STORE_TIMING").is_some() {
        eprintln!(
            "timing {step:<9} {:>7.2}s  {what}",
            since.elapsed().as_secs_f64()
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}
