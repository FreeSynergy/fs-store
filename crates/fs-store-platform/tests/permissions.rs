//! Rights behave the same on every system: private files cannot be replaced, programs are told from other files.
mod common;

use std::fs;

use fs_store_platform::current;

#[test]
fn a_private_file_is_new_and_exclusive() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("private");
    let permissions = &current().permissions;
    permissions.create_private_dir(&dir).unwrap();
    let file = dir.join("a.bin");
    permissions.create_private_file(&file, b"bytes").unwrap();
    assert_eq!(fs::read(&file).unwrap(), b"bytes");
    assert!(permissions.create_private_file(&file, b"other").is_err());
    assert_eq!(fs::read(&file).unwrap(), b"bytes");
}

#[test]
fn a_private_file_is_never_written_through_a_link() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("target.bin");
    fs::write(&target, "original").unwrap();
    let link = tmp.path().join("link.bin");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    if std::os::windows::fs::symlink_file(&target, &link).is_err() {
        return; // making a symlink needs a right this runner may not give; the file part is covered by the test above
    }
    assert!(
        current()
            .permissions
            .create_private_file(&link, b"x")
            .is_err()
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "original");
}

#[test]
fn an_executable_is_a_program_file_not_a_folder_or_a_text() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::make_program(tmp.path(), "tasks");
    let permissions = &current().permissions;
    assert!(permissions.is_executable(&program));
    let text = tmp.path().join("notes.txt");
    fs::write(&text, "x").unwrap();
    assert!(!permissions.is_executable(&text));
    assert!(!permissions.is_executable(tmp.path()));
    assert!(!permissions.is_executable(&tmp.path().join("missing")));
}

#[test]
fn a_private_folder_is_for_this_person_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("private");
    current().permissions.create_private_dir(&dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    #[cfg(windows)]
    {
        let out = std::process::Command::new("icacls")
            .arg(&dir)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        for others in ["Everyone", "BUILTIN\\Users", "Authenticated Users"] {
            assert!(
                !text.contains(others),
                "{others} may use the folder: {text}"
            );
        }
    }
}

/// While the guard lives nobody can replace or delete the file (Windows); an elevated setup starts what was written.
#[cfg(windows)]
#[test]
fn a_written_file_cannot_be_swapped_while_the_guard_lives() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("setup.exe");
    let guard = current()
        .permissions
        .create_private_file(&file, b"one")
        .unwrap();
    assert!(fs::write(&file, b"two").is_err());
    assert!(fs::remove_file(&file).is_err());
    assert!(fs::rename(&file, tmp.path().join("other.exe")).is_err());
    assert_eq!(fs::read(&file).unwrap(), b"one");
    drop(guard);
    fs::remove_file(&file).unwrap();
}
