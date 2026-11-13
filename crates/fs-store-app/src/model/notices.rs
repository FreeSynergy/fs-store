//! Pattern: Observer over the problems (the strip of messages at the top, with a clock that is handed in, so a test can set the
//! time). A message shows a few seconds (an error longer), has a close button, and stays in the history for good. The strip is
//! made from the problems the model holds; it only decides what is still shown.

use super::store::StoreModel;
use super::types::Problem;

/// How long a hint shows, in seconds.
pub const INFO_SECONDS: u64 = 6;
/// How long an error shows, in seconds (it can be closed earlier).
pub const ERROR_SECONDS: u64 = 20;
/// How many messages the history keeps.
const HISTORY_MAX: usize = 100;

/// How serious a message is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A hint (busy, nothing to do): goes quickly.
    Info,
    /// Something did not work: stays longer.
    Error,
}

/// One message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// Counts up; what the close button names.
    pub id: u64,
    /// What it says.
    pub text: String,
    /// How serious.
    pub severity: Severity,
    /// When it appeared (Unix seconds).
    pub since: u64,
}

impl Notice {
    /// True while the message is still due on the strip at `now`.
    #[must_use]
    pub fn due(&self, now: u64) -> bool {
        let span = match self.severity {
            Severity::Info => INFO_SECONDS,
            Severity::Error => ERROR_SECONDS,
        };
        now < self.since.saturating_add(span)
    }
}

#[derive(Debug, Clone)]
struct Entry {
    notice: Notice,
    /// The problem is still in the model.
    live: bool,
    closed: bool,
}

/// The messages: the ones on the strip and the history (the same list, oldest first).
#[derive(Debug, Clone, Default)]
pub struct NoticeBoard {
    next: u64,
    entries: Vec<Entry>,
}

impl NoticeBoard {
    /// What the model says now is `current`; a text that was not there before is a new message at `now`.
    fn sync(&mut self, current: Vec<(String, Severity)>, now: u64) {
        for entry in &mut self.entries {
            entry.live = current.iter().any(|(t, _)| *t == entry.notice.text) && entry.live;
        }
        for (text, severity) in current {
            if self
                .entries
                .iter()
                .any(|e| e.live && e.notice.text == text)
            {
                continue;
            }
            self.next += 1;
            self.entries.push(Entry {
                notice: Notice {
                    id: self.next,
                    text,
                    severity,
                    since: now,
                },
                live: true,
                closed: false,
            });
        }
        if self.entries.len() > HISTORY_MAX {
            let over = self.entries.len() - HISTORY_MAX;
            self.entries.drain(..over);
        }
    }

    /// A new read begins: what is told again after it is a new message (a reload that fails twice says so twice).
    pub(super) fn release(&mut self) {
        for entry in &mut self.entries {
            entry.live = false;
        }
    }
}

impl Problem {
    /// How serious the problem is for the strip.
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Problem::Busy(_)
            | Problem::Locked(_)
            | Problem::NothingToDo(_)
            | Problem::NoHomepage(_)
            | Problem::EmptyAddress => Severity::Info,
            _ => Severity::Error,
        }
    }
}

impl StoreModel {
    /// Takes the problems of the model into the board; call it before drawing, with the time of the draw.
    pub fn sync_notices(&mut self, now: u64) {
        let lines = self.problem_lines();
        let current = self
            .problems
            .iter()
            .zip(lines)
            .map(|(p, text)| (text, p.severity()))
            .collect();
        self.notices.sync(current, now);
    }

    /// The messages on the strip at `now`: not closed, not too old, still true. Oldest first.
    #[must_use]
    pub fn visible_notices(&self, now: u64) -> Vec<Notice> {
        self.notices
            .entries
            .iter()
            .filter(|e| e.live && !e.closed && e.notice.due(now))
            .map(|e| e.notice.clone())
            .collect()
    }

    /// When the strip next changes by itself (a message runs out), if one is on it: the view draws again then.
    #[must_use]
    pub fn next_notice_expiry(&self, now: u64) -> Option<u64> {
        self.visible_notices(now)
            .iter()
            .map(|n| {
                n.since
                    + match n.severity {
                        Severity::Info => INFO_SECONDS,
                        Severity::Error => ERROR_SECONDS,
                    }
            })
            .min()
    }

    /// The close button of a message.
    pub fn close_notice(&mut self, id: u64) {
        if let Some(entry) = self.notices.entries.iter_mut().find(|e| e.notice.id == id) {
            entry.closed = true;
        }
    }

    /// Every message there was, oldest first (the history page).
    #[must_use]
    pub fn notice_history(&self) -> Vec<Notice> {
        self.notices
            .entries
            .iter()
            .map(|e| e.notice.clone())
            .collect()
    }
}
