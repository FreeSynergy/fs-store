//! Pattern: Adapter (the menu of the title bar from `fs-components-app`; each entry is a model [`Command`]).

use fs_components_app::{AppError, AppWidget, Menu, MenuEntry};

use std::sync::Arc;

use crate::launcher::Launcher;
use crate::model::{Command, Effect, help_url};
use crate::words;
use crate::words_lang::{self, Lang};
use fs_store_sources::Face;

use super::shell::{Shared, locked, rebuild};

/// Quits unless the model refuses (a job is running; then it tells why).
pub(super) fn quit(shared: &Shared) {
    let effect = locked(shared).command(Command::Quit);
    if effect == Effect::Quit {
        std::process::exit(0);
    }
    rebuild();
}

/// F1 and the Help entry: the page of the current view in the browser; a failure is shown, never silent.
pub(super) fn open_help(shared: &Shared, launcher: &Arc<dyn Launcher>) {
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    let url = help_url(locked(shared).view(), &lang);
    if let Err(detail) = launcher.open_url(&url) {
        locked(shared).report_failure(&format!("{url}: {detail}"));
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
        Command::Help => words::MENU_HELP_PAGE,
        // the line of a face is made in `menus`
        Command::Face(_) => "",
    }
}

fn entry(command: Command) -> MenuEntry {
    MenuEntry::new(command.id(), label(command))
}

/// The menus of the title bar; the View menu lists every face, a dot marks the one worn now. Only faces the code has are listed.
#[must_use]
pub fn menus(face: Face, lang: Lang) -> Vec<Menu> {
    let faces = Face::ALL
        .into_iter()
        .map(|f| {
            let mark = if f == face { "\u{25cf}" } else { "\u{25cb}" };
            MenuEntry::new(
                Command::Face(f).id(),
                words_lang::face_line(lang, f).replace("{mark}", mark),
            )
        })
        .collect();
    vec![
        Menu::new(
            words::MENU_STORE,
            vec![
                entry(Command::Cards),
                entry(Command::Sources),
                entry(Command::Settings),
                entry(Command::Quit),
            ],
        ),
        Menu::new(words_lang::menu_view(lang), faces),
        Menu::new(
            words::MENU_HELP,
            vec![entry(Command::Help), entry(Command::About)],
        ),
    ]
}

/// Puts the menus into the title bar of `app`. Choosing a face puts them in again, with the dot on the new face.
///
/// # Errors
/// [`AppError`] if the menu cannot be set.
pub(super) fn install_menus(
    app: &AppWidget,
    model: &Shared,
    launcher: &Arc<dyn Launcher>,
) -> Result<(), AppError> {
    let face = locked(model).face();
    let shared = Arc::clone(model);
    let launcher = Arc::clone(launcher);
    let again = app.clone();
    app.set_menus(menus(face, Lang::current()), move |id| {
        let Some(command) = Command::from_id(id) else {
            return;
        };
        let effect = locked(&shared).command(command);
        match effect {
            Effect::Show(view) => locked(&shared).show(view),
            Effect::Quit => std::process::exit(0),
            Effect::Help => open_help(&shared, &launcher),
            Effect::Nothing => {}
        }
        if matches!(command, Command::Face(_)) {
            // the dot moves to the face now worn
            let _ = install_menus(&again, &shared, &launcher);
        }
        rebuild();
    })
}
