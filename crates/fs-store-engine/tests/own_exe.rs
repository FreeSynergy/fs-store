use fs_store_engine::own_exe;

#[test]
fn the_running_executable_is_found_and_a_deleted_suffix_would_not_survive() {
    let exe = own_exe().unwrap();
    assert!(exe.is_absolute());
    assert!(!exe.to_string_lossy().ends_with(" (deleted)"));
    assert!(exe.exists());
}

#[test]
fn the_deleted_suffix_of_a_replaced_executable_is_dropped() {
    use std::path::Path;
    let drop = fs_store_engine::without_deleted_suffix;
    assert_eq!(
        drop(Path::new("/usr/bin/fs-store (deleted)")),
        Path::new("/usr/bin/fs-store")
    );
    assert_eq!(
        drop(Path::new("/usr/bin/fs-store")),
        Path::new("/usr/bin/fs-store")
    );
    assert_eq!(
        drop(Path::new("/home/u/my (deleted) tools/fs-store")),
        Path::new("/home/u/my (deleted) tools/fs-store")
    );
}
