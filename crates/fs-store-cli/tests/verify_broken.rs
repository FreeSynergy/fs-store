//! `fs-store verify <package>` does not call a broken list of files "intact": only "no list" is let through (an rpm).
//! One test, because the commands read the store folder from the environment.

use std::fs;

fn run(args: &[&str]) -> Result<(), String> {
    fs_store_cli::run(&args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>())
}

#[test]
fn a_damaged_list_of_files_is_an_error_not_all_intact() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let dir = root.join("packages/prog/1.0.0");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("prog"), "original").unwrap();
    fs::create_dir_all(root.join("current")).unwrap();
    std::os::unix::fs::symlink(&dir, root.join("current/prog")).unwrap();
    fs::write(
        root.join("installed.toml"),
        "[[package]]\nname = \"prog\"\nversion = \"1.0.0\"\nsource = \"s\"\n",
    )
    .unwrap();
    // SAFETY: the only test of this file, so no other thread reads the environment meanwhile.
    unsafe {
        std::env::set_var("FS_STORE_ROOT", &root);
        std::env::set_var("FS_STORE_BIN", tmp.path().join("bin"));
        std::env::set_var("FS_STORE_SYSTEM", tmp.path().join("sys"));
    }
    run(&["verify", "prog"]).unwrap();

    fs::write(root.join("manifests/prog.toml"), "this is { not a list").unwrap();
    assert!(run(&["verify", "prog"]).is_err());
    assert!(run(&["verify"]).is_err());
}
