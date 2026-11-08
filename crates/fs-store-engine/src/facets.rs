//! Pattern: none (two functions). Facets as commands (ADR 0063 addendum): a facet is a symbolic link to the program,
//! the program reads `argv[0]`; each facet and the program itself get a menu entry. Only links and files that
//! are ours are ever replaced or removed.

use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

/// The executable a facet link points to: `<package>` or `<package>-app` in the unpacked package, else the same
/// in `/usr/bin` (rpm installs).
fn main_program(root: &Path, package: &str) -> Option<PathBuf> {
    let dirs = [
        root.join("current").join(package),
        PathBuf::from("/usr/bin"),
    ];
    dirs.iter()
        .flat_map(|d| ["", "-app"].map(|suffix| d.join(format!("{package}{suffix}"))))
        .find(|p| p.is_file())
}

fn ours(root: &Path, package: &str, target: &Path) -> bool {
    target.starts_with(root.join("current").join(package))
        || (target.starts_with("/usr/bin")
            && target
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(package)))
}

fn plain(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && !name.starts_with('.')
}

fn display_name(command: &str) -> String {
    let word = command.strip_prefix("fs-").unwrap_or(command);
    let mut chars = word.chars();
    let first = chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default();
    format!("FreeSynergy.{first}{}", chars.as_str())
}

fn desktop(command: &str, exec: &str, icon: Option<&Path>) -> String {
    let icon = icon.map_or_else(String::new, |p| format!("Icon={}\n", p.display()));
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec={exec}\n{icon}Terminal=false\nCategories=Office;\nX-FreeSynergy-Managed=true\n",
        display_name(command)
    )
}

/// The icon of a command: its own `icons/<command>.png`, else the program's `icons/<package>.png`.
fn icon_for(root: &Path, command: &str, package: &str) -> Option<PathBuf> {
    [command, package]
        .iter()
        .map(|n| root.join("icons").join(format!("{n}.png")))
        .find(|p| p.is_file())
}

/// Makes the commands for `facets` (links to the program) and menu entries for them and for the program.
/// Returns the command names created. A name that exists and is not ours is left alone.
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
    let Some(target) = main_program(root, package) else {
        return Ok(Vec::new());
    };
    fs::create_dir_all(bin_dir)?;
    fs::create_dir_all(apps_dir)?;
    let mut made = Vec::new();
    let commands = std::iter::once(&package.to_owned())
        .chain(facets.iter())
        .cloned()
        .collect::<Vec<_>>();
    for facet in &commands {
        if !plain(facet) {
            continue;
        }
        let link = bin_dir.join(facet);
        match fs::read_link(&link) {
            Ok(old) if ours(root, package, &old) => fs::remove_file(&link)?,
            Ok(_) => continue,
            Err(_) if fs::symlink_metadata(&link).is_ok() => continue,
            Err(_) => {}
        }
        symlink(&target, &link)?;
        made.push(facet.clone());
    }
    for command in made
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(package))
    {
        let file = apps_dir.join(format!("net.freesynergy.{command}.desktop"));
        let ours = !file.exists()
            || fs::read_to_string(&file).is_ok_and(|t| t.contains("X-FreeSynergy-Managed=true"));
        if ours {
            fs::write(
                &file,
                desktop(
                    command,
                    command,
                    icon_for(root, command, package).as_deref(),
                ),
            )?;
        }
    }
    Ok(made)
}

/// Removes what [`link_facets`] made for `package` and `facets`: only our links and our menu entries.
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
    for facet in facets.iter().filter(|f| plain(f)) {
        let link = bin_dir.join(facet);
        if fs::read_link(&link).is_ok_and(|t| ours(root, package, &t)) {
            fs::remove_file(&link)?;
        }
    }
    for command in facets
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(package))
    {
        let file = apps_dir.join(format!("net.freesynergy.{command}.desktop"));
        if fs::read_to_string(&file).is_ok_and(|t| t.contains("X-FreeSynergy-Managed=true")) {
            fs::remove_file(&file)?;
        }
    }
    Ok(())
}

/// Where menu entries go: `FS_STORE_APPS`, else `~/.local/share/applications`.
#[must_use]
pub fn default_apps_dir() -> PathBuf {
    std::env::var_os("FS_STORE_APPS").map_or_else(
        || {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join(".local/share/applications")
        },
        PathBuf::from,
    )
}

/// Makes sure the store itself has a menu entry ("FreeSynergy.Store") for this person: written at the first start of the
/// window, never over a file that is not ours. Returns whether a file was written.
///
/// # Errors
/// `io::Error` when the directory or file cannot be written.
pub fn ensure_own_menu_entry(apps_dir: &Path) -> io::Result<bool> {
    let file = apps_dir.join("net.freesynergy.Store.desktop");
    if file.exists() {
        return Ok(false);
    }
    let exe = std::env::current_exe()?;
    fs::create_dir_all(apps_dir)?;
    fs::write(
        &file,
        format!(
            "[Desktop Entry]\nType=Application\nName=FreeSynergy.Store\nGenericName=Software store\nComment=Install, update and extend FreeSynergy programs\nExec={}\nTerminal=false\nCategories=System;PackageManager;\nX-FreeSynergy-Managed=true\n",
            exe.display()
        ),
    )?;
    Ok(true)
}
