//! `fs-store verify` names a command that starts another version than `installed.toml` says (exit not zero, with the
//! repair), and `fs-store relink` repairs it. One test, because the commands read the folders from the environment.

use std::fs;
use std::os::unix::fs::symlink;

fn run(args: &[&str]) -> Result<(), String> {
    fs_store_cli::run(&args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>())
}

#[test]
fn verify_is_red_for_a_command_that_starts_the_old_version_and_green_after_relink() {
    let tmp = tempfile::tempdir().unwrap();
    let (root, bin, apps) = (
        tmp.path().join("store"),
        tmp.path().join("bin"),
        tmp.path().join("apps"),
    );
    // the package was unpacked as 0.0.2, then became an rpm 0.0.7: the register says 0.0.7, the command still starts 0.0.2
    let old = root.join("packages/fsprog/0.0.2");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("fsprog-app"), "x").unwrap();
    fs::create_dir_all(root.join("current")).unwrap();
    symlink(&old, root.join("current/fsprog")).unwrap();
    fs::create_dir_all(&bin).unwrap();
    symlink(root.join("current/fsprog/fsprog-app"), bin.join("fsprog")).unwrap();
    fs::write(
        root.join("installed.toml"),
        "[[package]]\nname = \"fsprog\"\nversion = \"0.0.7\"\nsource = \"s\"\n",
    )
    .unwrap();
    // SAFETY: the only test of this file, so no other thread reads the environment meanwhile.
    unsafe {
        std::env::set_var("FS_STORE_ROOT", &root);
        std::env::set_var("FS_STORE_BIN", &bin);
        std::env::set_var("FS_STORE_APPS", &apps);
    }

    let err = run(&["verify"]).unwrap_err();
    assert!(err.contains("another version"), "{err}");
    assert!(run(&["verify", "fsprog"]).is_err());

    run(&["relink", "fsprog"]).unwrap();
    assert!(!root.join("packages/fsprog").exists());
    assert!(fs::symlink_metadata(bin.join("fsprog")).is_err());
    run(&["verify"]).unwrap();
    run(&["verify", "fsprog"]).unwrap();
    assert!(run(&["relink", "nothing"]).is_err());
}
