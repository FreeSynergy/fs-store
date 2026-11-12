//! Pattern: Command (what "Cancel" does to a job, and how far a download is): the model's side of the stop.

use fs_store_core::{CancelToken, Progress};

use crate::backend::Phase;

use super::store::StoreModel;

impl StoreModel {
    /// The backend reports how far the download of `name` is.
    pub fn observe_progress(&mut self, name: &str, progress: Progress) {
        if let Some(job) = self
            .jobs
            .iter_mut()
            .find(|j| j.name == name && !j.is_finished())
        {
            job.progress = Some(progress);
        }
    }

    /// The token "Cancel" sets, for the worker of the job `name`.
    #[must_use]
    pub fn job_cancel(&self, name: &str) -> Option<CancelToken> {
        self.jobs
            .iter()
            .find(|j| j.name == name && !j.is_finished())
            .map(|j| j.cancel.clone())
    }

    /// Can the job of `name` be stopped now with nothing left half done? Downloading, checking and waiting: yes. Installing: only
    /// the unpacked archive (it is unpacked beside the target and renamed, so a stop leaves nothing); a system package (rpm, deb) is
    /// the package manager's work and cannot be stopped from here.
    #[must_use]
    pub fn can_cancel(&self, name: &str) -> bool {
        self.jobs
            .iter()
            .find(|j| j.name == name && !j.is_finished())
            .is_some_and(|j| match j.phase {
                Phase::Pending | Phase::Verified => true,
                Phase::Installing => self.backend.system_format() == fs_store_core::Format::TarGz,
                _ => false,
            })
    }

    /// "Cancel" was pressed on `name`: tells the work to stop at its next safe point. False (nothing happens) when it cannot be
    /// stopped now.
    pub fn cancel_job(&mut self, name: &str) -> bool {
        if !self.can_cancel(name) {
            return false;
        }
        match self.job_cancel(name) {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }
}
