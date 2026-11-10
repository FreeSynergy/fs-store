//! Pattern: Strategy (where things live on this system). One implementation per platform; the environment is
//! handed in as a function so a test needs no real environment.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Reads an environment variable.
pub type Vars = Box<dyn Fn(&str) -> Option<OsString> + Send + Sync>;

/// The variables of this process.
pub fn process_vars() -> Vars {
    #[allow(clippy::redundant_closure)] // the closure fixes the lifetime of the name
    Box::new(|name| std::env::var_os(name))
}

/// Where the store keeps its files and where programs are announced. `FS_STORE_ROOT`, `FS_STORE_BIN` and `FS_STORE_APPS`
/// override the first three on every system.
pub trait Paths: Send + Sync {
    /// The store's folder: unpacked packages, register, settings.
    fn store_root(&self) -> PathBuf;
    /// Where the commands of installed programs are made.
    fn bin_dir(&self) -> PathBuf;
    /// Where menu entries go.
    fn menu_dir(&self) -> PathBuf;
    /// The person's configuration folder (for the timer of the background check).
    fn config_dir(&self) -> PathBuf;
    /// Where the system package manager puts the program of `package`: `/usr/bin`, or the program folder of the setup.
    fn native_program_dirs(&self, package: &str) -> Vec<PathBuf>;
    /// The folder only this package lives in when the system gives each one its own (the program folder of a Windows
    /// setup); `None` where packages share folders (`/usr/bin`).
    fn own_program_dir(&self, package: &str) -> Option<PathBuf>;
    /// The file name of the program `command` (`fs-tasks`, or `fs-tasks.exe`).
    fn program_file_name(&self, command: &str) -> String;
    /// The file names a command may have on the person's path, in the order the system tries them.
    fn command_files(&self, command: &str) -> Vec<String>;
    /// The command a program file is called by: the file name without the system's executable suffix.
    fn command_name(&self, file: &str) -> String;
    /// True when `path` is `dir` or inside it, the way this system compares paths (Windows ignores case).
    fn is_below(&self, path: &Path, dir: &Path) -> bool;
}

/// What a person reads for the program `command`: `fs-tasks` becomes `FreeSynergy.Tasks`.
#[must_use]
pub fn display_name(command: &str) -> String {
    let word = command.strip_prefix("fs-").unwrap_or(command);
    let mut chars = word.chars();
    let first = chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default();
    format!("FreeSynergy.{first}{}", chars.as_str())
}
