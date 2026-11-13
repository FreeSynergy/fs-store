//! Pattern: Command (`fs-store relink [name]`). Makes the commands and menu entries of installed programs again for the
//! form that is active now (the repair `fs-store verify` names when a command starts another version).

use std::path::Path;

/// Redoes the follow-up of the install of `only` (or of every installed program) from the register.
pub(crate) fn relink(root: &Path, only: Option<&str>) -> Result<(), String> {
    let registry = fs_store_sources::Registry::load(&root.join("installed.toml"))
        .map_err(|e| format!("the list of installed programs cannot be read: {e:?}"))?;
    let todo: Vec<_> = registry
        .list()
        .iter()
        .filter(|e| only.is_none_or(|o| o == e.name))
        .collect();
    if todo.is_empty() {
        return Err(only.map_or_else(
            || "nothing is installed".to_owned(),
            |o| format!("{o} is not installed by this store"),
        ));
    }
    let mut problems = Vec::new();
    for e in todo {
        let system = fs_store_engine::default_system_dir();
        // the register says a form that is not there: do not take the only installation away for it
        let kept = root.join("packages").join(&e.name);
        let has_unpacked = std::fs::read_dir(&kept).is_ok_and(|mut d| d.next().is_some());
        let wanted_unpacked = kept.join(&e.version).is_dir();
        let has_system = [e.name.clone(), format!("{}-app", e.name)]
            .iter()
            .any(|p| system.join(p).exists());
        if has_unpacked && !wanted_unpacked && !has_system {
            problems.push(format!(
                "{0}: the register says {1}, but that version is neither unpacked by the store nor found in {2}; nothing was changed (install {0} {1} again, or correct installed.toml)",
                e.name,
                e.version,
                system.display()
            ));
            continue;
        }
        let facets: Vec<String> = e.facets.iter().filter(|f| **f != e.name).cloned().collect();
        let settled = fs_store_engine::settle_install_with(
            root,
            &e.name,
            &e.version,
            &facets,
            &fs_store_engine::default_bin_dir(),
            &fs_store_engine::default_apps_dir(),
            &system,
        );
        println!("relinked {} {}", e.name, e.version);
        problems.extend(settled.problems);
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}
