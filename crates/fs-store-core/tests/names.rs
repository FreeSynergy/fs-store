//! Tests of the one display-name function.

use fs_store_core::display_name;

#[test]
fn a_name_from_the_catalog_wins() {
    assert_eq!(
        display_name("fs-people", Some("FreeSynergy.People")),
        "FreeSynergy.People"
    );
    assert_eq!(display_name("fs-x", Some("  Odd Name ")), "Odd Name");
}

#[test]
fn without_a_catalog_name_fs_becomes_freesynergy() {
    assert_eq!(display_name("fs-people", None), "FreeSynergy.People");
    assert_eq!(display_name("fs-store", Some("")), "FreeSynergy.Store");
    assert_eq!(display_name("fs-tasks", Some("  ")), "FreeSynergy.Tasks");
}

#[test]
fn several_words_are_joined_with_capitals() {
    assert_eq!(
        display_name("fs-file-manager", None),
        "FreeSynergy.FileManager"
    );
}

#[test]
fn a_foreign_program_keeps_its_name_with_a_capital() {
    assert_eq!(display_name("kanidm", None), "Kanidm");
    assert_eq!(display_name("", None), "");
    assert_eq!(display_name("fs-", None), "fs-");
}
