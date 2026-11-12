//! The page of one package and the question before any work: every fact in words, nothing guessed, the steps with their risk.

use std::sync::Arc;

use fs_store_app::demo::DemoBackend;
use fs_store_app::model::{DescriptionView, PreviewAction, Risk, StoreModel};

fn model() -> StoreModel {
    let mut m = StoreModel::new(Arc::new(DemoBackend::default()));
    m.refresh();
    m
}

#[test]
fn a_page_shows_version_with_date_size_origin_trust_changes_and_pictures() {
    let m = model();
    let p = m.package_page("fs-tasks").unwrap();
    assert_eq!(p.version, "0.1.0 -> 0.2.0 (2026-11-09)");
    assert_eq!(p.size, "14 MiB");
    assert_eq!(p.trust_word, "Our key only");
    assert!(p.trust_text.contains("maker gives no signature"));
    assert_eq!(p.releases[0].version, "0.2.0");
    assert_eq!(p.releases[1].date, "2026-10-21");
    assert_eq!(
        p.pictures,
        [("The board".to_owned(), "CC-BY-4.0".to_owned())]
    );
    assert!(p.pictures_note.contains("1 with a clear licence"));
    assert!(p.pictures_note.contains("1 not shown"));
    assert!(
        p.origin
            .iter()
            .any(|(l, v)| *l == "Maker" && v == "made by FreeSynergy")
    );
    assert!(p.icon_missing);
}

#[test]
fn a_mirrored_package_names_its_maker_the_makers_signature_and_the_source_code() {
    let p = model().package_page("kanidm").unwrap();
    assert_eq!(p.trust_word, "Maker and mirror");
    let has = |l: &str, v: &str| p.origin.iter().any(|(a, b)| *a == l && b.contains(v));
    assert!(has("Maker", "github.com/kanidm/kanidm"));
    assert!(has("Maker's signature", "kanidm.upstream.asc"));
    assert!(p.source_code.contains("kanidm-1.5.0-src"));
}

#[test]
fn a_description_can_point_to_a_wiki_block_and_is_never_loaded() {
    let p = model().package_page("fs-wiki").unwrap();
    assert_eq!(
        p.description,
        DescriptionView::Wiki("wiki:store/fs-wiki-intro".to_owned())
    );
}

#[test]
fn a_package_with_no_facts_says_unknown_everywhere() {
    let p = model().package_page("fs-mystery").unwrap();
    assert_eq!(p.size, "unknown");
    assert_eq!(p.trust_word, "unknown");
    assert!(p.trust_text.contains("does not say"));
    assert_eq!(p.releases.len(), 0);
    assert_eq!(p.pictures_note, "No pictures from the maker.");
    assert!(p.origin.iter().any(|(l, _)| *l == "Source"));
    assert!(model().package_page("nothing").is_none());
}

#[test]
fn an_install_says_what_it_will_do_step_by_step_with_the_highest_risk() {
    let mut m = model();
    m.ask_install("fs-wiki");
    let p = m.preview().unwrap();
    assert_eq!(p.action, PreviewAction::Install);
    assert_eq!(p.title, "Install FreeSynergy.Wiki: what will happen");
    assert!(p.steps[0].text.contains("signature is checked"));
    assert!(p.steps[1].text.contains("0.1.4 (9.2 MiB)"));
    assert_eq!(p.risk(), Risk::Install);
    assert_eq!(p.risk_line(), "Highest risk: install");
    assert!(p.steps.iter().all(|s| s.risk != Risk::System));
    // nothing is started by asking
    assert_eq!(m.jobs().len(), 0);
    assert!(m.take_preview().is_some());
    assert!(m.preview().is_none());
}

#[test]
fn an_update_also_ends_a_server_and_a_removal_says_that_data_stays() {
    let mut m = model();
    m.ask_install("fs-tasks");
    let p = m.preview().unwrap();
    assert_eq!(p.action, PreviewAction::Update);
    assert!(
        p.steps
            .iter()
            .any(|s| s.risk == Risk::Start && s.text.contains("background server"))
    );
    m.cancel_preview();
    m.ask_remove_preview("fs-tasks");
    let r = m.preview().unwrap();
    assert_eq!(r.action, PreviewAction::Remove);
    assert!(
        r.steps
            .iter()
            .any(|s| s.text.contains("Your own data stays"))
    );
    assert!(
        r.steps
            .iter()
            .any(|s| s.text.contains("including older versions"))
    );
    // a program that is not installed has nothing to remove
    m.cancel_preview();
    m.ask_remove_preview("fs-wiki");
    assert!(m.preview().is_none());
}

#[test]
fn update_all_is_one_question_naming_how_many() {
    let mut m = model();
    m.ask_update_all();
    let p = m.preview().unwrap();
    assert_eq!(p.names.len(), 2);
    assert_eq!(p.title, "Update 2 programs: what will happen to each");
}
