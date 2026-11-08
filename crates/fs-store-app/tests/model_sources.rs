//! Tests of the view-model: adding, switching off and removing sources.
#![allow(clippy::assert_is_empty)]

mod common;

use fs_store_app::model::Problem;
use fs_store_core::SourceError;

use common::{fake, model};

#[test]
fn reference_source_cannot_be_removed_but_can_be_switched_off() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.remove_source("freesynergy");
    assert_eq!(
        m.problems(),
        [Problem::Source(SourceError::ReferenceSource)]
    );
    assert_eq!(f.state.lock().unwrap().remove_calls, 0);
    assert_eq!(m.sources().len(), 1);
    assert_eq!(
        m.problem_lines(),
        ["The FreeSynergy source cannot be removed. You can switch it off instead."]
    );
    m.set_enabled("freesynergy", false);
    assert!(!m.sources()[0].enabled);
    assert!(m.available().is_empty());
    assert!(m.problems().is_empty());
}

#[test]
fn add_and_remove_a_source() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.set_new_source("   ");
    m.add_source();
    assert_eq!(m.sources().len(), 1);
    m.set_new_source(" https://example.org/mine ");
    m.add_source();
    assert_eq!(m.sources().len(), 2);
    assert_eq!(m.new_source(), "");
    m.set_new_source("https://example.org/mine");
    m.add_source();
    assert_eq!(m.problems(), [Problem::Source(SourceError::Duplicate)]);
    m.remove_source("mine");
    assert_eq!(m.sources().len(), 1);
    assert_eq!(f.state.lock().unwrap().remove_calls, 1);
}
