//! `fs-store verify` and `fs-store owner` through the real command entry: exit codes and the migration of an old install.
//! One test, because the commands read the store folder from the environment.

use std::fs;

fn run(args: &[&str]) -> Result<(), String> {
    fs_store_cli::run(&args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>())
}

#[test]
fn verify_fails_on_a_changed_file_and_owner_names_the_package() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let dir = root.join("packages/prog/1.0.0/bin");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("prog"), "original").unwrap();
    fs::create_dir_all(root.join("current")).unwrap();
    std::os::unix::fs::symlink(root.join("packages/prog/1.0.0"), root.join("current/prog"))
        .unwrap();
    // SAFETY: the only test of this file, so no other thread reads the environment meanwhile.
    unsafe { std::env::set_var("FS_STORE_ROOT", &root) };

    // an old install without a list: the first verify makes one and is clean
    run(&["verify"]).unwrap();
    assert!(root.join("manifests/prog.toml").is_file());
    run(&[
        "owner",
        root.join("current/prog/bin/prog").to_str().unwrap(),
    ])
    .unwrap();
    run(&["owner", tmp.path().join("elsewhere").to_str().unwrap()]).unwrap();

    fs::write(dir.join("prog"), "tampered").unwrap();
    let err = run(&["verify"]).unwrap_err();
    assert!(err.contains("1 files changed or missing"), "{err}");
    assert!(run(&["verify", "prog"]).is_err());
    assert!(
        run(&[
            "owner",
            root.join("current/prog/bin/prog").to_str().unwrap()
        ])
        .is_err()
    );

    fs::remove_file(dir.join("prog")).unwrap();
    assert!(run(&["verify"]).unwrap_err().contains("missing"));
    assert!(run(&["verify", "nothing"]).is_err());
}
