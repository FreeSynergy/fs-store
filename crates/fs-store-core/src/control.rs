//! Pattern: Value object (a way to say "stop" and a way to say how far a download is). A cancel token is a shared flag: the
//! person's "Cancel" sets it, the steps that can be undone look at it between their small steps. A step that cannot be undone does
//! not look at it (the window shows its Cancel locked there).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// The text a step that stopped because of the token puts into its error (`InstallError`).
pub const CANCELLED: &str = "cancelled by the person";

/// A flag shared between the person's "Cancel" and the work.
#[derive(Debug, Clone)]
pub struct CancelToken(Arc<Inner>);

#[derive(Debug)]
struct Inner {
    flag: AtomicBool,
    /// Looks left until the flag is set by itself (`-1`: never). Only for tests that must cancel in the middle of a step.
    countdown: AtomicI64,
}

impl Default for CancelToken {
    fn default() -> Self {
        Self(Arc::new(Inner {
            flag: AtomicBool::new(false),
            countdown: AtomicI64::new(-1),
        }))
    }
}

impl PartialEq for CancelToken {
    /// Two tokens are equal when they are the same flag.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CancelToken {}

impl CancelToken {
    /// A token that is not set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A token that sets itself at the `looks`-th time somebody looks at it: a cancel in the middle of a step, the same every time
    /// (for tests; the person's "Cancel" is [`CancelToken::cancel`]).
    #[must_use]
    pub fn after_looks(looks: u32) -> Self {
        let token = Self::default();
        token.0.countdown.store(i64::from(looks), Ordering::SeqCst);
        token
    }

    /// Asks the work to stop at its next safe point.
    pub fn cancel(&self) {
        self.0.flag.store(true, Ordering::SeqCst);
    }

    /// Has the person asked to stop?
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        if self.0.flag.load(Ordering::SeqCst) {
            return true;
        }
        if self.0.countdown.load(Ordering::SeqCst) >= 0
            && self.0.countdown.fetch_sub(1, Ordering::SeqCst) <= 1
        {
            self.cancel();
            return true;
        }
        false
    }
}

/// How far a transfer is: bytes done, and the total when the server said it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Bytes read so far.
    pub done: u64,
    /// Bytes in all (`Content-Length`); `None` when not said.
    pub total: Option<u64>,
}

impl Progress {
    /// The share 0..1 when the total is known and not zero.
    #[must_use]
    pub fn share(&self) -> Option<f32> {
        let total = self.total.filter(|t| *t > 0)?;
        #[allow(clippy::cast_precision_loss)]
        Some((self.done as f32 / total as f32).clamp(0.0, 1.0))
    }
}
