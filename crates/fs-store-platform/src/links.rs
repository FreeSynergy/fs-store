//! Pattern: Strategy (what stands for a link: a symlink on Linux, a junction, a launcher file and a shortcut on Windows).
//! Three small traits: a pointer to a folder, a command on the person's path, an entry in the menu.

use std::io;
use std::path::{Path, PathBuf};

/// A pointer to a folder: `current/<name>` points at the active version, so an update only moves the pointer.
pub trait Pointers: Send + Sync {
    /// Makes `link` point at the folder `target`; an older pointer at `link` is replaced in one step where the system
    /// allows it.
    ///
    /// # Errors
    /// `io::Error` when the pointer cannot be made.
    fn point(&self, link: &Path, target: &Path) -> io::Result<()>;
    /// Where the pointer `link` points; `None` when there is no pointer there.
    fn pointer_target(&self, link: &Path) -> Option<PathBuf>;
    /// True when anything (a pointer or not) is at `link`.
    fn pointer_exists(&self, link: &Path) -> bool;
    /// Removes the pointer, never what it points at.
    ///
    /// # Errors
    /// `io::Error` when it cannot be removed.
    fn remove_pointer(&self, link: &Path) -> io::Result<()>;
}

/// A command of an installed program on the person's path.
pub trait Commands: Send + Sync {
    /// Makes the command `name` in `bin_dir` run the program file `target`.
    ///
    /// # Errors
    /// `io::Error` when it cannot be made.
    fn make_command(&self, bin_dir: &Path, name: &str, target: &Path) -> io::Result<()>;
    /// The file that stands for the command `name` in `bin_dir`.
    fn command_path(&self, bin_dir: &Path, name: &str) -> PathBuf;
    /// The program the command `name` runs, when it is one made by [`Commands::make_command`].
    fn command_target(&self, bin_dir: &Path, name: &str) -> Option<PathBuf>;
    /// True when anything is at the place of the command, ours or not.
    fn command_exists(&self, bin_dir: &Path, name: &str) -> bool;
    /// Removes the command, never the program.
    ///
    /// # Errors
    /// `io::Error` when it cannot be removed.
    fn remove_command(&self, bin_dir: &Path, name: &str) -> io::Result<()>;
    /// Every command of ours in `bin_dir`, with the program it runs.
    fn commands(&self, bin_dir: &Path) -> Vec<(String, PathBuf)>;
}

/// One entry of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    /// Stable id (`net.freesynergy.fs-tasks`): the file name on Linux.
    pub id: String,
    /// What the person reads in the menu.
    pub title: String,
    /// A sentence for the tooltip, if any.
    pub comment: Option<String>,
    /// What Linux writes as `Exec` (a command on the path, or a file).
    pub exec: String,
    /// The file Windows starts (a program, or the launcher of a facet).
    pub program: PathBuf,
    /// The executable whose icon Windows shows (`program` itself for a program).
    pub program_icon: PathBuf,
    /// Icon file (`png`); Windows uses the icon of the program instead.
    pub icon: Option<PathBuf>,
    /// Freedesktop categories (`Office;`); Windows has none.
    pub categories: String,
}

/// Who made a menu entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryState {
    /// Nothing there.
    Missing,
    /// Made by the store: may be replaced and removed.
    Ours,
    /// Something of the person or another program: never touched.
    Foreign,
}

/// Entries in the person's menu.
pub trait MenuEntries: Send + Sync {
    /// Whose entry `id` (shown as `title`) in `dir` is.
    fn entry_state(&self, dir: &Path, id: &str, title: &str) -> EntryState;
    /// Writes the entry, marked as ours.
    ///
    /// # Errors
    /// `io::Error` when it cannot be written.
    fn write_entry(&self, dir: &Path, entry: &MenuEntry) -> io::Result<()>;
    /// Removes the entry when it is ours.
    ///
    /// # Errors
    /// `io::Error` when it cannot be removed.
    fn remove_entry(&self, dir: &Path, id: &str, title: &str) -> io::Result<()>;
}
