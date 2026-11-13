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
        let facets: Vec<String> = e.facets.iter().filter(|f| **f != e.name).cloned().collect();
        let settled = fs_store_engine::settle_install(
            root,
            &e.name,
            &e.version,
            &facets,
            &fs_store_engine::default_bin_dir(),
            &fs_store_engine::default_apps_dir(),
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
