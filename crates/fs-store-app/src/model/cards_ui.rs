//! Pattern: Command + State (what the person does on the cards: hover, select, the repository link and the
//! uninstall that asks first). The window only calls these and draws [`super::Card`].

use crate::launcher::Launcher;

use super::store::StoreModel;
use super::types::Problem;

/// The little state of the card page.
#[derive(Debug, Clone, Default)]
pub struct CardsUi {
    /// The card under the cursor.
    pub(super) hovered: Option<String>,
    /// The card whose description is shown.
    pub(super) selected: Option<String>,
    /// The card whose "Uninstall" waits for an answer.
    pub(super) confirm: Option<String>,
    /// The program that is being taken away.
    pub(super) removing: Option<String>,
}

impl StoreModel {
    /// The cursor came onto the card `name`.
    /// True when that changed something (the window draws again only then).
    pub fn hover(&mut self, name: &str) -> bool {
        let changed = self.ui.hovered.as_deref() != Some(name);
        self.ui.hovered = Some(name.to_owned());
        changed
    }

    /// The cursor left the card `name` (a card that is not the hovered one changes nothing: the enter event of
    /// the next card may come first). True when that changed something.
    pub fn unhover(&mut self, name: &str) -> bool {
        let changed = self.ui.hovered.as_deref() == Some(name);
        if changed {
            self.ui.hovered = None;
        }
        changed
    }

    /// The person clicked the card: its description is shown. Clicking the shown card again hides it.
    pub fn select(&mut self, name: &str) {
        self.ui.selected = if self.ui.selected.as_deref() == Some(name) {
            None
        } else {
            Some(name.to_owned())
        };
    }

    /// Hides the description.
    pub fn close_description(&mut self) {
        self.ui.selected = None;
    }

    /// The card whose description is shown.
    #[must_use]
    pub fn selected(&self) -> Option<super::Card> {
        let name = self.ui.selected.as_deref()?;
        self.cards().into_iter().find(|c| c.name == name)
    }

    /// Opens the project page of `name` with the system's opener, as a process of its own. A missing or odd
    /// address and a failing opener are told as problems.
    pub fn open_repository(&mut self, name: &str, launcher: &dyn Launcher) {
        self.problems.clear();
        let url = self
            .available
            .iter()
            .find(|r| r.package.name == name)
            .map(|r| r.package.homepage.trim().to_owned())
            .unwrap_or_default();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            self.problems.push(Problem::NoHomepage(name.to_owned()));
            return;
        }
        if let Err(detail) = launcher.open_url(&url) {
            self.problems
                .push(Problem::RepositoryFailed { url, detail });
        }
    }

    /// "Uninstall" was chosen: ask first. Only an installed program, and not while something runs.
    pub fn ask_uninstall(&mut self, name: &str) {
        self.problems.clear();
        if let Some(busy) = self.busy_name() {
            self.problems.push(Problem::Busy(busy));
        } else if self.installed.iter().any(|r| r.name == name) {
            self.ui.confirm = Some(name.to_owned());
        } else {
            self.problems.push(Problem::NothingToDo(name.to_owned()));
        }
    }

    /// The person said no.
    pub fn cancel_uninstall(&mut self) {
        self.ui.confirm = None;
    }

    /// The person said yes: returns the name to take away; the card is "removing" until
    /// [`StoreModel::finish_uninstall`]. `None` when nothing was asked, or something runs.
    pub fn begin_uninstall(&mut self) -> Option<String> {
        let name = self.ui.confirm.take()?;
        if let Some(busy) = self.busy_name() {
            self.problems = vec![Problem::Busy(busy)];
            return None;
        }
        self.problems.clear();
        self.ui.removing = Some(name.clone());
        Some(name)
    }

    /// The backend is done. A failure is told as a problem and the program stays on its card.
    pub fn finish_uninstall(&mut self, result: Result<(), String>) {
        let Some(name) = self.ui.removing.take() else {
            return;
        };
        match result {
            Ok(()) => self.updated(&name),
            Err(detail) => self
                .problems
                .push(Problem::UninstallFailed { name, detail }),
        }
        self.read_installed();
        self.merge();
    }

    /// Takes the confirmed program away on this thread (tests, and programs without a window).
    pub fn run_uninstall(&mut self) {
        if let Some(name) = self.begin_uninstall() {
            let result = self.backend().uninstall(&name);
            self.finish_uninstall(result);
        }
    }

    /// The name of what runs now (a job or an uninstall), if anything.
    pub(super) fn busy_name(&self) -> Option<String> {
        self.ui.removing.clone().or_else(|| {
            self.jobs
                .iter()
                .find(|j| !j.is_finished())
                .map(|j| j.name.clone())
        })
    }
}
