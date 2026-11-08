mod common;

use std::fs;

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, link_facets, unlink_facets};

fn installed(tmp: &std::path::Path) -> std::path::PathBuf {
    let root = tmp.join("root");
    let a = archive(tmp, "one", "v1");
    run_update(
        &Update {
            package: &verified("people", Version::new(1, 0, 0)),
            previous: None,
        },
        &mut TarGz::new(&root, &a),
        &Idle,
    )
    .unwrap();
    let exe = root.join("current/people/people-app");
    fs::write(&exe, "#!/bin/sh\n").unwrap();
    root
}

#[test]
fn facets_become_links_and_menu_entries_and_go_away_again() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    let facets = vec!["contacts".to_owned()];
    assert_eq!(
        link_facets(&root, "people", &facets, &bin, &apps).unwrap(),
        facets
    );
    assert_eq!(
        fs::read_link(bin.join("contacts")).unwrap(),
        root.join("current/people/people-app")
    );
    let entry = fs::read_to_string(apps.join("net.freesynergy.contacts.desktop")).unwrap();
    assert!(entry.contains("Name=FreeSynergy.Contacts") && entry.contains("Exec=contacts"));
    let program = fs::read_to_string(apps.join("net.freesynergy.people.desktop")).unwrap();
    assert!(program.contains("Name=FreeSynergy.People") && program.contains("Exec=people-app"));
    unlink_facets(&root, "people", &facets, &bin, &apps).unwrap();
    assert!(fs::symlink_metadata(bin.join("contacts")).is_err());
    assert_eq!(fs::read_dir(&apps).unwrap().count(), 0);
}

#[test]
fn a_command_name_that_is_not_ours_is_never_taken_over() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    fs::create_dir(&bin).unwrap();
    fs::write(bin.join("contacts"), "mine").unwrap();
    let facets = vec!["contacts".to_owned(), "../evil".to_owned()];
    assert_eq!(
        link_facets(&root, "people", &facets, &bin, &apps)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(fs::read_to_string(bin.join("contacts")).unwrap(), "mine");
    unlink_facets(&root, "people", &facets, &bin, &apps).unwrap();
    assert_eq!(fs::read_to_string(bin.join("contacts")).unwrap(), "mine");
}
