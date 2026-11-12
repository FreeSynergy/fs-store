//! Reading the signed catalogs of the enabled sources, as `fs-store available` does.
//!
//! Pattern: Template method — per source: clone, check the signature against the trusted keys,
//! parse, check freshness (a replayed old catalog is refused). Nothing of an unverified catalog is
//! believed; a source that fails answers as one skipped entry, never silently.

use std::path::Path;

use fs_store_catalog::parse_catalog;
use fs_store_core::{SignaturePolicy, SignatureVerifier, Source};
use fs_store_engine::net::clone_catalog;
use fs_store_pgp::PgpVerifier;

use crate::view::{Offers, PackageInfo, SkippedInfo};

/// Offers of all `sources`, and what was left out.
pub(crate) fn read_all(root: &Path, sources: &[Source]) -> Offers {
    let verifier = PgpVerifier::builtin();
    let policy = SignaturePolicy::default();
    let mut packages = Vec::new();
    let mut skipped = Vec::new();
    for source in sources {
        match read_one(root, source, &verifier, &policy) {
            Ok((p, s)) => {
                packages.extend(p);
                skipped.extend(s);
            }
            Err(reason) => skipped.push(SkippedInfo {
                id: source.id.0.clone(),
                reason,
            }),
        }
    }
    (packages, skipped)
}

fn read_one(
    root: &Path,
    source: &Source,
    verifier: &PgpVerifier,
    policy: &SignaturePolicy,
) -> Result<Offers, String> {
    let work = root
        .join("sources")
        .join(format!("serve-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(work.parent().unwrap_or(root)).map_err(|e| e.to_string())?;
    let read = fetch(&work, &source.url, verifier, policy);
    let _ = std::fs::remove_dir_all(&work);
    let catalog = parse_catalog(&read?).map_err(|e| e.0)?;
    fs_store_sources::ensure_fresh(
        &root.join("serials.toml"),
        &source.url,
        catalog.serial,
        catalog.expires,
    )?;
    let packages = catalog
        .entries
        .into_iter()
        .map(|e| PackageInfo {
            name: e.name.as_str().to_owned(),
            kind: e.kind,
            version: e.version.to_string(),
            format: e.format,
            source: source.id.0.clone(),
            facets: e.facets,
            summary: e.summary,
            icon_sha256: e
                .icon
                .map(|i| i.sha256.to_ascii_lowercase())
                .unwrap_or_default(),
            details: e.details,
        })
        .collect();
    let skipped = catalog
        .skipped
        .into_iter()
        .map(|s| SkippedInfo {
            id: s.id,
            reason: s.reason,
        })
        .collect();
    Ok((packages, skipped))
}

fn fetch(
    work: &Path,
    url: &str,
    verifier: &PgpVerifier,
    policy: &SignaturePolicy,
) -> Result<String, String> {
    clone_catalog(url, work).map_err(|e| format!("{url}: {e}"))?;
    let catalog = std::fs::read(work.join("catalog.xml")).map_err(|e| format!("{url}: {e}"))?;
    let sig = std::fs::read(work.join("catalog.xml.asc")).map_err(|e| format!("{url}: {e}"))?;
    if !policy.is_satisfied_by(&verifier.verify(&catalog, Some(&sig))) {
        return Err(format!(
            "{url}: the catalog signature is not valid for a trusted key"
        ));
    }
    Ok(String::from_utf8_lossy(&catalog).into_owned())
}
