//! Pattern: Command (the entries of the menu as values; the model says what each one does, the view does it).

use fs_store_sources::Face;

use super::store::StoreModel;
use super::types::{Problem, View};

/// An entry of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Show the cards.
    Cards,
    /// Close the window.
    Quit,
    /// Show the sources.
    Sources,
    /// Show the settings page.
    Settings,
    /// Show the About page.
    About,
    /// Open the help page of the current view in the browser (F1).
    Help,
    /// Wear this face (View menu); the choice is remembered.
    Face(Face),
}

impl Command {
    /// All entries.
    pub const ALL: [Command; 8] = [
        Command::Cards,
        Command::Sources,
        Command::Settings,
        Command::Quit,
        Command::About,
        Command::Help,
        Command::Face(Face::Storefront),
        Command::Face(Face::Compact),
    ];

    /// Stable id of the entry (what the menu hands back).
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Command::Face(Face::Storefront) => "face-storefront",
            Command::Face(Face::Compact) => "face-compact",
            Command::Cards => "cards",
            Command::Quit => "quit",
            Command::Help => "help",
            Command::Sources => "sources",
            Command::Settings => "settings",
            Command::About => "about",
        }
    }

    /// The entry with this id.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Command> {
        Self::ALL.into_iter().find(|c| c.id() == id)
    }
}

/// What the view has to do after a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Show this screen.
    Show(View),
    /// Close the window.
    Quit,
    /// Open the help page of the current view; the view opens it and tells a problem if that fails.
    Help,
    /// Nothing (a problem was told instead).
    Nothing,
}

/// DOM code of the key F1.
const KEY_F1: u32 = 112;

/// True for F1.
#[must_use]
pub fn is_help_key(key_code: u32) -> bool {
    key_code == KEY_F1
}

/// The help page of a view (ADR 0065; addresses in `fs-documentation` `todo/help-addresses.md`, never renamed).
/// `lang` is the person's language (`de...` gets the German pages, everything else English).
#[must_use]
pub fn help_url(view: View, lang: &str) -> String {
    let page = match view {
        View::Cards => "programs",
        View::Sources => "sources",
        View::Settings => "settings",
        View::About => "about",
    };
    let german = lang.to_ascii_lowercase().starts_with("de");
    format!(
        "https://freesynergy.github.io/fs-documentation/{}help/store/{page}/",
        if german { "de/" } else { "" }
    )
}

/// DOM code of the key Q.
const KEY_Q: u32 = 81;

/// True for Ctrl+Q.
#[must_use]
pub fn is_quit_key(key_code: u32, ctrl: bool) -> bool {
    ctrl && key_code == KEY_Q
}

impl StoreModel {
    /// Tells a problem that has no other place (for example the help page could not be opened).
    pub fn report_failure(&mut self, detail: &str) {
        self.problems.push(Problem::Backend(detail.to_owned()));
    }

    /// Carries out a menu entry. Quitting while a job runs is refused, and told.
    pub fn command(&mut self, command: Command) -> Effect {
        match command {
            Command::Cards => Effect::Show(View::Cards),
            Command::Sources => Effect::Show(View::Sources),
            Command::Settings => Effect::Show(View::Settings),
            Command::About => Effect::Show(View::About),
            Command::Help => Effect::Help,
            Command::Face(face) => {
                self.set_face(face);
                Effect::Nothing
            }
            Command::Quit => match self.busy_name() {
                Some(name) => {
                    self.problems = vec![Problem::Busy(name)];
                    Effect::Nothing
                }
                None => Effect::Quit,
            },
        }
    }
}
