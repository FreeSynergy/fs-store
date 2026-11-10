//! Pattern: Strategy (Linux: a freedesktop `.desktop` file is the menu entry).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{EntryState, MenuEntries, MenuEntry};

const MARK: &str = "X-FreeSynergy-Managed=true";

/// Menu entries as `.desktop` files.
pub(super) struct DesktopEntries;

fn file(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.desktop"))
}

fn text(entry: &MenuEntry) -> String {
    let icon = entry
        .icon
        .as_ref()
        .map_or_else(String::new, |p| format!("Icon={}\n", p.display()));
    let comment = entry
        .comment
        .as_ref()
        .map_or_else(String::new, |c| format!("Comment={c}\n"));
    format!(
        "[Desktop Entry]\nType=Application\nName={}\n{comment}Exec={}\n{icon}Terminal=false\nCategories={}\n{MARK}\n",
        entry.title, entry.exec, entry.categories
    )
}

impl MenuEntries for DesktopEntries {
    fn entry_state(&self, dir: &Path, id: &str, _title: &str) -> EntryState {
        let path = file(dir, id);
        if !path.exists() {
            EntryState::Missing
        } else if fs::read_to_string(&path).is_ok_and(|t| t.contains(MARK)) {
            EntryState::Ours
        } else {
            EntryState::Foreign
        }
    }

    fn write_entry(&self, dir: &Path, entry: &MenuEntry) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(file(dir, &entry.id), text(entry))
    }

    fn remove_entry(&self, dir: &Path, id: &str, title: &str) -> io::Result<()> {
        if self.entry_state(dir, id, title) == EntryState::Ours {
            fs::remove_file(file(dir, id))?;
        }
        Ok(())
    }
}
