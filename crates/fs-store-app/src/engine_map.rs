//! Pattern: Adapter (small conversions from the engine's types to the window's).

use fs_store_core::UpdateError;
use fs_store_engine::service::InstallFailure;

use crate::backend::{JobError, SourceRow};

pub(crate) fn row(s: fs_store_core::Source) -> SourceRow {
    SourceRow {
        id: s.id.0,
        url: s.url,
        enabled: s.enabled,
        reference: s.reference,
    }
}

pub(crate) fn id_from_url(url: &str) -> String {
    url.trim_end_matches(['/', '.'])
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("source")
        .trim_end_matches(".git")
        .to_owned()
}

pub(crate) fn job_error(failure: InstallFailure) -> JobError {
    match failure {
        InstallFailure::Fetch(text) | InstallFailure::Catalog(text) => JobError::Other(text),
        InstallFailure::CatalogSignature => JobError::CatalogSignature,
        InstallFailure::NotOffered => JobError::NotOffered,
        InstallFailure::Downgrade => JobError::Downgrade,
        InstallFailure::Unsupported(format) => JobError::Unsupported(format!("{format:?}")),
        InstallFailure::Checksum => JobError::Checksum,
        InstallFailure::ReleaseSignature => JobError::ReleaseSignature,
        InstallFailure::Update(UpdateError::Running) => JobError::Running,
        InstallFailure::Update(UpdateError::RolledBack) => {
            JobError::Other("the installer failed".to_owned())
        }
        InstallFailure::Update(UpdateError::RollbackFailed) => JobError::RollbackFailed,
    }
}
