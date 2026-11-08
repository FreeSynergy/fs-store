//! The first half of an install: plan from the signed catalog, download, check. Nothing on the machine changes.
//! Pattern: Unit of Work in two steps (prepare, then commit in `batch.rs`); the download is the only part that
//! waits for the network, so a batch runs it for all packages at once.

use fs_store_catalog::{CatalogEntry, parse_catalog};
use fs_store_core::{Format, Package, SignatureOutcome, SourceId, VerifiedPackage, Version};
use sha2::{Digest, Sha256};

use crate::service::{InstallFailure, Installer, hex, timing};

/// What to fetch for one package, decided from the verified catalog.
#[derive(Debug)]
pub struct Plan {
    entry: CatalogEntry,
    previous: Option<Version>,
    format: Format,
    /// (location, signature address, sha256) of each file.
    wanted: Vec<(String, String, String)>,
}

impl Plan {
    /// The name of the package.
    #[must_use]
    pub fn name(&self) -> &str {
        self.entry.name.as_str()
    }
}

/// Downloaded files with their signatures, not checked yet.
#[derive(Debug)]
pub struct Downloaded(Vec<(Vec<u8>, Vec<u8>)>);

/// A package that passed every check and waits to be installed.
#[derive(Debug)]
pub struct Prepared {
    pub(crate) verified: VerifiedPackage,
    pub(crate) previous: Option<Version>,
    pub(crate) files: Vec<Vec<u8>>,
}

impl Prepared {
    /// The name of the package.
    #[must_use]
    pub fn name(&self) -> &str {
        self.verified.package().name.as_str()
    }

    /// The version that will be installed.
    #[must_use]
    pub fn version(&self) -> Version {
        self.verified.package().version
    }
}

impl Installer<'_> {
    /// Verifies the catalog and decides which files `name` needs.
    ///
    /// # Errors
    /// `InstallFailure` before anything was downloaded.
    pub fn plan(
        &self,
        catalog: &[u8],
        catalog_sig: &[u8],
        name: &str,
        previous: Option<Version>,
    ) -> Result<Plan, InstallFailure> {
        if !self
            .policy
            .is_satisfied_by(&self.verifier.verify(catalog, Some(catalog_sig)))
        {
            return Err(InstallFailure::CatalogSignature);
        }
        let text =
            std::str::from_utf8(catalog).map_err(|e| InstallFailure::Catalog(e.to_string()))?;
        let parsed = parse_catalog(text).map_err(|e| InstallFailure::Catalog(e.0))?;
        let entry = parsed
            .entries
            .into_iter()
            .find(|e| e.name.as_str() == name)
            .ok_or(InstallFailure::NotOffered)?;
        if !matches!(entry.format, Format::TarGz | Format::Rpm) {
            return Err(InstallFailure::Unsupported(entry.format));
        }
        if previous.is_some_and(|p| entry.version < p) {
            return Err(InstallFailure::Downgrade);
        }
        // rpm when the store itself came as rpm and the release has rpm files; else the `tar.gz`.
        let (format, wanted) = if self.prefer == Format::Rpm && !entry.rpms.is_empty() {
            let all = entry.rpms.iter().map(|r| {
                (
                    r.location.clone(),
                    r.signature_url.clone(),
                    r.sha256.clone(),
                )
            });
            (Format::Rpm, all.collect())
        } else {
            (
                entry.format,
                vec![(
                    entry.location.clone(),
                    entry.signature_url.clone(),
                    entry.sha256.clone(),
                )],
            )
        };
        Ok(Plan {
            entry,
            previous,
            format,
            wanted,
        })
    }

    /// Downloads the files of the plan.
    ///
    /// # Errors
    /// `Fetch` when an address cannot be read.
    pub fn download(&self, plan: &Plan) -> Result<Downloaded, InstallFailure> {
        let mut all = Vec::new();
        for (location, signature_url, _) in &plan.wanted {
            let t = std::time::Instant::now();
            let file = self.fetch.get(location).map_err(InstallFailure::Fetch)?;
            let sig = self
                .fetch
                .get(signature_url)
                .map_err(InstallFailure::Fetch)?;
            timing("download", location, t);
            all.push((file, sig));
        }
        Ok(Downloaded(all))
    }

    /// Checks sums and signatures of what was downloaded.
    ///
    /// # Errors
    /// `Checksum` or `ReleaseSignature`.
    pub fn check(&self, plan: &Plan, downloaded: Downloaded) -> Result<Prepared, InstallFailure> {
        let mut files = Vec::new();
        let mut outcome = SignatureOutcome::Missing;
        for ((location, _, sha256), (file, sig)) in plan.wanted.iter().zip(downloaded.0) {
            let t = std::time::Instant::now();
            if hex(&Sha256::digest(&file)) != sha256.to_ascii_lowercase() {
                return Err(InstallFailure::Checksum);
            }
            timing("sha256", location, t);
            let t = std::time::Instant::now();
            outcome = self.verifier.verify(&file, Some(&sig));
            timing("signature", location, t);
            if !self.policy.is_satisfied_by(&outcome) {
                return Err(InstallFailure::ReleaseSignature);
            }
            files.push(file);
        }
        let package = Package {
            name: plan.entry.name.clone(),
            kind: plan.entry.kind,
            version: plan.entry.version,
            format: plan.format,
            source: SourceId("catalog".to_owned()),
        };
        let verified = self
            .policy
            .admit(package, outcome)
            .map_err(|_| InstallFailure::ReleaseSignature)?;
        Ok(Prepared {
            verified,
            previous: plan.previous,
            files,
        })
    }
}
