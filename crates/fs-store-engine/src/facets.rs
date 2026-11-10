//! Pattern: Facade over `Commands` and `MenuEntries` of the platform. Facets as commands (ADR 0063 addendum): a facet is a
//! command that runs the program under the facet's name (a link on Linux, a launcher file on Windows); each facet and
//! the program itself get a menu entry. Only commands and entries that are ours are ever replaced or removed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_store_platform::{EntryState, MenuEntry, Platform, current, display_name};

/// The executable a facet command runs: `<package>` or `<package>-app` in the unpacked package, else the same where the
/// system's own package put it (`/usr/bin`, the program folder of the setup).
fn main_program(platform: &Platform, root: &Path, package: &str) -> Option<PathBuf> {
    let mut dirs = vec![root.join("current").join(package)];
    dirs.extend(platform.paths.native_program_dirs(package));
    dirs.iter()
        .flat_map(|d| {
            ["", "-app"].map(|suffix| {
                d.join(
                    platform
                        .paths
                        .program_file_name(&format!("{package}{suffix}")),
                )
            })
        })
        .find(|p| p.is_file())
}

fn ours(platform: &Platform, root: &Path, package: &str, target: &Path) -> bool {
    let paths = &platform.paths;
    paths.is_below(target, &root.join("current").join(package))
        || (paths
            .native_program_dirs(package)
            .iter()
            .any(|d| paths.is_below(target, d))
            && target
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(package)))
}

fn plain(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && !name.starts_with('.')
}

fn entry_id(command: &str) -> String {
    format!("net.freesynergy.{command}")
}

/// The icon of a command: its own `icons/<command>.png`, else the program's `icons/<package>.png`.
fn icon_for(root: &Path, command: &str, package: &str) -> Option<PathBuf> {
    [command, package]
        .iter()
        .map(|n| root.join("icons").join(format!("{n}.png")))
        .find(|p| p.is_file())
}

/// Makes the commands for `facets` and menu entries for them and for the program. Returns the command names created. A
/// name that exists and is not ours is left alone.
///
/// # Errors
/// `io::Error` when a directory or file cannot be written.
pub fn link_facets(
    root: &Path,
    package: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
) -> io::Result<Vec<String>> {
    let platform = current();
    let Some(target) = main_program(platform, root, package) else {
        return Ok(Vec::new());
    };
    fs::create_dir_all(bin_dir)?;
    let mut made = Vec::new();
    let commands = std::iter::once(package.to_owned()).chain(facets.iter().cloned());
    for facet in commands.filter(|f| plain(f)) {
        match platform.commands.command_target(bin_dir, &facet) {
            Some(old) if ours(platform, root, package, &old) => {
                platform.commands.remove_command(bin_dir, &facet)?;
            }
            Some(_) => continue,
            None if platform.commands.command_exists(bin_dir, &facet) => continue,
            None => {}
        }
        platform.commands.make_command(bin_dir, &facet, &target)?;
        made.push(facet);
    }
    for command in made.iter().map(String::as_str).chain([package]) {
        let (id, title) = (entry_id(command), display_name(command));
        if platform.menu.entry_state(apps_dir, &id, &title) != EntryState::Foreign {
            platform.menu.write_entry(
                apps_dir,
                &MenuEntry {
                    id,
                    title,
                    comment: None,
                    exec: command.to_owned(),
                    program: target.clone(),
                    icon: icon_for(root, command, package),
                    categories: "Office;".to_owned(),
                },
            )?;
        }
    }
    Ok(made)
}

/// Removes what [`link_facets`] made for `package` and `facets`: only our commands and our menu entries.
///
/// # Errors
/// `io::Error` when a file cannot be removed.
pub fn unlink_facets(
    root: &Path,
    package: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
) -> io::Result<()> {
    let platform = current();
    for facet in facets.iter().filter(|f| plain(f)) {
        let mine = platform
            .commands
            .command_target(bin_dir, facet)
            .is_some_and(|t| ours(platform, root, package, &t));
        if mine {
            platform.commands.remove_command(bin_dir, facet)?;
        }
    }
    for command in facets.iter().map(String::as_str).chain([package]) {
        platform
            .menu
            .remove_entry(apps_dir, &entry_id(command), &display_name(command))?;
    }
    Ok(())
}

/// Where menu entries go: `FS_STORE_APPS`, else the platform's place.
#[must_use]
pub fn default_apps_dir() -> PathBuf {
    current().paths.menu_dir()
}

/// Makes sure the store itself has a menu entry ("FreeSynergy.Store") for this person: written at the first start of the
/// window, never over an entry that is not ours. Returns whether an entry was written.
///
/// # Errors
/// `io::Error` when the directory or file cannot be written.
pub fn ensure_own_menu_entry(apps_dir: &Path) -> io::Result<bool> {
    let platform = current();
    let (id, title) = ("net.freesynergy.Store", "FreeSynergy.Store");
    if platform.menu.entry_state(apps_dir, id, title) != EntryState::Missing {
        return Ok(false);
    }
    let exe = crate::own_exe()?;
    platform.menu.write_entry(
        apps_dir,
        &MenuEntry {
            id: id.to_owned(),
            title: title.to_owned(),
            comment: Some("Install, update and extend FreeSynergy programs".to_owned()),
            exec: exe.display().to_string(),
            program: exe,
            icon: None,
            categories: "System;PackageManager;".to_owned(),
        },
    )?;
    Ok(true)
}
