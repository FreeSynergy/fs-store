//! Pattern: none (pure functions from the update state to sentences; the sentences are in `words`).

use fs_store_sources::{CheckEvery, InstallMode, Update};

use crate::words;

use super::store::StoreModel;
use super::updates::CheckState;

/// "2026-11-08 14:05 UTC" for Unix seconds.
#[must_use]
pub fn format_time(secs: u64) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rest = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02} UTC",
        rest / 3600,
        rest % 3600 / 60
    )
}

/// The label of a choice of "Look for updates".
#[must_use]
pub fn check_label(every: CheckEvery) -> &'static str {
    match every {
        CheckEvery::Off => words::CHECK_OFF,
        CheckEvery::Start => words::CHECK_START,
        CheckEvery::Daily => words::CHECK_DAILY,
        CheckEvery::Hourly => words::CHECK_HOURLY,
    }
}

/// The label of a choice of "Install updates".
#[must_use]
pub fn install_label(mode: InstallMode) -> &'static str {
    match mode {
        InstallMode::Ask => words::INSTALL_ASK,
        InstallMode::Auto => words::INSTALL_AUTO,
    }
}

/// "name: from -> to".
#[must_use]
pub fn update_line(update: &Update) -> String {
    words::fill(
        words::UPDATE_LINE,
        &[
            ("name", &update.name),
            ("from", &update.from),
            ("to", &update.to),
        ],
    )
}

impl StoreModel {
    /// Title of the window: "FreeSynergy.Store (n)" when there is a badge.
    #[must_use]
    pub fn window_title(&self) -> String {
        match self.badge() {
            0 => words::TITLE.to_owned(),
            n => words::fill(words::TITLE_COUNT, &[("n", &n.to_string())]),
        }
    }

    /// When the last check was, or that there was none.
    #[must_use]
    pub fn last_check_line(&self) -> String {
        match self.last_checked() {
            0 => words::LAST_CHECK_NEVER.to_owned(),
            secs => words::fill(words::LAST_CHECK, &[("time", &format_time(secs))]),
        }
    }

    /// What the check is doing or did in this window; `None` before the first one.
    #[must_use]
    pub fn check_line(&self) -> Option<String> {
        match &self.check {
            CheckState::Idle => None,
            CheckState::Running => Some(words::CHECK_RUNNING.to_owned()),
            CheckState::Done => Some(words::CHECK_DONE.to_owned()),
            CheckState::Failed(detail) => {
                Some(words::fill(words::CHECK_FAILED, &[("detail", detail)]))
            }
        }
    }
}
