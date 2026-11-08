use std::fs;
use std::process::Command;

use fs_store_engine::net::{HttpsFetch, clone_catalog};
use fs_store_engine::service::Fetch;

#[test]
fn plain_http_is_refused() {
    assert!(
        HttpsFetch
            .get("http://example.org/x")
            .unwrap_err()
            .contains("https")
    );
}

#[test]
fn catalog_is_cloned_from_a_local_git_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("catalog.xml"), "<components/>").unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&src)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@t",
        "commit",
        "-qm",
        "c",
    ]);
    let dest = tmp.path().join("clone");
    clone_catalog(&format!("file://{}", src.display()), &dest).unwrap();
    assert_eq!(
        fs::read_to_string(dest.join("catalog.xml")).unwrap(),
        "<components/>"
    );
}
