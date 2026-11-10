//! A command on the person's path behaves the same on every system.
mod common;

use std::fs;

use fs_store_platform::current;

#[test]
fn a_command_runs_its_program_is_listed_and_goes_away() {
    let tmp = tempfile::tempdir().unwrap();
    let program = common::make_program(&tmp.path().join("pkg"), "tasks");
    let bin = tmp.path().join("bin");
    let commands = &current().commands;
    assert!(!commands.command_exists(&bin, "tasks"));
    commands.make_command(&bin, "tasks", &program).unwrap();
    assert!(commands.command_exists(&bin, "tasks"));
    assert!(commands.command_path(&bin, "tasks").exists());
    assert_eq!(
        commands.command_target(&bin, "tasks"),
        Some(program.clone())
    );
    assert_eq!(commands.commands(&bin), vec![("tasks".to_owned(), program)]);
    commands.remove_command(&bin, "tasks").unwrap();
    assert!(!commands.command_exists(&bin, "tasks"));
    assert_eq!(commands.commands(&bin).len(), 0);
}

#[test]
fn a_file_of_someone_else_is_not_a_command_of_ours() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let commands = &current().commands;
    fs::write(commands.command_path(&bin, "tasks"), "mine").unwrap();
    assert!(commands.command_exists(&bin, "tasks"));
    assert_eq!(commands.command_target(&bin, "tasks"), None);
    assert_eq!(commands.commands(&bin).len(), 0);
}

#[test]
fn a_command_name_is_the_program_file_name_without_the_suffix() {
    let paths = &current().paths;
    let file = paths.program_file_name("fs-tasks");
    assert_eq!(paths.command_name(&file), "fs-tasks");
    assert!(paths.command_files("fs-tasks").contains(&file));
}
