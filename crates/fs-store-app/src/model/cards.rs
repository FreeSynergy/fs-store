//! Pattern: View-model (one card per program: what the card shows and which buttons it has). Pure Rust, no GUI
//! types; the window only draws a [`Card`].

use fs_store_core::{Kind, Version};

use crate::backend::{JobKind, Phase};
use crate::words;

use super::store::StoreModel;
use super::types::Action;

/// Where a program is, as the card tells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// Offered, not on this machine: the button is "Install".
    NotInstalled,
    /// On this machine and up to date: "Open" over the whole width.
    Installed,
    /// On this machine, a newer version is offered: "Open" at half the width, "Update" beside it.
    UpdateAvailable,
}

/// How wide the "Open" button is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenWidth {
    /// The whole width of the button row.
    Full,
    /// Half of it; "Update" takes the other half.
    Half,
}

/// Which buttons a card has (the three-dot menu is always there).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    /// "Install" (not installed).
    pub install: bool,
    /// "Open", and how wide; `None` when the program is not installed.
    pub open: Option<OpenWidth>,
    /// "Update" (next to a half-wide "Open").
    pub update: bool,
}

/// What a card is busy with right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Working {
    /// An install or update job, in this phase.
    Job(JobKind, Phase),
    /// The program is being taken away.
    Removing,
    /// "Uninstall" was chosen and waits for the person's answer.
    Asking,
}

/// The stand-in for a logo: the first letter on a coloured block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placeholder {
    /// The first letter of the name, upper case; a leading `fs-` is skipped, for it is on every name.
    pub letter: char,
    /// The colour at the top left, derived from the name only, so it is the same every time.
    pub rgb: (u8, u8, u8),
    /// The colour at the bottom right: the same hue, a little darker, for a soft gradient.
    pub rgb_end: (u8, u8, u8),
}

/// One program on the page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// Name (shown, and the key of every command).
    pub name: String,
    /// The name a person reads (`FreeSynergy.People`); never `fs-...`.
    pub title: String,
    /// Kind, when a source offers it.
    pub kind: Option<Kind>,
    /// Where it is.
    pub state: CardState,
    /// The offered version, when a source offers it.
    pub offered: Option<Version>,
    /// The installed version.
    pub installed: Option<Version>,
    /// The description of the catalog entry (may be empty).
    pub description: String,
    /// The project page (may be empty).
    pub homepage: String,
    /// Marked "New" by the last check.
    pub is_new: bool,
    /// The cursor is over this card.
    pub hovered: bool,
    /// The description panel shows this card.
    pub selected: bool,
    /// What it is busy with, or waits for.
    pub working: Option<Working>,
    /// Why its last job failed, as a sentence; stays on the card until the next try.
    pub failure: Option<String>,
    /// Its buttons.
    pub buttons: Buttons,
    /// Its stand-in logo.
    pub placeholder: Placeholder,
    /// The stored icon (a PNG file), when the catalog entry has one and the engine kept it; it replaces the stand-in.
    pub icon: Option<std::path::PathBuf>,
}

impl Card {
    /// True while "Uninstall" waits for the person's answer.
    #[must_use]
    pub fn is_confirming(&self) -> bool {
        self.working == Some(Working::Asking)
    }

    /// Text of the description panel: the description, or a sentence saying there is none.
    #[must_use]
    pub fn description_text(&self) -> String {
        if self.description.trim().is_empty() {
            words::NO_DESCRIPTION.to_owned()
        } else {
            self.description.trim().to_owned()
        }
    }

    /// The version line under the name: the installed one, or the offered one; "old -> new" for an update.
    #[must_use]
    pub fn version_line(&self) -> String {
        match (self.installed, self.offered, self.state) {
            (Some(old), Some(new), CardState::UpdateAvailable) => {
                format!("{old} -> {new}")
            }
            (Some(v), _, _) | (None, Some(v), _) => v.to_string(),
            (None, None, _) => String::new(),
        }
    }
}

