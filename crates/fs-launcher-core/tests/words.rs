//! Tests of the sentences: all filled by name, none with positional `{}`.
use fs_launcher_core::words::{SIGNATURE_REJECTED, fill, source_problem, update_problem};
use fs_launcher_core::{SourceError, UpdateError};

#[test]
fn every_problem_has_a_sentence_without_positional_braces() {
    let mut all = vec![SIGNATURE_REJECTED];
    for e in [
        SourceError::ReferenceSource,
        SourceError::NotFound,
        SourceError::Duplicate,
    ] {
        all.push(source_problem(&e));
    }
    for e in [
        UpdateError::Running,
        UpdateError::RolledBack,
        UpdateError::RollbackFailed,
    ] {
        all.push(update_problem(&e));
    }
    assert!(all.iter().all(|s| !s.is_empty() && !s.contains("{}")));
}

#[test]
fn named_placeholders_are_filled_whole() {
    let text = fill(
        update_problem(&UpdateError::Running),
        &[("name", "Messenger")],
    );
    assert!(text.starts_with("Messenger is running"));
    assert!(!text.contains('{'));
}
