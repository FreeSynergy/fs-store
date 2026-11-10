mod common;

use std::fs;

use common::{Idle, archive, verified};
use fs_store_core::*;
use fs_store_engine::{TarGz, link_facets, unlink_facets};
use fs_store_platform::EntryState;

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
    common::make_program(&root.join("current/people"), "people-app");
    root
}

fn platform() -> &'static fs_store_platform::Platform {
    fs_store_platform::current()
}

#[test]
fn facets_become_commands_and_menu_entries_and_go_away_again() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    let facets = vec!["contacts".to_owned()];
    assert_eq!(
        link_facets(&root, "people", &facets, &bin, &apps).unwrap(),
        vec!["people".to_owned(), "contacts".to_owned()]
    );
    let program = root
        .join("current/people")
        .join(platform().paths.program_file_name("people-app"));
    assert_eq!(
        platform().commands.command_target(&bin, "contacts"),
        Some(program)
    );
    for (id, title) in [
        ("net.freesynergy.contacts", "FreeSynergy.Contacts"),
        ("net.freesynergy.people", "FreeSynergy.People"),
    ] {
        assert_eq!(
            platform().menu.entry_state(&apps, id, title),
            EntryState::Ours
        );
    }
    unlink_facets(&root, "people", &facets, &bin, &apps).unwrap();
    assert!(!platform().commands.command_exists(&bin, "contacts"));
    assert_eq!(
        platform()
            .menu
            .entry_state(&apps, "net.freesynergy.people", "FreeSynergy.People"),
        EntryState::Missing
    );
}

#[cfg(unix)]
#[test]
fn the_linux_entry_names_the_program_and_its_command() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    link_facets(&root, "people", &["contacts".to_owned()], &bin, &apps).unwrap();
    let entry = fs::read_to_string(apps.join("net.freesynergy.contacts.desktop")).unwrap();
    assert!(entry.contains("Name=FreeSynergy.Contacts") && entry.contains("Exec=contacts"));
}

#[test]
fn a_command_name_that_is_not_ours_is_never_taken_over() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    fs::create_dir(&bin).unwrap();
    fs::write(platform().commands.command_path(&bin, "contacts"), "mine").unwrap();
    let facets = vec!["contacts".to_owned(), "../evil".to_owned()];
    assert_eq!(
        link_facets(&root, "people", &facets, &bin, &apps)
            .unwrap()
            .len(),
        1
    );
    let file = platform().commands.command_path(&bin, "contacts");
    assert_eq!(fs::read_to_string(&file).unwrap(), "mine");
    unlink_facets(&root, "people", &facets, &bin, &apps).unwrap();
    let file = platform().commands.command_path(&bin, "contacts");
    assert_eq!(fs::read_to_string(&file).unwrap(), "mine");
}

#[cfg(unix)] // Windows shortcuts take the icon of the program
#[test]
fn a_stored_icon_goes_into_the_menu_entries_of_the_program_and_its_facets() {
    let tmp = tempfile::tempdir().unwrap();
    let root = installed(tmp.path());
    fs::create_dir_all(root.join("icons")).unwrap();
    fs::write(root.join("icons/people.png"), b"x").unwrap();
    let (bin, apps) = (tmp.path().join("bin"), tmp.path().join("apps"));
    link_facets(&root, "people", &["contacts".to_owned()], &bin, &apps).unwrap();
    let want = format!("Icon={}", root.join("icons/people.png").display());
    for name in ["people", "contacts"] {
        let entry =
            fs::read_to_string(apps.join(format!("net.freesynergy.{name}.desktop"))).unwrap();
        assert!(entry.contains(&want), "{name}: {entry}");
    }
}
