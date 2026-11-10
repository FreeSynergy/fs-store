//! Pattern: Command (the automatic part: update everything, per-package switch, the systemd user timer).

use std::path::Path;

use fs_store_sources::Registry;

use crate::batch::{Want, install_batch};
use crate::commands::{fetch_source, read_catalog, root, trust};

fn registry(root: &Path) -> Result<Registry, String> {
    Registry::load(&root.join("installed.toml")).map_err(|e| format!("{e:?}"))
}

/// The source a package was installed from, if the register knows it.
pub(crate) fn registered_source(root: &Path, name: &str) -> Option<String> {
    registry(root).ok()?.get(name).map(|e| e.source.clone())
}

/// Updates every installed package that has automatic updates on, each from its own source, as ONE batch:
/// downloads at the same time, one install step (one password for all rpm). A failing package does not stop the others.
pub(crate) fn update_all(root: &Path, extra: Option<&String>) -> Result<(), String> {
    let mut wants = Vec::new();
    for e in registry(root)?.list() {
        if e.auto_update {
            wants.push(Want {
                name: e.name.clone(),
                source: Some(e.source.clone()),
            });
        } else {
            println!("{}: automatic updates are off", e.name);
        }
    }
    install_batch(root, &wants, extra, true)
}

/// `fs-store uninstall <name>` (also `remove`): takes the package away and forgets it in the register.
pub(crate) fn remove(root: &Path, name: &str) -> Result<(), String> {
    // Installed by rpm or deb: dnf or apt takes it away (pkexec asks for the password); else the unpacked package is deleted.
    // The person's data is never removed.
    if fs_store_engine::native_installed_version(name).is_some() {
        fs_store_engine::remove_native(name, &mut fs_store_engine::SystemCommander)
            .map_err(|e| format!("{name}: {e}"))?;
    } else {
        let running = fs_store_engine::ProcRunningCheck::new(root);
        fs_store_engine::remove(root, name, &running).map_err(|e| format!("{name}: {e:?}"))?;
    }
    let _ = fs_store_engine::unlink_programs(root, name, &bin_dir());
    let mut registry = registry(root)?;
    let facets = registry
        .get(name)
        .map(|e| e.facets.clone())
        .unwrap_or_default();
    let _ = fs_store_engine::unlink_facets(
        root,
        name,
        &facets,
        &bin_dir(),
        &fs_store_engine::default_apps_dir(),
    );
    registry.forget(name);
    registry.save().map_err(|e| format!("{e:?}"))?;
    println!("removed {name}");
    Ok(())
}

fn bin_dir() -> std::path::PathBuf {
    fs_store_engine::default_bin_dir()
}

/// Puts the programs of `name` on the PATH; says what it did, never fails the install.
pub(crate) fn link(root: &Path, name: &str) {
    if !root.join("current").join(name).exists() {
        return; // installed by rpm: dnf put the programs in /usr/bin
    }
    match fs_store_engine::link_programs(root, name, &bin_dir()) {
        Ok((linked, skipped)) => {
            if !linked.is_empty() {
                println!("commands in {}: {}", bin_dir().display(), linked.join(", "));
            }
            for s in skipped {
                eprintln!("{s}: another file of that name is in the way, not linked");
            }
        }
        Err(e) => eprintln!("{name}: commands not linked: {e}"),
    }
}

/// `fs-store check`: says which installed programs have a newer version, installs nothing.
/// The catalog signature is verified before anything is read from it.
pub(crate) fn check(root: &Path, extra: Option<&String>) -> Result<(), String> {
    let (verifier, policy) = trust(extra)?;
    let mut newer = 0;
    for e in registry(root)?.list() {
        let Some(have) = fs_store_core::Version::parse(&e.version) else {
            continue;
        };
        let offered = fetch_source(root, &e.source, &verifier, &policy)
            .and_then(|(c, _)| read_catalog(&c))
            .map(|cat| cat.entries.into_iter().find(|o| o.name.as_str() == e.name));
        match offered {
            Ok(Some(o)) if o.version > have => {
                println!("{} {} -> {}", e.name, have, o.version);
                newer += 1;
            }
            Ok(_) => {}
            Err(message) => eprintln!("{}: {message}", e.name),
        }
    }
    if newer == 0 {
        println!("everything is up to date");
    }
    Ok(())
}

/// Makes the facet commands and menu entries of a program; returns the names made (for the register).
pub(crate) fn link_facets(root: &Path, name: &str, facets: &[String]) -> Vec<String> {
    match fs_store_engine::link_facets(
        root,
        name,
        facets,
        &bin_dir(),
        &fs_store_engine::default_apps_dir(),
    ) {
        Ok(made) => {
            if !made.is_empty() {
                println!("facet commands: {}", made.join(", "));
            }
            made
        }
        Err(e) => {
            eprintln!("{name}: facet commands not made: {e}");
            Vec::new()
        }
    }
}

/// `fs-store auto <name> on|off`.
pub(crate) fn auto(root: &Path, name: &str, mode: &str) -> Result<(), String> {
    let on = match mode {
        "on" => true,
        "off" => false,
        _ => return Err("usage: fs-store auto <name> on|off".into()),
    };
    let mut r = registry(root)?;
    if !r.set_auto_update(name, on) {
        return Err(format!("{name} is not installed"));
    }
    r.save().map_err(|e| format!("{e:?}"))?;
    println!("{name}: automatic updates {mode}");
    Ok(())
}

/// `fs-store timer enable|disable|status`: the timer of this system that updates by itself.
pub(crate) fn timer(rest: &[String]) -> Result<(), String> {
    let background = &fs_store_platform::current().background;
    let said = match rest.first().map(String::as_str) {
        Some("enable") => {
            let exe = fs_store_engine::own_exe().map_err(|e| e.to_string())?;
            let every = match fs_store_sources::Settings::load(&root().join("settings.toml")).map(|s| s.check) {
                Ok(fs_store_sources::CheckEvery::Hourly) => fs_store_platform::Every::Hourly,
                Ok(fs_store_sources::CheckEvery::Daily) => fs_store_platform::Every::Daily,
                _ => return Err("settings say: check only at start or never; the timer is not needed (fs-store settings check daily|hourly)".into()),
            };
            background.timer_enable(&exe, every)?
        }
        Some("disable") => background.timer_disable()?,
        Some("status") => background.timer_status()?,
        _ => return Err("usage: fs-store timer enable|disable|status".into()),
    };
    print!("{said}");
    Ok(())
}