/// Which buttons a state has.
#[must_use]
pub fn buttons_for(state: CardState) -> Buttons {
    match state {
        CardState::NotInstalled => Buttons {
            install: true,
            open: None,
            update: false,
        },
        CardState::Installed => Buttons {
            install: false,
            open: Some(OpenWidth::Full),
            update: false,
        },
        CardState::UpdateAvailable => Buttons {
            install: false,
            open: Some(OpenWidth::Half),
            update: true,
        },
    }
}

/// Letter and colour for `name`. The colour comes from a hash of the whole name (FNV-1a), turned into a hue with
/// fixed lightness, so every name keeps its colour and white letters stay readable.
#[must_use]
pub fn placeholder(name: &str) -> Placeholder {
    let plain = name.strip_prefix("fs-").unwrap_or(name);
    let letter = plain
        .chars()
        .find(|c| c.is_alphanumeric())
        .or_else(|| name.chars().next())
        .map_or('?', |c| c.to_uppercase().next().unwrap_or(c));
    let hash = name.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    #[allow(clippy::cast_precision_loss)]
    let hue = (hash % 360) as f32;
    Placeholder {
        letter,
        rgb: hsl(hue, 0.52, 0.52),
        rgb_end: hsl((hue + 16.0) % 360.0, 0.58, 0.36),
    }
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

/// Hue 0..360, saturation and lightness 0..1, to rgb.
fn hsl(hue: f32, saturation: f32, lightness: f32) -> (u8, u8, u8) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let mid = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let offset = lightness - chroma / 2.0;
    let (red, green, blue) = match hue {
        h if h < 60.0 => (chroma, mid, 0.0),
        h if h < 120.0 => (mid, chroma, 0.0),
        h if h < 180.0 => (0.0, chroma, mid),
        h if h < 240.0 => (0.0, mid, chroma),
        h if h < 300.0 => (mid, 0.0, chroma),
        _ => (chroma, 0.0, mid),
    };
    let byte = |value: f32| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let out = ((value + offset) * 255.0).round() as u8;
        out
    };
    (byte(red), byte(green), byte(blue))
}

impl StoreModel {
    /// One card per program, by name: everything a source offers plus everything installed, never a name twice.
    #[must_use]
    pub fn cards(&self) -> Vec<Card> {
        let mut names: Vec<&str> = self
            .available
            .iter()
            .map(|r| r.package.name.as_str())
            .chain(self.installed.iter().map(|r| r.name.as_str()))
            .collect();
        names.sort_unstable();
        names.dedup();
        names.into_iter().map(|name| self.card(name)).collect()
    }

    /// The card of `name`, as it is now.
    fn card(&self, name: &str) -> Card {
        let row = self.available.iter().find(|r| r.package.name == name);
        let installed = self
            .installed
            .iter()
            .find(|r| r.name == name)
            .map(|r| r.version);
        let state = match (installed, row.map(|r| r.action)) {
            (None, _) => CardState::NotInstalled,
            (Some(_), Some(Action::Update)) => CardState::UpdateAvailable,
            (Some(_), _) => CardState::Installed,
        };
        let working = if self.ui.removing.as_deref() == Some(name) {
            Some(Working::Removing)
        } else if self.ui.confirm.as_deref() == Some(name) {
            Some(Working::Asking)
        } else {
            self.jobs
                .iter()
                .find(|j| j.name == name && !j.is_finished())
                .map(|j| Working::Job(j.kind, j.phase))
        };
        Card {
            name: name.to_owned(),
            title: self.title(name),
            kind: row.map(|r| r.package.kind),
            state,
            offered: row.map(|r| r.package.version),
            installed,
            description: row
                .map(|r| r.package.description.clone())
                .unwrap_or_default(),
            homepage: row.map(|r| r.package.homepage.clone()).unwrap_or_default(),
            is_new: self.is_fresh(name),
            hovered: self.ui.hovered.as_deref() == Some(name),
            selected: self.ui.selected.as_deref() == Some(name),
            working,
            failure: self
                .jobs
                .iter()
                .find(|j| j.name == name)
                .and_then(|j| j.error.as_ref())
                .map(|e| super::text::job_failed_line(&self.title(name), e)),
            buttons: buttons_for(state),
            placeholder: placeholder(name),
            icon: self.backend.icon_path(name),
        }
    }
}
