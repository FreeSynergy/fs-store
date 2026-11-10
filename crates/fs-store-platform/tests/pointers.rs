//! The pointer to a folder behaves the same on every system: it points, it can be moved, and removing it never touches
//! the folder it pointed at.
use std::fs;

use fs_store_platform::current;

#[test]
fn a_pointer_points_moves_and_goes_away_without_touching_its_target() {
    let tmp = tempfile::tempdir().unwrap();
    let (one, two) = (tmp.path().join("v1"), tmp.path().join("v2"));
    fs::create_dir_all(&one).unwrap();
    fs::create_dir_all(&two).unwrap();
    fs::write(one.join("file"), "one").unwrap();
    let link = tmp.path().join("current").join("tasks");
    let pointers = &current().pointers;
    assert!(!pointers.pointer_exists(&link));
    assert_eq!(pointers.pointer_target(&link), None);
    pointers.point(&link, &one).unwrap();
    assert!(pointers.pointer_exists(&link));
    assert_eq!(fs::read_to_string(link.join("file")).unwrap(), "one");
    let target = pointers.pointer_target(&link).unwrap();
    assert!(current().paths.is_below(&target, &one) && current().paths.is_below(&one, &target));
    pointers.point(&link, &two).unwrap();
    let moved = pointers.pointer_target(&link).unwrap();
    assert!(current().paths.is_below(&moved, &two));
    assert!(!link.join("file").exists());
    pointers.remove_pointer(&link).unwrap();
    assert!(!pointers.pointer_exists(&link));
    assert_eq!(fs::read_to_string(one.join("file")).unwrap(), "one");
}

#[test]
fn a_folder_that_is_not_a_pointer_has_no_target() {
    let tmp = tempfile::tempdir().unwrap();
    let plain = tmp.path().join("plain");
    fs::create_dir(&plain).unwrap();
    assert_eq!(current().pointers.pointer_target(&plain), None);
}
