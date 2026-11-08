//! Pattern: Command (the entries of the menu as values; the model says what each one does, the view does it).

use super::store::StoreModel;
use super::types::{Problem, View};

/// An entry of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Close the window.
    Quit,
    /// Show the sources.
    Sources,
    /// Show the settings page.
    Settings,
    /// Show the About page.
    About,
}

impl Command {
    /// All entries.
    pub const ALL: [Command; 4] = [
        Command::Sources,
        Command::Settings,
        Command::Quit,
        Command::About,
    ];

    /// Stable id of the entry (what the menu hands back).
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Command::Quit => "quit",
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
    /// Nothing (a problem was told instead).
    Nothing,
}

/// DOM code of the key Q.
const KEY_Q: u32 = 81;

/// True for Ctrl+Q.
#[must_use]
pub fn is_quit_key(key_code: u32, ctrl: bool) -> bool {
    ctrl && key_code == KEY_Q
}

impl StoreModel {
    /// Carries out a menu entry. Quitting while a job runs is refused, and told.
    pub fn command(&mut self, command: Command) -> Effect {
        match command {
            Command::Sources => Effect::Show(View::Sources),
            Command::Settings => Effect::Show(View::Settings),
            Command::About => Effect::Show(View::About),
            Command::Quit => match self.job.as_ref().filter(|j| !j.is_finished()) {
                Some(job) => {
                    self.problems = vec![Problem::Busy(job.name.clone())];
                    Effect::Nothing
                }
                None => Effect::Quit,
            },
        }
    }
}
