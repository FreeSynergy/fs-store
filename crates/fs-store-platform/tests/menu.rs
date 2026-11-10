//! A menu entry behaves the same on every system: it is written as ours, read back as ours, and removed.
mod common;

use fs_store_platform::{EntryState, MenuEntry, current};

fn entry(program: &std::path::Path) -> MenuEntry {
    MenuEntry {
        id: "net.freesynergy.tasks".to_owned(),
        title: "FreeSynergy.Tasks".to_owned(),
        comment: Some("Tasks".to_owned()),
        exec: "tasks".to_owned(),
        program: program.to_owned(),
        program_icon: program.to_owned(),
        icon: None,
        categories: "Office;".to_owned(),
    }
}

#[test]
fn an_entry_is_ours_after_writing_and_gone_after_removing() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::make_program(tmp.path(), "tasks");
    let dir = tmp.path().join("menu");
    let menu = &current().menu;
    let e = entry(&program);
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Missing);
    menu.write_entry(&dir, &e).unwrap();
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Ours);
    menu.write_entry(&dir, &e).unwrap(); // ours may be written again
    menu.remove_entry(&dir, &e.id, &e.title).unwrap();
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Missing);
}

#[cfg(unix)]
#[test]
fn an_entry_of_someone_else_is_never_taken_over() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::make_program(tmp.path(), "tasks");
    let dir = tmp.path().join("menu");
    std::fs::create_dir_all(&dir).unwrap();
    let e = entry(&program);
    std::fs::write(
        dir.join(format!("{}.desktop", e.id)),
        "[Desktop Entry]\nName=mine\n",
    )
    .unwrap();
    let menu = &current().menu;
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Foreign);
    menu.remove_entry(&dir, &e.id, &e.title).unwrap();
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Foreign);
}

/// A shortcut of the person is never taken over: its description does not carry our mark.
#[cfg(windows)]
#[test]
fn a_shortcut_of_someone_else_is_never_taken_over() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::make_program(tmp.path(), "tasks");
    let dir = tmp.path().join("menu");
    std::fs::create_dir_all(&dir).unwrap();
    let e = entry(&program);
    let script = "$s = (New-Object -ComObject WScript.Shell).CreateShortcut($env:FS_LNK); $s.TargetPath = $env:FS_TARGET; \
                  $s.Description = 'mine'; $s.Save()";
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("FS_LNK", dir.join(format!("{}.lnk", e.title)))
        .env("FS_TARGET", &program)
        .status()
        .unwrap();
    assert!(status.success());
    let menu = &current().menu;
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Foreign);
    menu.remove_entry(&dir, &e.id, &e.title).unwrap();
    assert_eq!(menu.entry_state(&dir, &e.id, &e.title), EntryState::Foreign);
}
