//! The store's own list of files: written at install, checked by `verify`, asked by `owner`, made afterwards for old installs.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, Verdict, owner, record_install, verify};
use fs_store_sources::{Basis, ManifestStore};

fn install(tmp: &Path, name: &str, v: Version, content: &str) -> PathBuf {
    let root = tmp.join("root");
    let a = archive(tmp, &format!("{name}-{v}"), content);
    run_update(
        &Update {
            package: &verified(name, v),
            previous: None,
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    root
}

fn none(_: &str) -> String {
    String::new()
}

#[test]
fn the_install_writes_path_package_version_source_signer_and_checksum() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(1, 2, 3), "one");
    assert!(record_install(&root, "prog", "1.2.3", "https://example.org/cat", "ABCD").unwrap());
    let m = ManifestStore::new(&root).load("prog").unwrap().unwrap();
    assert_eq!((m.name.as_str(), m.version.as_str()), ("prog", "1.2.3"));
    assert_eq!(
        (m.source.as_str(), m.signer.as_str()),
        ("https://example.org/cat", "ABCD")
    );
    assert_eq!(m.basis, Basis::Installed);
    assert_eq!(m.files.len(), 1);
    assert_eq!(m.files[0].path, "bin/prog");
    assert!(m.files[0].hash.starts_with("sha256:") && m.files[0].hash.len() == 7 + 64);
    assert_eq!((m.files[0].size, m.files[0].mode), (3, 0o755));
}

#[test]
fn a_package_of_the_package_manager_has_no_list() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!record_install(tmp.path(), "prog", "1.0.0", "s", "").unwrap());
    assert_eq!(ManifestStore::new(tmp.path()).names().unwrap().len(), 0);
}

#[test]
fn verify_sees_a_changed_and_a_missing_file_and_only_ours() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(1, 0, 0), "one");
    record_install(&root, "prog", "1.0.0", "s", "K").unwrap();
    let reports = verify(&root, None, &none).unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].is_clean() && !reports[0].made_now);

    let file = root.join("packages/prog/1.0.0/bin/prog");
    fs::write(&file, "changed").unwrap();
    let r = &verify(&root, Some("prog"), &none).unwrap()[0];
    assert_eq!(r.files, [("bin/prog".to_owned(), Verdict::Changed)]);
    assert!(!r.is_clean());

    fs::remove_file(&file).unwrap();
    let r = &verify(&root, None, &none).unwrap()[0];
    assert_eq!(r.files[0].1, Verdict::Missing);

    assert!(verify(&root, Some("other"), &none).is_err());
}

#[test]
fn an_old_install_without_a_list_gets_one_marked_as_made_afterwards() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(0, 9, 0), "old");
    assert_eq!(ManifestStore::new(&root).names().unwrap().len(), 0);
    let src = |_: &str| "https://example.org/cat".to_owned();
    let first = verify(&root, None, &src).unwrap();
    assert!(first[0].made_now && first[0].is_clean());
    let m = ManifestStore::new(&root).load("prog").unwrap().unwrap();
    assert_eq!(m.basis, Basis::Retroactive);
    assert_eq!(m.source, "https://example.org/cat");
    assert_eq!(m.signer, "");
    let second = verify(&root, None, &src).unwrap();
    assert!(!second[0].made_now);
    assert_eq!(second[0].manifest.basis, Basis::Retroactive);
}

#[test]
fn owner_finds_the_package_through_a_link_and_says_how_sure_it_is() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(1, 0, 0), "one");
    record_install(&root, "prog", "1.0.0", "s", "K").unwrap();
    let bin = tmp.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(root.join("current/prog/bin/prog"), bin.join("prog")).unwrap();

    let o = owner(&root, &bin.join("prog")).unwrap().unwrap();
    assert_eq!(o.manifest.name, "prog");
    assert_eq!(o.verdict, Verdict::Intact);
    assert!(o.proof().starts_with("recorded by the store"));

    fs::write(root.join("packages/prog/1.0.0/bin/prog"), "evil").unwrap();
    let o = owner(&root, &bin.join("prog")).unwrap().unwrap();
    assert_eq!(o.verdict, Verdict::Changed);
    assert!(o.proof().contains("changed"));

    assert!(owner(&root, &tmp.path().join("nothing")).unwrap().is_none());
    let foreign = tmp.path().join("foreign");
    fs::write(&foreign, "x").unwrap();
    assert!(owner(&root, &foreign).unwrap().is_none());
}

