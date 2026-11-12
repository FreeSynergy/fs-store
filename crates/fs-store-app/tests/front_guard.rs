//! Small guards of the window: a package is not installed twice at once, a locked "Open" says why, the question before an install says
//! what is missing, and the blocks with nothing to show are one line. Each has a counter-proof.

use std::sync::Arc;

use fs_store_app::demo::DemoBackend;
use fs_store_app::model::{PreviewAction, StoreModel};
use fs_store_app::words_front::{EmptyBlocks, Lang};

fn model() -> StoreModel {
    // the words are English in these tests, whatever language the machine has
    static ONCE: std::sync::Once = std::sync::Once::new();
    // SAFETY: every test of this file sets the same value before it reads anything, and nothing else reads the environment here.
    ONCE.call_once(|| unsafe { std::env::set_var("FS_LANG", "en") });
    let mut m = StoreModel::new(Arc::new(DemoBackend::default()));
    m.refresh();
    m
}

#[test]
fn a_package_that_is_being_installed_is_not_started_twice_and_another_one_still_can() {
    let mut m = model();
    assert!(m.begin_job("fs-wiki").is_some());
    // twice: refused, and told
    assert!(m.begin_job("fs-wiki").is_none());
    m.ask_install("fs-wiki");
    assert!(m.preview().is_none(), "no question for what is running");
    assert_ne!(m.problem_lines().len(), 0);
    // counter-proof: another package is asked as usual
    m.ask_install("fs-people-none-such");
    m.ask_install("fs-local-test");
    assert_eq!(m.preview().unwrap().names, ["fs-local-test"]);
    assert_eq!(m.preview().unwrap().action, PreviewAction::Install);
}

#[test]
fn update_all_leaves_out_what_runs_and_asks_for_the_rest() {
    let mut m = model();
    assert!(m.begin_job("fs-tasks").is_some());
    m.ask_update_all();
    assert_eq!(m.preview().unwrap().names, ["kanidm"]);
    // counter-proof: with nothing running both are in it
    let mut m = model();
    m.ask_update_all();
    assert_eq!(m.preview().unwrap().names.len(), 2);
}

#[test]
fn open_is_locked_with_a_reason_for_a_component_and_not_for_a_program() {
    let m = model();
    let card = |n: &str| m.cards().into_iter().find(|c| c.name == n).unwrap();
    assert!(card("kanidm").open_blocked.unwrap().contains("no window"));
    assert!(
        card("fs-people").open_blocked.is_none(),
        "a program can be opened"
    );
    assert!(fs_store_app::words_front::no_window(Lang::De).contains("Komponente"));
}

#[test]
fn the_question_before_an_install_says_what_nobody_offers() {
    let mut m = model();
    m.ask_install("fs-wiki");
    let p = m.preview().unwrap();
    assert_eq!(p.warnings.len(), 1);
    assert!(p.warnings[0].missing);
    assert!(
        p.warnings[0].text.contains("texts.blocks")
            && p.warnings[0].text.contains("offered by no package")
    );
    // two missing needs of tasks: only `groups` is missing (people.persons has a provider)
    m.cancel_preview();
    m.ask_install("fs-tasks");
    let t = m.preview().unwrap();
    assert_eq!(t.warnings.len(), 1);
    assert!(t.warnings[0].text.contains("groups"));
    // counter-proof: a package that needs nothing has no warning
    m.cancel_preview();
    m.ask_install("fs-local-test");
    assert_eq!(m.preview().unwrap().warnings.len(), 0);
}

#[test]
fn the_blocks_with_nothing_to_show_are_one_line() {
    let m = model();
    // a program with nothing offered, nothing needed, nothing extra and no pictures
    let p = m.package_page("fs-mystery").unwrap();
    assert_eq!(
        p.nothing_line.as_deref(),
        Some("Offers nothing \u{b7} Needs nothing \u{b7} Uses nothing extra \u{b7} No pictures")
    );
    // counter-proof: kanidm offers and needs and can use things, so only the empty part is named
    let k = m.package_page("kanidm").unwrap();
    assert_eq!(k.nothing_line.as_deref(), Some("No pictures"));
    let de = fs_store_app::words_front::nothing_line(
        Lang::De,
        EmptyBlocks {
            offers: true,
            uses: true,
            ..EmptyBlocks::default()
        },
    )
    .unwrap();
    assert_eq!(de, "Bietet nichts an \u{b7} Nutzt nichts zus\u{e4}tzlich");
    assert!(fs_store_app::words_front::nothing_line(Lang::En, EmptyBlocks::default()).is_none());
}
