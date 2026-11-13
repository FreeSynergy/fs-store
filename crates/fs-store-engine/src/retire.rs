//! Pattern: none (two functions). The package changed its form (`tar.gz` to `rpm`/`deb`): what the store unpacked for it
//! is taken away, and the commands are checked against the register (which version do they really start?).

use std::fs;
use std::path::Path;

use fs_store_sources::{Installed, ManifestStore};

use crate::own_files::package_dir;
use crate::own_judge::stray;

/// Takes away what the store itself made for the unpacked form of `name`: links into `current/<name>`, the `current`
/// link, the version folders below `packages/<name>/` and the list of files. A folder that holds a file the store did
/// not put there stays and is reported. Returns the problems, one sentence each.
pub(crate) fn retire_unpacked(root: &Path, name: &str, bin_dir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let mut say = |what: &str, e: &dyn std::fmt::Display| {
        problems.push(format!("{name}: old unpacked {what} not removed: {e}"));
    };
    if let Err(e) = crate::unlink_programs(root, name, bin_dir) {
        say("commands", &e);
    }
    let link = root.join("current").join(name);
    if fs::symlink_metadata(&link).is_ok_and(|m| m.file_type().is_symlink())
        && let Err(e) = fs::remove_file(&link)
    {
        say("link", &e);
    }
    let store = ManifestStore::new(root);
    let listed = store.load(name).ok().flatten();
    let Ok(versions) = fs::read_dir(root.join("packages").join(name)) else {
        return problems;
    };
    let mut kept = false;
    for entry in versions.flatten() {
        let dir = entry.path();
        let version = entry.file_name().to_string_lossy().into_owned();
        let own_list = listed.as_ref().filter(|m| m.version == version);
        if own_list.is_some_and(|m| stray(&dir, &m.files).is_ok_and(|s| !s.is_empty())) {
            kept = true;
            say(
                "files",
                &format!("{} holds files that are not ours", dir.display()),
            );
        } else if let Err(e) = fs::remove_dir_all(&dir) {
            say("files", &e);
        }
    }
    if !kept {
        let _ = fs::remove_dir(root.join("packages").join(name));
        if let Err(e) = store.forget(name) {
            say("list of files", &format!("{e:?}"));
        }
    }
    problems
}

/// The commands of installed programs against the register: a command of ours that starts another version than the
/// register says (a link into `current/<name>` after the package became an rpm, or into the package manager's folder
/// while the unpacked form is the active one) or a file that is gone. Returns one sentence per command, with the repair.
#[must_use]
pub fn check_links(
    root: &Path,
    bin_dir: &Path,
    installed: &[Installed],
    system: &Path,
) -> Vec<String> {
    let mut out = Vec::new();
    for e in installed {
        let unpacked = package_dir(root, &e.name, &e.version).is_dir();
        let own_dir = root.join("current").join(&e.name);
        for command in std::iter::once(&e.name).chain(e.facets.iter()) {
            let Ok(target) = fs::read_link(bin_dir.join(command)) else {
                continue;
            };
            let runs = if target.starts_with(&own_dir) {
                let now = fs::read_link(&own_dir)
                    .ok()
                    .and_then(|t| t.file_name().map(|n| n.to_string_lossy().into_owned()));
                match now {
                    Some(v) if unpacked && v == e.version => broken(&target),
                    Some(v) => Some(format!("version {v} ({})", target.display())),
                    None => Some(format!("{}", target.display())),
                }
            } else if target.starts_with(system) && unpacked {
                Some(format!("the system program {}", target.display()))
            } else {
                broken(&target)
            };
            if let Some(runs) = runs {
                out.push(format!(
                    "{}: the command `{command}` starts {runs}, but the register says {} {}. Repair: fs-store relink {}",
                    e.name, e.name, e.version, e.name
                ));
            }
        }
    }
    out
}

/// `None` when the link is fine (the file is there); else what is wrong with it.
fn broken(target: &Path) -> Option<String> {
    (!target.exists()).then(|| format!("{}, which is gone", target.display()))
}