#[test]
fn a_list_made_afterwards_is_never_called_original() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(1, 0, 0), "one");
    verify(&root, None, &none).unwrap();
    let o = owner(&root, &root.join("current/prog/bin/prog"))
        .unwrap()
        .unwrap();
    assert!(o.proof().contains("afterwards") && o.proof().contains("not the original"));
}

fn list_clean(root: &Path) -> bool {
    verify(root, None, &none).unwrap()[0].is_clean()
}

#[test]
fn a_reinstall_over_a_changed_tree_repairs_it_and_never_lists_the_changed_content_as_original() {
    let tmp = tempfile::tempdir().unwrap();
    let root = install(tmp.path(), "prog", Version::new(1, 0, 0), "original");
    record_install(&root, "prog", "1.0.0", "s", "K").unwrap();
    let file = root.join("packages/prog/1.0.0/bin/prog");
    fs::write(&file, "evil").unwrap();
    fs::write(root.join("packages/prog/1.0.0/bin/stranger"), "x").unwrap();
    // the version folder is there already: it is NOT trusted, the archive is unpacked and compared
    let a = archive(tmp.path(), "again", "original");
    run_update(
        &Update {
            package: &verified("prog", Version::new(1, 0, 0)),
            previous: Some(Version::new(1, 0, 0)),
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "original",
        "the changed file is back as the archive has it"
    );
    assert!(
        !root.join("packages/prog/1.0.0/bin/stranger").exists(),
        "the stranger is gone with the old tree"
    );
    assert_eq!(
        names(&root.join("packages/prog")),
        ["1.0.0"],
        "no temporary or old folder is left"
    );
    record_install(&root, "prog", "1.0.0", "s", "K").unwrap();
    assert!(
        list_clean(&root),
        "clean now, and only because the tree is the archive's"
    );
    // counter-proof: the same list over the evil content would not be clean
    fs::write(&file, "evil").unwrap();
    assert!(!list_clean(&root));
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn listed(tmp: &Path) -> PathBuf {
    let root = install(tmp, "prog", Version::new(1, 0, 0), "original");
    record_install(&root, "prog", "1.0.0", "s", "K").unwrap();
    assert!(list_clean(&root), "counter-proof: untouched is clean");
    root
}

fn verdict(root: &Path) -> Verdict {
    verify(root, None, &none).unwrap()[0].files[0].1
}

#[test]
fn another_mode_a_setuid_bit_a_link_for_the_file_and_an_unreadable_file_are_each_told() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let root = listed(tmp.path());
    let file = root.join("packages/prog/1.0.0/bin/prog");

    fs::set_permissions(&file, fs::Permissions::from_mode(0o4755)).unwrap();
    assert_eq!(verdict(&root), Verdict::Mode, "setuid is never allowed");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        verdict(&root),
        Verdict::Mode,
        "other permission bits than listed"
    );
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(verdict(&root), Verdict::Intact);

    // the file replaced by a link to a copy with the same content
    let copy = tmp.path().join("copy");
    fs::copy(&file, &copy).unwrap();
    fs::remove_file(&file).unwrap();
    std::os::unix::fs::symlink(&copy, &file).unwrap();
    assert_eq!(verdict(&root), Verdict::Type);
    fs::remove_file(&file).unwrap();
    fs::copy(&copy, &file).unwrap();
    assert_eq!(verdict(&root), Verdict::Intact);

    // not readable (a test run by root reads it anyway, so then there is nothing to tell)
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    let v = verdict(&root);
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    if !fs::File::open(&file).is_ok_and(|_| v == Verdict::Mode) {
        assert_eq!(v, Verdict::Unreadable);
    }
    // gone is gone
    fs::remove_file(&file).unwrap();
    assert_eq!(verdict(&root), Verdict::Missing);
}

#[test]
fn a_stranger_in_the_package_folder_is_an_extra_warning_and_not_a_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let root = listed(tmp.path());
    let dir = root.join("packages/prog/1.0.0");
    fs::write(dir.join("bin/added"), "x").unwrap();
    std::os::unix::fs::symlink("/etc/hostname", dir.join("bin/link")).unwrap();
    let r = &verify(&root, None, &none).unwrap()[0];
    assert!(r.is_clean(), "every listed file is as it was");
    assert_eq!(r.extra, ["bin/added", "bin/link"]);
}
