//! Pattern: Facade over `Commands` of the platform (what a link is differs per system). Puts the programs of a package on
//! the person's path, and takes them away again. A command runs through `current/<name>/`, so an update changes what it
//! runs without touching the command.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_store_platform::current;

/// Where the commands of installed programs are made: `FS_STORE_BIN`, else the platform's place.
#[must_use]
pub fn default_bin_dir() -> PathBuf {
    current().paths.bin_dir()
}

/// Makes a command for every executable file at the top of `current/<name>/` in `bin_dir`. Returns the names made.
/// A name that exists and is not one of our commands is left alone and reported in `skipped`.
///
/// # Errors
/// `io::Error` when the directory cannot be read or a command cannot be made.
pub fn link_programs(
    root: &Path,
    name: &str,
    bin_dir: &Path,
) -> io::Result<(Vec<String>, Vec<String>)> {
    let platform = current();
    let pkg = root.join("current").join(name);
    let (mut linked, mut skipped) = (Vec::new(), Vec::new());
    for entry in fs::read_dir(&pkg)? {
        let entry = entry?;
        let file = entry.file_name().to_string_lossy().into_owned();
        let command = platform.paths.command_name(&file);
        // `<name>-app` and `<name>-server` are reached through the one command `<name>` (see `link_facets`)
        let plain = command == format!("{name}-app") || command == format!("{name}-server");
        if !platform.permissions.is_executable(&entry.path()) || file.starts_with('.') || plain {
            continue;
        }
        match platform.commands.command_target(bin_dir, &command) {
            Some(old) if platform.paths.is_below(&old, &root.join("current")) => {
                platform.commands.remove_command(bin_dir, &command)?;
            }
            Some(_) => {
                skipped.push(command);
                continue;
            }
            None if platform.commands.command_exists(bin_dir, &command) => {
                skipped.push(command);
                continue;
            }
            None => {}
        }
        platform
            .commands
            .make_command(bin_dir, &command, &pkg.join(&file))?;
        linked.push(command);
    }
    Ok((linked, skipped))
}

/// Removes the commands in `bin_dir` that run a program in `current/<name>/`. Returns how many were removed.
///
/// # Errors
/// `io::Error` when a command cannot be removed.
pub fn unlink_programs(root: &Path, name: &str, bin_dir: &Path) -> io::Result<usize> {
    let platform = current();
    let pkg = root.join("current").join(name);
    let mut n = 0;
    for (command, target) in platform.commands.commands(bin_dir) {
        if platform.paths.is_below(&target, &pkg) {
            platform.commands.remove_command(bin_dir, &command)?;
            n += 1;
        }
    }
    Ok(n)
}
