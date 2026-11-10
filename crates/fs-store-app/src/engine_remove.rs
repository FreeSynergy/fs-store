//! Pattern: Adapter (taking a program away, with the same steps as `fs-store uninstall` of the CLI; the CLI keeps
//! its code inside its own crate, so the steps are repeated here on purpose).

use std::path::Path;

use fs_store_engine::{ProcRunningCheck, SystemCommander};
use fs_store_sources::Registry;

/// rpm or deb: `pkexec dnf remove` / `pkexec apt-get remove`; else the unpacked package is deleted. Then the commands are unlinked and the
/// register forgets the program. The data of the person is never removed.
pub(crate) fn uninstall(root: &Path, name: &str) -> Result<(), String> {
    if fs_store_engine::native_installed_version(name).is_some() {
        fs_store_engine::remove_native(name, &mut SystemCommander)
            .map_err(|e| format!("{name}: {e}"))?;
    } else {
        let running = ProcRunningCheck::new(root);
        fs_store_engine::remove(root, name, &running).map_err(|e| format!("{name}: {e:?}"))?;
    }
    let bin = fs_store_engine::default_bin_dir();
    let _ = fs_store_engine::unlink_programs(root, name, &bin);
    let mut registry =
        Registry::load(&root.join("installed.toml")).map_err(|e| format!("{e:?}"))?;
    let facets = registry
        .get(name)
        .map(|e| e.facets.clone())
        .unwrap_or_default();
    let _ = fs_store_engine::unlink_facets(
        root,
        name,
        &facets,
        &bin,
        &fs_store_engine::default_apps_dir(),
    );
    registry.forget(name);
    registry.save().map_err(|e| format!("{e:?}"))
}
