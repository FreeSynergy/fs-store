//! Pattern: State (how the job of one package ends: installed, installed with a note, or failed and rolled back).

use fs_store_core::Version;

use super::store::StoreModel;
use super::types::Problem;
use crate::backend::{JobError, Phase};

impl StoreModel {
    pub(super) fn fail_job(&mut self, name: &str, error: JobError) {
        let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) else {
            return;
        };
        job.history.push(Phase::Failed);
        job.phase = Phase::Failed;
        if error != JobError::RollbackFailed {
            job.history.push(Phase::RolledBack);
            job.phase = Phase::RolledBack;
        }
        self.problems.push(Problem::JobFailed {
            name: job.name.clone(),
            error: error.clone(),
        });
        job.error = Some(error);
    }

    pub(super) fn end_job(&mut self, name: &str, result: Result<Version, JobError>) {
        match result {
            Ok(version) => {
                if let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) {
                    job.version = Some(version);
                    job.phase = Phase::Installed;
                    job.history.push(Phase::Installed);
                }
                self.updated(name);
            }
            // Installed, only the old server stays: the job is done, and the person is told (never silent).
            Err(JobError::ServerNotEnded { version, reason }) => {
                if let Some(job) = self.jobs.iter_mut().find(|j| j.name == name) {
                    job.version = Some(version);
                    job.phase = Phase::Installed;
                    job.history.push(Phase::Installed);
                    self.problems.push(Problem::JobFailed {
                        name: job.name.clone(),
                        error: JobError::ServerNotEnded { version, reason },
                    });
                }
                self.updated(name);
            }
            Err(error) => self.fail_job(name, error),
        }
    }
}
