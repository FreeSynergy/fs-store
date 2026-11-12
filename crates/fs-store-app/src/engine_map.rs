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
        InstallFailure::NotInstallable(reason) => {
            JobError::Other(format!("this store version does not install it: {reason}"))
        }
        InstallFailure::CatalogSignature => JobError::CatalogSignature,
        InstallFailure::NotOffered => JobError::NotOffered,
        InstallFailure::Downgrade => JobError::Downgrade,
        InstallFailure::Unsupported(format) => JobError::Unsupported(format!("{format:?}")),
        InstallFailure::Checksum => JobError::Checksum,
        InstallFailure::ReleaseSignature => JobError::ReleaseSignature,
        InstallFailure::Cancelled => JobError::Cancelled,
        InstallFailure::Locked => JobError::Other(
            "another store process is installing it right now; try again when it is done"
                .to_owned(),
        ),
        InstallFailure::Update(UpdateError::Running) => JobError::Running,
        InstallFailure::Update(UpdateError::RolledBack(cause)) => {
            JobError::Other(format!("the installer failed: {}", cause.0))
        }
        InstallFailure::Update(UpdateError::RollbackFailed) => JobError::RollbackFailed,
        InstallFailure::ServerNotEnded { version, reason } => {
            JobError::ServerNotEnded { version, reason }
        }
    }
}
