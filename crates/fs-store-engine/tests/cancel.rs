#![allow(clippy::assert_is_empty)] // test data
//! "Cancel": where it may stop and what is left. Every test has a counter-proof: after the stop the machine is as it was before
//! (the folder of the old version, the `current` link, no half unpacked tree, no leftover temporary folder), and without the stop
//! the same data installs.

mod cancel_support;

use std::fs;

use cancel_support::*;
use fs_store_core::*;
use fs_store_engine::service::InstallFailure;

#[test]
fn cancel_in_the_middle_of_a_download_changes_nothing_and_without_it_the_same_data_installs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    let plan = i.plan(&catalog, &sig, "prog", None).unwrap();
    // the token sets itself at the third look: two pieces are read, then the stop
    let stopped = i.download_controlled(&plan, &CancelToken::after_looks(3), &|_| {});
    assert!(
        matches!(stopped, Err(InstallFailure::Cancelled)),
        "{stopped:?}"
    );
    assert_eq!(names(&root), Vec::<String>::new(), "nothing on the machine");
    // counter-proof: the same data, not cancelled, installs; progress was reported in pieces
    let ok = i
        .download_controlled(&plan, &CancelToken::new(), &|_| {})
        .unwrap();
    let prepared = i.check(&plan, ok).unwrap();
    assert_eq!(
        i.commit_all(vec![prepared])[0].1.as_ref().unwrap(),
        &Version::new(1, 0, 0)
    );
    let seen = fetch.1.lock().unwrap();
    assert!(
        seen.iter().any(|p| p.share() == Some(0.5)) && seen.iter().any(|p| p.share() == Some(1.0))
    );
}

#[test]
fn cancel_after_the_checks_and_before_the_install_writes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    let plan = i.plan(&catalog, &sig, "prog", None).unwrap();
    let cancel = CancelToken::new();
    let prepared = i
        .check(
            &plan,
            i.download_controlled(&plan, &cancel, &|_| {}).unwrap(),
        )
        .unwrap();
    cancel.cancel();
    let done = i.commit_all(vec![prepared]);
    assert!(matches!(done[0].1, Err(InstallFailure::Cancelled)));
    assert_eq!(names(&root), Vec::<String>::new());
}

#[test]
fn cancel_in_the_middle_of_unpacking_leaves_the_old_version_current_and_no_half_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    install(&root, "1.0.0");
    let before = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );

    let (fetch, catalog, sig) = setup("1.1.0", 40);
    let i = installer(&root, &fetch);
    let plan = i
        .plan(&catalog, &sig, "prog", Some(Version::new(1, 0, 0)))
        .unwrap();
    let cancel = CancelToken::new();
    let prepared = i
        .check(
            &plan,
            i.download_controlled(&plan, &cancel, &|_| {}).unwrap(),
        )
        .unwrap();
    // the token sets itself at the tenth look at it: the unpacking has written part of the tree by then
    let prepared = prepared_with(prepared, CancelToken::after_looks(10));
    let done = i.commit_all(vec![prepared]);
    assert!(
        matches!(done[0].1, Err(InstallFailure::Cancelled)),
        "{:?}",
        done[0].1
    );

    let after = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );
    assert_eq!(
        after, before,
        "the old version is current, the new one has no folder, no temporary folder is left"
    );
    assert!(
        root.join("current/prog/bin/prog0").is_file(),
        "the old files are in place"
    );
    assert_eq!(
        fs::read_to_string(root.join("current/prog/bin/prog0")).unwrap(),
        "1.0.0"
    );

    // counter-proof: the same unpacking without the stop installs the new version completely
    let (fetch, catalog, sig) = setup("1.1.0", 40);
    let i = installer(&root, &fetch);
    i.install(&catalog, &sig, "prog", Some(Version::new(1, 0, 0)))
        .unwrap();
    assert_eq!(names(&root.join("packages/prog")), ["1.0.0", "1.1.0"]);
    assert_eq!(
        fs::read_dir(root.join("packages/prog/1.1.0/bin"))
            .unwrap()
            .count(),
        40
    );
}

/// Gives a prepared package another token (the test's way to cancel at a chosen look).
fn prepared_with(p: fs_store_engine::Prepared, token: CancelToken) -> fs_store_engine::Prepared {
    p.with_cancel(token)
}
