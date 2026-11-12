//! Pattern: none (the words under a card: the state badge and the line with date and size).

use super::cards::{Card, CardState};

impl Card {
    /// The state in words, for the badge on the card.
    #[must_use]
    pub fn state_word(&self) -> &'static str {
        match self.state {
            CardState::NotInstalled => "Available to install",
            CardState::Installed => "Installed",
            CardState::UpdateAvailable => "Update available",
        }
    }

    /// The state of a running job in a word, for the badge ("Installing", "Waiting", ...).
    #[must_use]
    pub fn working_word(&self) -> &'static str {
        use crate::backend::Phase;
        match self.working {
            Some(super::cards::Working::Job(_, Phase::Pending)) => "Waiting",
            Some(super::cards::Working::Job(_, Phase::Verified)) => "Verified",
            Some(super::cards::Working::Job(_, Phase::Installing)) => "Installing",
            Some(super::cards::Working::Job(_, _)) => "Working",
            Some(super::cards::Working::Removing | super::cards::Working::Asking) => "Removing",
            None => "",
        }
    }

    /// `0.1.0 installed -> 0.2.0 released`, or `Version 0.7.0`: the first line under the badge of a tile.
    #[must_use]
    pub fn version_line_long(&self) -> String {
        match (self.installed, self.offered, self.state) {
            (Some(old), Some(new), CardState::UpdateAvailable) => {
                format!("{old} installed \u{2192} {new} released")
            }
            (Some(v), _, _) | (None, Some(v), _) => format!("Version {v}"),
            (None, None, _) => String::new(),
        }
    }

    /// The line `0.7.0 - 2026-11-09 - 13 MiB`, with `unknown` where the catalog is silent.
    #[must_use]
    pub fn meta_line(&self) -> String {
        let date = if self.released.is_empty() {
            crate::words_front::UNKNOWN
        } else {
            self.released.as_str()
        };
        format!("{date} - {}", self.size)
    }

    /// How far a running job is, as a share 0..1 and the sentence under the bar. While the release file is read it is bytes (what the
    /// server said, nothing guessed); else the phase. `None` when the card is not busy with a job.
    #[must_use]
    pub fn progress(&self) -> Option<(f32, String)> {
        use crate::backend::Phase;
        let super::cards::Working::Job(_, phase) = self.working? else {
            return None;
        };
        if phase == Phase::Pending
            && let Some(bytes) = self.bytes
        {
            let line = match bytes.total {
                Some(total) => format!(
                    "Downloading {} of {}{}",
                    super::front_text::size_text(bytes.done),
                    super::front_text::size_text(total),
                    bytes.share().map_or_else(String::new, |s| format!(
                        " \u{b7} {} %",
                        (s * 100.0).round()
                    ))
                ),
                None => format!("Downloading {}", super::front_text::size_text(bytes.done)),
            };
            return Some((bytes.share().unwrap_or(0.1), line));
        }
        let (share, line) = match phase {
            Phase::Pending => (0.1, "Waiting for the download and the checks"),
            Phase::Verified => (
                0.45,
                "Checksum and signature are good; waiting for the others",
            ),
            Phase::Installing => (0.8, "Installing"),
            Phase::Installed => (1.0, "Installed"),
            Phase::Failed | Phase::RolledBack => (1.0, "Something went wrong; going back"),
        };
        Some((share, line.to_owned()))
    }
}

/// Why "Open" is locked for a package of this kind: a component or a theme has no window of its own. `None` for everything else.
#[must_use]
pub(super) fn open_blocked(kind: Option<fs_store_core::Kind>) -> Option<&'static str> {
    use fs_store_core::Kind;
    matches!(kind, Some(Kind::Component | Kind::Theme))
        .then(|| crate::words_front::no_window(crate::words_front::Lang::current()))
}

/// Why "Install" is locked: the package carries a reason (`locked`), and the words say it for its kind in the language of the program. `None`
/// when it can be installed.
#[must_use]
pub(super) fn install_blocked(
    locked: Option<&str>,
    kind: Option<fs_store_core::Kind>,
) -> Option<&'static str> {
    use fs_store_core::Kind;
    locked?;
    let lang = crate::words_front::Lang::current();
    Some(match kind {
        Some(Kind::Server | Kind::IconTheme) => crate::words_lang::coming_soon(lang),
        _ => crate::words_lang::not_supported(lang),
    })
}

/// How many cards of width `item` with `gap` between them fit in a row of `room` pixels (at least one).
#[must_use]
pub fn columns(room: f32, item: f32, gap: f32) -> usize {
    let fit = ((room + gap) / (item + gap)).floor();
    if fit < 1.0 {
        return 1;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let fit = fit as usize;
    fit
}
