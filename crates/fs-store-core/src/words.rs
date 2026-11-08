//! Pattern: none (sentences as constants, plus one pure function to fill them).
//! Every text a person reads comes from here (coding rules 2a). Whole sentences with named
//! placeholders in `{braces}`; translation comes later (ADR 0053). The code never compares these texts.

use crate::source::SourceError;
use crate::update::UpdateError;

/// Sentence for a source problem. Placeholder: none.
#[must_use]
pub fn source_problem(error: &SourceError) -> &'static str {
    match error {
        SourceError::ReferenceSource => {
            "The FreeSynergy source cannot be removed. You can switch it off instead."
        }
        SourceError::NotFound => "This source is not in the list.",
        SourceError::Duplicate => "A source with this name is in the list already.",
    }
}

/// Sentence for an update problem. Placeholder: `{name}`.
#[must_use]
pub fn update_problem(error: &UpdateError) -> &'static str {
    match error {
        UpdateError::Running => "{name} is running and was not replaced. Close it and try again.",
        UpdateError::RolledBack(_) => "The update of {name} failed. The previous version is back.",
        UpdateError::RollbackFailed => {
            "The update of {name} failed and the previous version could not be restored."
        }
    }
}

/// Sentence for a download that failed the signature check. Placeholder: `{name}`.
pub const SIGNATURE_REJECTED: &str =
    "{name} was not installed: its signature could not be verified.";

/// Replaces each `{key}` by its value; unknown placeholders stay as they are.
#[must_use]
pub fn fill(sentence: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(sentence.to_owned(), |text, (key, value)| {
            text.replace(&format!("{{{key}}}"), value)
        })
}
