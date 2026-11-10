//! Pattern: Optimistic concurrency (a counter). The window reads the catalogs on a worker thread, outside the lock
//! of the model; every change of what is installed (a job ended, an uninstall ended) or of the sources counts the
//! epoch up, and a result read under an older epoch is thrown away, so an old read never undoes a newer change.

use std::sync::Arc;

use crate::backend::StoreBackend;

use super::fetch::{Fetched, fetch};
use super::store::StoreModel;

/// The counter and the little state of the refresh.
#[derive(Debug, Clone, Copy, Default)]
pub struct RefreshState {
    /// Counts up with every change that makes a running read old.
    pub(super) epoch: u64,
    /// A read runs on a worker thread.
    pub(super) running: bool,
    /// The window reads on a worker thread (see [`StoreModel::defer_refresh`]).
    pub(super) defer: bool,
    /// A source command asked for a read that the window has not started yet.
    pub(super) wanted: bool,
}

/// What a worker needs to read.
pub struct ReadTicket {
    backend: Arc<dyn StoreBackend>,
    epoch: u64,
}

impl ReadTicket {
    /// Reads (slow); give the result to [`StoreModel::finish_read`].
    #[must_use]
    pub fn read(&self) -> (u64, Fetched) {
        (self.epoch, fetch(self.backend.as_ref()))
    }
}

impl StoreModel {
    /// The window reads on a worker thread: source commands then only ask for a read ([`StoreModel::take_read_wanted`])
    /// instead of reading on the spot under the lock.
    pub fn defer_refresh(&mut self) {
        self.sync.defer = true;
    }

    /// True while a read runs (the button shows it and cannot be pressed twice).
    #[must_use]
    pub fn is_refreshing(&self) -> bool {
        self.sync.running
    }

    /// Starts a read: `None` when one runs already.
    pub fn begin_read(&mut self) -> Option<ReadTicket> {
        if self.sync.running {
            return None;
        }
        self.sync.running = true;
        self.sync.wanted = false;
        Some(ReadTicket {
            backend: Arc::clone(&self.backend),
            epoch: self.sync.epoch,
        })
    }

    /// The read is done. Taken over when nothing changed meanwhile; otherwise thrown away and `false` (read again).
    pub fn finish_read(&mut self, result: (u64, Fetched)) -> bool {
        self.sync.running = false;
        let (epoch, fetched) = result;
        if epoch != self.sync.epoch {
            return false;
        }
        self.apply_fetched(fetched);
        true
    }

    /// True once after a source command asked for a read.
    pub fn take_read_wanted(&mut self) -> bool {
        std::mem::take(&mut self.sync.wanted)
    }

    /// A source command changed the list: read again, now (tests, programs without a window) or by the window.
    pub(super) fn sources_changed(&mut self) {
        self.sync.epoch += 1;
        if self.sync.defer {
            self.sync.wanted = true;
        } else {
            self.refresh();
        }
    }
}
