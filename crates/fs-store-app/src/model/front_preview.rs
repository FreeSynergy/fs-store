//! Pattern: Builder (the fixed list of steps an install, update or removal does, in plain words, before it does
//! anything). The steps are those the store core really runs (`C-install.md`); each has a risk and an actor.

use fs_store_core::Format;

use crate::words;
use crate::words_front as w;

use super::cards::CardState;
use super::front_steps::{Actor, Preview, PreviewAction, Risk, Step, Warning};
use super::store::StoreModel;

fn step(text: String, risk: Risk, actor: Actor) -> Step {
    Step { text, risk, actor }
}

impl StoreModel {
    /// Size in words for a sentence ("13 MiB", or "size unknown").
    pub(super) fn size_for_sentence(&self, name: &str) -> String {
        self.available
            .iter()
            .find(|r| r.package.name == name)
            .and_then(|r| r.package.details.size)
            .map_or_else(|| "size unknown".to_owned(), super::front_text::size_text)
    }

    /// The steps of a first install or an update of `name`.
    fn install_steps(&self, name: &str, update: bool) -> Vec<Step> {
        let title = self.title(name);
        let Some(row) = self.available.iter().find(|r| r.package.name == name) else {
            return Vec::new();
        };
        let source = self
            .sources
            .iter()
            .find(|s| s.id == row.package.source)
            .map_or_else(|| row.package.source.clone(), |s| s.url.clone());
        let version = row.package.version.to_string();
        let size = self.size_for_sentence(name);
        let native = match self.backend.system_format() {
            f @ (Format::Rpm | Format::Deb) if row.package.native.contains(&f) => Some(f),
            _ => None,
        };
        let mut steps = vec![
            step(
                format!("The list of {source} is read and its signature is checked against the project key."),
                Risk::Look,
                Actor::Program,
            ),
            step(
                format!("{title} {version} ({size}) is downloaded."),
                Risk::Look,
                Actor::Program,
            ),
            step(
                "Checksum and signature of the file are checked. Without a valid signature nothing is installed."
                    .to_owned(),
                Risk::Look,
                Actor::Program,
            ),
            step(
                format!("If {title} is open it is not replaced; close it first."),
                Risk::Look,
                Actor::Program,
            ),
        ];
        if update {
            steps.push(step(
                format!("A background server of {title} that still runs is ended in order, then started again by the program."),
                Risk::Start,
                Actor::Program,
            ));
        }
        if let Some(format) = native {
            let tool = if format == Format::Rpm { "dnf" } else { "apt" };
            steps.push(step(
                "The checked package file is put into a private folder of the store.".to_owned(),
                Risk::Install,
                Actor::Program,
            ));
            steps.push(step(
                    format!("The package is installed for the whole system with {tool}. You confirm the administrator password; the store never sees it."),
                    Risk::System,
                    Actor::Person,
                ));
        } else {
            steps.push(step(
                    if update {
                        "The new files are unpacked into the store's folder; the version you have stays there to go back to.".to_owned()
                    } else {
                        "The files are unpacked into the store's folder. Nothing outside it is written yet.".to_owned()
                    },
                    Risk::Install,
                    Actor::Program,
                ));
            steps.push(step(
                    format!("The command and the menu entry {title} are set up for you; a file of another maker with the same name is never replaced."),
                    Risk::Install,
                    Actor::Program,
                ));
            steps.push(step(
                    "The install is noted in the store's list with the checksum of every file (fs-store verify checks them later).".to_owned(),
                    Risk::Open,
                    Actor::Program,
                ));
        }
        steps
    }

    /// What the person must read about the needs of `name` before the install: a need nobody offers is a warning, a need with several
    /// providers is a note. Nothing for a package that needs nothing or whose needs are all met by one package.
    fn need_warnings(&self, name: &str) -> Vec<Warning> {
        use super::front_types::NeedsState;
        let lang = w::Lang::current();
        let title = self.title(name);
        self.capability_lines(name)
            .requires
            .iter()
            .filter_map(|l| match l.state {
                NeedsState::Missing => Some(Warning {
                    text: words::fill(
                        w::missing_need(lang),
                        &[("need", &l.text), ("title", &title)],
                    ),
                    missing: true,
                }),
                NeedsState::Choice => Some(Warning {
                    text: words::fill(
                        w::choice_need(lang),
                        &[("need", &l.text), ("by", &l.answer)],
                    ),
                    missing: false,
                }),
                _ => None,
            })
            .collect()
    }

