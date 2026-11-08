//! Pattern: Adapter (the menu of the title bar from `fs-components-app`; each entry is a model [`Command`]).

use fs_components_app::{AppError, AppWidget, Menu, MenuEntry};

use crate::model::{Command, Effect};
use crate::words;

use super::shell::{Shared, locked, rebuild};

/// Quits unless the model refuses (a job is running; then it tells why).
pub(super) fn quit(shared: &Shared) {
    let effect = locked(shared).command(Command::Quit);
    if effect == Effect::Quit {
        std::process::exit(0);
    }
    rebuild();
}

fn label(command: Command) -> &'static str {
    match command {
        Command::Quit => words::MENU_QUIT,
        Command::Cards => words::MENU_CARDS,
        Command::Sources => words::MENU_SOURCES,
        Command::Settings => words::MENU_SETTINGS,
        Command::About => words::MENU_ABOUT,
    }
}

fn entry(command: Command) -> MenuEntry {
    MenuEntry::new(command.id(), label(command))
}

/// Puts the menus into the title bar of `app`.
///
/// # Errors
/// [`AppError`] if the menu cannot be set.
pub(super) fn install_menus(app: &AppWidget, model: &Shared) -> Result<(), AppError> {
    let menus = vec![
        Menu::new(
            words::MENU_STORE,
            vec![
                entry(Command::Cards),
                entry(Command::Sources),
                entry(Command::Settings),
                entry(Command::Quit),
            ],
        ),
        Menu::new(words::MENU_HELP, vec![entry(Command::About)]),
    ];
    let shared = std::sync::Arc::clone(model);
    app.set_menus(menus, move |id| {
        let Some(command) = Command::from_id(id) else {
            return;
        };
        let effect = locked(&shared).command(command);
        match effect {
            Effect::Show(view) => locked(&shared).show(view),
            Effect::Quit => std::process::exit(0),
            Effect::Nothing => {}
        }
        rebuild();
    })
}
