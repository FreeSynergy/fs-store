#![allow(clippy::assert_is_empty)] // test data
//! "Cancel" right before the rename, and one install of a package at a time across processes. Each with a counter-proof.

mod cancel_support;
mod common;

use std::fs;

use cancel_support::*;
use common::verified;
use fs_store_core::*;
use fs_store_engine::service::InstallFailure;

#[test]
fn cancel_right_before_the_folder_takes_its_name_leaves_the_old_version_active() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    install(&root, "1.0.0");
    let before = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );
    let bytes = tarball(3, "1.1.0");
    // three entries look at the token (looks 1 to 3), the fourth look is the one just before the rename: the token sets itself there
    let mut stopped = fs_store_engine::TarGz::from_bytes(&root, bytes.clone())
        .with_cancel(CancelToken::after_looks(4));
    let result = run_update(
        &Update {
            package: &verified("prog", v(1, 1, 0)),
            previous: Some(v(1, 0, 0)),
        },
        &mut stopped,
        &Idle,
    );
    assert!(
        matches!(result, Err(UpdateError::RolledBack(ref c)) if c.0 == CANCELLED),
        "{result:?}"
    );
    let after = (
        names(&root.join("packages/prog")),
        fs::read_link(root.join("current/prog")).unwrap(),
    );
    assert_eq!(
        after, before,
        "the new folder never got its name, no temporary folder is left, the old link stays"
    );
    // counter-proof: one look later (the fifth) nothing stops it, and the same bytes install
    let mut free =
        fs_store_engine::TarGz::from_bytes(&root, bytes).with_cancel(CancelToken::after_looks(5));
    run_update(
        &Update {
            package: &verified("prog", v(1, 1, 0)),
            previous: Some(v(1, 0, 0)),
        },
        &mut free,
        &Idle,
    )
    .unwrap();
    assert_eq!(names(&root.join("packages/prog")), ["1.0.0", "1.1.0"]);
}

fn v(a: u64, b: u64, c: u64) -> Version {
    Version::new(a, b, c)
}

#[test]
fn only_one_install_of_a_package_runs_at_a_time_across_processes_and_a_free_lock_lets_it_run() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("root");
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    let plan = i.plan(&catalog, &sig, "prog", None).unwrap();
    let prepared = i
        .check(
            &plan,
            i.download_controlled(&plan, &CancelToken::new(), &|_| {})
                .unwrap(),
        )
        .unwrap();
    // another process holds the lock of the package (here: another holder of the same lock file)
    let held = fs_store_engine::PackageLock::acquire(&root, "prog")
        .unwrap()
        .unwrap();
    assert!(
        fs_store_engine::PackageLock::acquire(&root, "prog")
            .unwrap()
            .is_none(),
        "a second holder is refused"
    );
    let done = i.commit_all(vec![prepared]);
    assert!(
        matches!(done[0].1, Err(InstallFailure::Locked)),
        "{:?}",
        done[0].1
    );
    assert!(
        !root.join("current").exists() && !root.join("packages").exists(),
        "nothing was written"
    );
    // counter-proof: the lock let go, the same install runs
    drop(held);
    let (fetch, catalog, sig) = setup("1.0.0", 3);
    let i = installer(&root, &fetch);
    i.install(&catalog, &sig, "prog", None).unwrap();
    assert!(root.join("current/prog").exists());
}