    /// The steps of taking `name` away.
    fn remove_steps(&self, name: &str) -> Vec<Step> {
        let title = self.title(name);
        let system = self.details(name).kinds.iter().any(|k| {
            matches!(
                k,
                crate::backend::InstallKind::Rpm | crate::backend::InstallKind::Deb
            )
        });
        let mut steps = vec![step(
            format!("If {title} is open it is not removed; close it first."),
            Risk::Look,
            Actor::Program,
        )];
        if system {
            steps.push(step(
                format!("The package {title} is removed from the system. You confirm the administrator password."),
                Risk::System,
                Actor::Person,
            ));
        } else {
            steps.push(step(
                format!("All files of {title}, including older versions, are deleted from the store's folder."),
                Risk::Install,
                Actor::Program,
            ));
        }
        steps.push(step(
            "The commands and menu entries the store made are removed; nothing else is touched."
                .to_owned(),
            Risk::Install,
            Actor::Program,
        ));
        steps.push(step(
            "It is taken out of the store's list. Your own data stays.".to_owned(),
            Risk::Open,
            Actor::Program,
        ));
        steps
    }

    /// Asks before an install or update: the steps, the highest risk, the two answers. Not while something runs.
    pub fn ask_install(&mut self, name: &str) {
        // A locked package (a kind this store version does not install) is never asked about, whoever calls.
        if self.is_locked(name) {
            self.problems = vec![super::types::Problem::Locked(name.to_owned())];
            return;
        }
        // A package that is being installed is not installed a second time.
        if self.jobs.iter().any(|j| j.name == name && !j.is_finished()) {
            self.problems
                .push(super::types::Problem::Busy(name.to_owned()));
            return;
        }
        let update = self
            .cards()
            .iter()
            .find(|c| c.name == name)
            .is_some_and(|c| c.state == CardState::UpdateAvailable);
        let steps = self.install_steps(name, update);
        if steps.is_empty() {
            return;
        }
        let heading = if update {
            w::PREVIEW_UPDATE
        } else {
            w::PREVIEW_INSTALL
        };
        self.front.preview = Some(Preview {
            action: if update {
                PreviewAction::Update
            } else {
                PreviewAction::Install
            },
            names: vec![name.to_owned()],
            title: words::fill(heading, &[("title", &self.title(name))]),
            steps,
            warnings: self.need_warnings(name),
            facts: self.facts_line(name),
            notice: self.password_notice(&[name.to_owned()]),
        });
    }

    /// Asks before "Update all": one preview that lists the steps once and names the programs.
    pub fn ask_update_all(&mut self) {
        let names: Vec<String> = self
            .available
            .iter()
            .filter(|r| r.action == super::types::Action::Update)
            .filter(|r| {
                !self
                    .jobs
                    .iter()
                    .any(|j| j.name == r.package.name && !j.is_finished())
            })
            .map(|r| r.package.name.clone())
            .collect();
        let Some(first) = names.first() else {
            return;
        };
        let warnings: Vec<Warning> = names.iter().flat_map(|n| self.need_warnings(n)).collect();
        let steps = self.install_steps(first, true);
        self.front.preview = Some(Preview {
            action: PreviewAction::Update,
            title: words::fill(w::PREVIEW_MANY, &[("count", &names.len().to_string())]),
            warnings,
            facts: self.facts_line(first),
            notice: self.password_notice(&names),
            names,
            steps,
        });
    }

    /// Asks before taking `name` away (only an installed program).
    pub fn ask_remove_preview(&mut self, name: &str) {
        if !self.installed.iter().any(|r| r.name == name) {
            return;
        }
        self.front.preview = Some(Preview {
            action: PreviewAction::Remove,
            names: vec![name.to_owned()],
            title: words::fill(w::PREVIEW_REMOVE, &[("title", &self.title(name))]),
            steps: self.remove_steps(name),
            warnings: Vec::new(),
            facts: String::new(),
            notice: None,
        });
    }

    /// The question that waits for an answer.
    #[must_use]
    pub fn preview(&self) -> Option<&Preview> {
        self.front.preview.as_ref()
    }

    /// The person said "not now".
    pub fn cancel_preview(&mut self) {
        self.front.preview = None;
    }

    /// The person said yes: the question goes away and what to do is handed out (names), once.
    pub fn take_preview(&mut self) -> Option<Preview> {
        self.front.preview.take()
    }
}
