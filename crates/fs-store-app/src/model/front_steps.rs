//! Pattern: Value object (a step of the question before any work, with how far it reaches and who does it, and the question itself).

use crate::words;
use crate::words_front as w;

/// How far a step reaches, lowest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    /// Only reads.
    Look,
    /// Opens something of ours.
    Open,
    /// Starts or ends a process.
    Start,
    /// Writes below the person's own folders.
    Install,
    /// Changes the system for everyone (needs the administrator password).
    System,
}

impl Risk {
    /// The word for the person.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Risk::Look => w::RISK_LOOK,
            Risk::Open => w::RISK_OPEN,
            Risk::Start => w::RISK_START,
            Risk::Install => w::RISK_INSTALL,
            Risk::System => w::RISK_SYSTEM,
        }
    }
}

/// Who does a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// The store itself.
    Program,
    /// The system (dnf, apt).
    Machine,
    /// The person (confirms the password).
    Person,
}

impl Actor {
    /// The word for the person.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Actor::Program => w::ACTOR_PROGRAM,
            Actor::Machine => w::ACTOR_MACHINE,
            Actor::Person => w::ACTOR_PERSON,
        }
    }
}

/// One step, as a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// What will happen.
    pub text: String,
    /// How far it reaches.
    pub risk: Risk,
    /// Who does it.
    pub actor: Actor,
}

/// What the preview is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewAction {
    /// A first install.
    Install,
    /// A newer version.
    Update,
    /// Take away.
    Remove,
}

/// The question before the work starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// What is asked.
    pub action: PreviewAction,
    /// The packages (more than one for "Update all").
    pub names: Vec<String>,
    /// The heading.
    pub title: String,
    /// The steps, in order.
    pub steps: Vec<Step>,
}

impl Preview {
    /// The highest risk of all steps.
    #[must_use]
    pub fn risk(&self) -> Risk {
        self.steps
            .iter()
            .map(|s| s.risk)
            .max()
            .unwrap_or(Risk::Look)
    }

    /// The line "Highest risk: ...".
    #[must_use]
    pub fn risk_line(&self) -> String {
        words::fill(w::PREVIEW_RISK, &[("risk", self.risk().word())])
    }
}
