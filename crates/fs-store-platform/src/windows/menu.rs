//! Pattern: Strategy (Windows: a `.lnk` shortcut in the Start menu folder is the menu entry). The shortcut is made and read
//! through the shell's own COM object (`WScript.Shell`), which is the one way Windows documents for it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::shell::powershell;
use crate::{EntryState, MenuEntries, MenuEntry};

/// What the description of a shortcut of ours starts with.
const MARK: &str = "Made by FreeSynergy.Store";

/// Menu entries as shortcuts.
pub(super) struct Shortcuts;

fn file(dir: &Path, title: &str) -> PathBuf {
    dir.join(format!("{title}.lnk"))
}

impl MenuEntries for Shortcuts {
    fn entry_state(&self, dir: &Path, _id: &str, title: &str) -> EntryState {
        let path = file(dir, title);
        if !path.exists() {
            return EntryState::Missing;
        }
        let script =
            "(New-Object -ComObject WScript.Shell).CreateShortcut($env:FS_LNK).Description";
        let ours = powershell(script, &[("FS_LNK", &path.to_string_lossy())])
            .is_ok_and(|d| d.starts_with(MARK));
        if ours {
            EntryState::Ours
        } else {
            EntryState::Foreign
        }
    }

    fn write_entry(&self, dir: &Path, entry: &MenuEntry) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        let description = entry
            .comment
            .as_ref()
            .map_or_else(|| MARK.to_owned(), |c| format!("{MARK}: {c}"));
        let script = "$s = (New-Object -ComObject WScript.Shell).CreateShortcut($env:FS_LNK); \
            $s.TargetPath = $env:FS_TARGET; $s.Description = $env:FS_DESC; \
            $s.IconLocation = $env:FS_TARGET + ',0'; $s.Save()";
        let envs = [
            (
                "FS_LNK",
                file(dir, &entry.title).to_string_lossy().into_owned(),
            ),
            ("FS_TARGET", entry.program.to_string_lossy().into_owned()),
            ("FS_DESC", description),
        ];
        let envs: Vec<(&str, &str)> = envs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        powershell(script, &envs)
            .map(drop)
            .map_err(io::Error::other)
    }

    fn remove_entry(&self, dir: &Path, id: &str, title: &str) -> io::Result<()> {
        if self.entry_state(dir, id, title) == EntryState::Ours {
            fs::remove_file(file(dir, title))?;
        }
        Ok(())
    }
}
