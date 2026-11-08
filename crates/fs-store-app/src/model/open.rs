//! Pattern: Command (opening an installed program or one of its facets; starting sits behind [`Launcher`]).

use crate::launcher::Launcher;

use super::store::StoreModel;
use super::types::Problem;

impl StoreModel {
    /// The command that opens the program itself: `fs-<name>` if it exists, else `<name>-app`, else the first
    /// command the package brought. `None` when none exists.
    #[must_use]
    pub fn program_command(&self, name: &str, launcher: &dyn Launcher) -> Option<String> {
        let prefixed = if name.starts_with("fs-") {
            name.to_owned()
        } else {
            format!("fs-{name}")
        };
        [prefixed, format!("{name}-app")]
            .into_iter()
            .find(|c| launcher.exists(c))
            .or_else(|| {
                self.details(name)
                    .commands
                    .into_iter()
                    .map(|(command, _)| command)
                    .find(|c| launcher.exists(c))
            })
    }

    /// Opens the program `name` (`facet` is `None`) or one of its facets, as a process of its own. A failure
    /// is told as a problem, never swallowed.
    pub fn open(&mut self, name: &str, facet: Option<&str>, launcher: &dyn Launcher) {
        self.problems.clear();
        let command = match facet {
            Some(f) => Some(f.to_owned()),
            None => self.program_command(name, launcher),
        };
        let Some(command) = command else {
            self.problems.push(Problem::LaunchFailed {
                command: String::new(),
                detail: name.to_owned(),
            });
            return;
        };
        if let Err(detail) = launcher.start(&command) {
            self.problems
                .push(Problem::LaunchFailed { command, detail });
        }
    }
}
