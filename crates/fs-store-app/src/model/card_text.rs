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
}
