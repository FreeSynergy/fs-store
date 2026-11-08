//! Pattern: Observer (the worker thread reports each phase to the model, and the window is asked to draw
//! again). The job bar shows the phases of the job the model holds.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::backend::{JobError, Phase};
use crate::model::StoreModel;
use crate::words;

use super::shell::{Shared, locked, rebuild};

/// The six phases in a row; the one the job is in stands out. After a failure, `Failed` is told too.
pub(super) fn job_bar(model: &StoreModel) -> Div {
    let mut bar = div().class("fs-store-job").child(text(model.job_line()));
    if let Some(job) = model.job() {
        let mut marks = div().class("fs-store-phases");
        for phase in [
            Phase::Pending,
            Phase::Verified,
            Phase::Installing,
            if job.error.is_some() {
                if job.error == Some(JobError::RollbackFailed) {
                    Phase::Failed
                } else {
                    Phase::RolledBack
                }
            } else {
                Phase::Installed
            },
        ] {
            let mut mark = div().child(text(words::phase_name(phase)).class("fs-store-phase"));
            if job.phase == phase {
                mark = mark.class("fs-store-phase-now");
            }
            if job.history.contains(&phase) {
                marks = marks.child(mark);
            } else {
                marks = marks
                    .child(div().child(text(words::phase_name(phase)).class("fs-store-phase")));
            }
        }
        bar = bar.child(marks);
    }
    bar
}

/// Starts the job of `name` on a worker thread; every phase asks the window to draw again.
pub(super) fn start_job(shared: &Shared, name: &str) {
    let (backend, kind, name) = {
        let mut model = locked(shared);
        let Some(ticket) = model.begin_job(name) else {
            drop(model);
            rebuild();
            return;
        };
        let (kind, name) = (ticket.kind, ticket.name.to_owned());
        (model.backend(), kind, name)
    };
    rebuild();
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        // A panic in the worker must not leave the job "running" for ever, silently.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            backend.run(kind, &name, &mut |phase| {
                locked(&shared).observe(phase);
                rebuild();
            })
        }))
        .unwrap_or_else(|_| Err(JobError::Other(words::JOB_PANICKED.to_owned())));
        locked(&shared).finish(result);
        rebuild();
    });
}

/// Takes the confirmed program away on a worker thread (`pkexec` may wait for a password); the window is asked
/// to draw again when it starts and when it ends.
pub(super) fn start_uninstall(shared: &Shared) {
    let (backend, name) = {
        let mut model = locked(shared);
        let Some(name) = model.begin_uninstall() else {
            drop(model);
            rebuild();
            return;
        };
        (model.backend(), name)
    };
    rebuild();
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| backend.uninstall(&name)))
                .unwrap_or_else(|_| Err(words::JOB_PANICKED.to_owned()));
        locked(&shared).finish_uninstall(result);
        rebuild();
    });
}
