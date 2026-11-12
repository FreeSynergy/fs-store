//! A package this store version cannot install (a server, a kind it does not know) is listed and read, shown locked, and never installed:
//! not by the button, not by a call. The packages next to it work as before.

mod common;

use std::sync::Arc;

use fs_store_app::backend::PackageRow;
use fs_store_app::model::{Problem, Section, StoreModel};
use fs_store_core::{Kind, Version};

fn locked_row(name: &str, kind: Kind, reason: &str) -> PackageRow {
    PackageRow {
        kind,
        locked: Some(reason.into()),
        ..common::package(name, Version::new(0, 1, 0))
    }
}

fn model_with_locked() -> (StoreModel, Arc<common::Fake>) {
    // the words of the test are English
    // SAFETY: set once before any test reads it; every test of this file wants the same value
    unsafe { std::env::set_var("FS_LANG", "en") };
    let f = common::fake(Vec::new(), None);
    {
        let mut st = f.state.lock().unwrap();
        st.offered
            .push(common::package("fs-fine", Version::new(1, 0, 0)));
        st.offered
            .push(locked_row("fs-srv", Kind::Server, "coming soon"));
        st.offered.push(locked_row(
            "fs-odd",
            Kind::Unsupported,
            "not supported by this store version",
        ));
    }
    (common::model(&f), f)
}

#[test]
fn a_locked_card_has_a_reason_and_the_packages_next_to_it_have_none() {
    let (m, _f) = model_with_locked();
    let card = |n: &str| m.cards().into_iter().find(|c| c.name == n).unwrap();
    assert!(
        card("fs-srv")
            .install_blocked
            .unwrap()
            .contains("Coming soon")
    );
    assert!(
        card("fs-odd")
            .install_blocked
            .unwrap()
            .contains("Not supported by this store version")
    );
    // counter-proof
    assert!(card("fs-fine").install_blocked.is_none());
    assert!(m.is_locked("fs-srv") && !m.is_locked("fs-fine"));
}

#[test]
fn a_locked_package_is_never_installed_by_a_job_or_a_question() {
    let (mut m, f) = model_with_locked();
    assert!(m.begin_job("fs-srv").is_none());
    assert_eq!(m.problems(), [Problem::Locked("fs-srv".into())]);
    m.ask_install("fs-odd");
    assert!(m.preview().is_none());
    assert_eq!(f.state.lock().unwrap().runs.len(), 0);
    // counter-proof: the package next to them still starts a job
    assert!(m.begin_job("fs-fine").is_some());
}

#[test]
fn the_navigation_opens_a_section_for_servers_and_the_start_lists_unknown_kinds_apart() {
    let (mut m, _f) = model_with_locked();
    let servers = m.nav().into_iter().find(|n| n.label == "Servers").unwrap();
    assert_eq!((servers.count, servers.enabled), (1, true));
    assert_eq!(servers.note, Some("Coming soon"));
    m.open_section(Section::Kind(Kind::Server));
    assert_eq!(m.listing().len(), 1);
    let blocks = m.start_blocks();
    let unknown = blocks
        .iter()
        .find(|b| b.title == "Not supported by this store version")
        .unwrap();
    assert_eq!(unknown.cards[0].name, "fs-odd");
    // counter-proof: the unknown entry is in no kind block, and a catalog without one has no such block
    assert!(
        blocks
            .iter()
            .filter(|b| b.title != unknown.title)
            .all(|b| b.cards.iter().all(|c| c.name != "fs-odd"))
    );
}

#[test]
fn a_locked_update_is_not_counted_and_not_started_by_update_all() {
    unsafe { std::env::set_var("FS_LANG", "en") };
    let f = common::fake(
        vec![
            fs_store_app::backend::InstalledRow {
                name: "fs-old".into(),
                version: Version::new(1, 0, 0),
            },
            fs_store_app::backend::InstalledRow {
                name: "fs-fine".into(),
                version: Version::new(1, 0, 0),
            },
        ],
        None,
    );
    {
        let mut st = f.state.lock().unwrap();
        // a newer version is offered for both; the first wants a newer store
        st.offered.push(PackageRow {
            locked: Some("not supported by this store version".into()),
            ..common::package("fs-old", Version::new(2, 0, 0))
        });
        st.offered
            .push(common::package("fs-fine", Version::new(2, 0, 0)));
    }
    let mut m = common::model(&f);
    let started: Vec<String> = m.begin_all_updates().into_iter().map(|(_, n)| n).collect();
    assert_eq!(started, ["fs-fine"]);
    let updates = m
        .tab_counts()
        .into_iter()
        .find(|(t, _)| *t == fs_store_app::model::Tab::Updates);
    assert_eq!(updates.map(|(_, n)| n), Some(1));
}
