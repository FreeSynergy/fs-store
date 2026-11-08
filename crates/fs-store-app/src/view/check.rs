//! Pattern: Observer (the check for updates runs on a worker thread; the window is asked to draw again).

use std::sync::Arc;

use crate::words;

use super::shell::{Shared, locked, rebuild};

/// Looks for updates once, on a worker thread (the check is a child process of its own); the window is asked
/// to draw again when it starts and when it ends. Does nothing if a check runs already.
pub fn start_check(shared: &Shared) {
    let service = {
        let mut model = locked(shared);
        if !model.begin_check() {
            return;
        }
        model.update_service()
    };
    rebuild();
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service.check()))
            .unwrap_or_else(|_| Err(words::JOB_PANICKED.to_owned()));
        locked(&shared).finish_check(result);
        rebuild();
    });
}
