//! "Cancel" and the progress of a download in the model: what the person may stop, and that a stop leaves the card as it was.

mod common;

use common::{fake, model, v};
use fs_store_app::backend::{JobError, Phase};
use fs_store_core::Progress;

#[test]
fn a_job_can_be_stopped_while_it_downloads_and_the_stop_leaves_no_job_and_no_problem() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-tasks").unwrap();
    assert!(m.can_cancel("fs-tasks"));
    let token = m.job_cancel("fs-tasks").unwrap();
    assert!(!token.is_cancelled());
    assert!(m.cancel_job("fs-tasks"));
    assert!(token.is_cancelled(), "the worker sees the stop");
    // the worker answers: cancelled, nothing changed
    m.prepared("fs-tasks", Err(JobError::Cancelled));
    assert_eq!(m.jobs().len(), 0);
    assert_eq!(
        m.problem_lines().len(),
        0,
        "a stop the person asked for is not a problem"
    );
    let card = m
        .cards()
        .into_iter()
        .find(|c| c.name == "fs-tasks")
        .unwrap();
    assert!(
        card.working.is_none() && card.buttons.install,
        "the card is as it was"
    );
}

#[test]
fn a_finished_job_or_an_unknown_name_cannot_be_stopped() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(!m.cancel_job("fs-tasks"), "no job");
    m.run_job("fs-theme");
    assert!(!m.can_cancel("fs-theme"));
    assert!(!m.cancel_job("fs-theme"));
}

#[test]
fn while_installing_only_the_unpacked_archive_can_be_stopped() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-tasks").unwrap();
    m.observe_job("fs-tasks", Phase::Verified);
    assert!(m.can_cancel("fs-tasks"));
    m.observe_job("fs-tasks", Phase::Installing);
    // the fake backend installs the archive (the default `system_format`), which unpacks beside the target and can stop
    assert!(m.can_cancel("fs-tasks"));
    assert!(
        m.cards()
            .iter()
            .find(|c| c.name == "fs-tasks")
            .unwrap()
            .can_cancel
    );
}

#[test]
fn the_download_says_how_far_it_is_in_bytes_and_a_total_it_does_not_know_is_not_made_up() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.begin_job("fs-tasks").unwrap();
    let card = |m: &fs_store_app::model::StoreModel| {
        m.cards()
            .into_iter()
            .find(|c| c.name == "fs-tasks")
            .unwrap()
    };
    assert!(card(&m).progress().unwrap().1.starts_with("Waiting"));
    m.observe_progress(
        "fs-tasks",
        Progress {
            done: 48 << 20,
            total: Some(131 << 20),
        },
    );
    let (share, line) = card(&m).progress().unwrap();
    assert_eq!(line, "Downloading 48 MiB of 131 MiB \u{b7} 37 %");
    assert!((share - 0.366).abs() < 0.01);
    m.observe_progress(
        "fs-tasks",
        Progress {
            done: 3 << 20,
            total: None,
        },
    );
    assert_eq!(card(&m).progress().unwrap().1, "Downloading 3.0 MiB");
    let _ = v(1, 0, 0);
}
