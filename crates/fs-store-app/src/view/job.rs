//! Pattern: Observer (the worker thread reports each phase to the model, and the window is asked to draw
//! again). The job list shows every job the model holds.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::backend::{JobError, JobKind, StoreBackend};
use crate::model::StoreModel;
use crate::words;

use super::shell::{Shared, button, locked, rebuild};

/// The job list: one line per job of the queue, all at once; the finished ones can be cleared.
pub(super) fn job_list(model: &StoreModel, shared: &Shared) -> Div {
    let clear = Arc::clone(shared);
    let mut list = div()
        .class("fs-store-job")
        .child(text(words::JOBS_HEADING).bold());
    for line in model.job_lines() {
        list = list.child(text(line));
    }
    if model.jobs().iter().any(crate::model::JobView::is_finished) {
        list = list.child(
            div().child(
                button(
                    "clear-jobs",
                    words::BUTTON_CLEAR_JOBS,
                    ButtonVariant::Ghost,
                    true,
                )
                .on_click(move |_| {
                    locked(&clear).clear_finished_jobs();
                    rebuild();
                }),
            ),
        );
    }
    list
}

/// Puts the job of `name` into the queue; its download starts at once, next to the others.
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
    spawn_prepare(shared, backend, kind, name);
}

/// "Update all": every program with an update goes into the queue; they share one install step.
pub(super) fn start_all_updates(shared: &Shared) {
    let (backend, items) = {
        let mut model = locked(shared);
        let items = model.begin_all_updates();
        (model.backend(), items)
    };
    rebuild();
    for (kind, name) in items {
        spawn_prepare(shared, Arc::clone(&backend), kind, name);
    }
}

/// Downloads and checks on a worker thread; then, if every download of the queue is done, installs the batch.
fn spawn_prepare(shared: &Shared, backend: Arc<dyn StoreBackend>, kind: JobKind, name: String) {
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        // A panic in the worker must not leave the job "running" for ever, silently.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            backend.prepare(kind, &name)
        }))
        .unwrap_or_else(|_| Err(JobError::Other(words::JOB_PANICKED.to_owned())));
        locked(&shared).prepared(&name, result);
        rebuild();
        drive(&shared, backend.as_ref());
    });
}

/// Installs the batch when nothing downloads any more; repeats for what became ready meanwhile.
/// Only one thread installs at a time (the model hands the batch out once).
fn drive(shared: &Shared, backend: &dyn StoreBackend) {
    loop {
        let Some(batch) = locked(shared).take_commit() else {
            return;
        };
        rebuild();
        let names: Vec<String> = batch.iter().map(|(n, _)| n.clone()).collect();
        let results = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            backend.commit(batch, &mut |name, phase| {
                locked(shared).observe_job(name, phase);
                rebuild();
            })
        }))
        .unwrap_or_else(|_| {
            names
                .into_iter()
                .map(|n| (n, Err(JobError::Other(words::JOB_PANICKED.to_owned()))))
                .collect()
        });
        locked(shared).committed(results);
        rebuild();
    }
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
