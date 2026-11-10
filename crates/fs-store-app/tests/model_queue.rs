//! Tests of the start page model: updates on the start page, the job list with several jobs, display names.

mod common;

use fs_store_app::backend::{InstalledRow, Phase};
use fs_store_app::model::Problem;

use common::{fake, model, v};

fn installed(name: &str, version: fs_store_core::Version) -> InstalledRow {
    InstalledRow {
        name: name.into(),
        version,
    }
}

#[test]
fn programs_are_shown_as_freesynergy_name() {
    let f = fake(Vec::new(), None);
    let m = model(&f);
    assert_eq!(m.title("fs-tasks"), "FreeSynergy.Tasks");
    for card in m.cards() {
        assert!(card.title.starts_with("FreeSynergy."), "{}", card.title);
    }
}

#[test]
fn no_sentence_shows_an_fs_id() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.run_job("fs-theme");
    assert_eq!(m.job_line(), "FreeSynergy.Theme 1.0.0 is installed.");
    m.run_job("fs-theme");
    for line in m.problem_lines().into_iter().chain(m.job_lines()) {
        assert!(!line.contains("fs-"), "{line}");
    }
}

#[test]
fn updates_are_listed_for_the_start_page() {
    let f = fake(vec![installed("fs-tasks", v(0, 6, 0))], None);
    let mut m = model(&f);
    let rows = m.update_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "FreeSynergy.Tasks");
    assert_eq!((rows[0].from, rows[0].to), (v(0, 6, 0), v(0, 6, 1)));
    assert!(!rows[0].queued);
    m.begin_job("fs-tasks");
    assert!(m.update_rows()[0].queued);
}

#[test]
fn several_jobs_are_in_one_list_and_nothing_blocks() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    assert!(m.begin_job("fs-theme").is_some());
    assert!(m.begin_job("fs-tasks").is_some());
    assert_eq!(m.jobs().len(), 2);
    assert_eq!(m.job_lines().len(), 2);
    assert_eq!(m.job_lines()[0], "FreeSynergy.Theme: waiting to start.");
    assert!(m.problems().iter().all(|p| !matches!(p, Problem::Busy(_))));
    // the same program twice is refused, the others stay possible
    assert!(m.begin_job("fs-theme").is_none());
    m.observe_job("fs-tasks", Phase::Verified);
    assert_eq!(m.jobs()[1].phase, Phase::Verified);
    assert_eq!(m.jobs()[0].phase, Phase::Pending);
}

#[test]
fn finished_jobs_can_be_cleared_and_running_ones_stay() {
    let f = fake(Vec::new(), None);
    let mut m = model(&f);
    m.run_job("fs-theme");
    m.begin_job("fs-tasks");
    m.clear_finished_jobs();
    assert_eq!(m.jobs().len(), 1);
    assert_eq!(m.jobs()[0].name, "fs-tasks");
}
