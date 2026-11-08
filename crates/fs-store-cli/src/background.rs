//! Pattern: Command (`fs-store background`, the check that the timer and the window start) and settings commands.
//! Finds updates and new programs, writes `updates.toml`, tells the person (notification, launcher badge) and, if the
//! person chose so, installs the updates. Every way of telling is best effort and can be switched off.

use std::path::Path;
use std::process::Command;

use fs_store_sources::{CheckEvery, Findings, InstallMode, Novelty, Registry, Settings, Update};

use crate::commands::{addresses, fetch_source, read_catalog, trust};

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Looks at every enabled source; returns the updates of installed programs and the names on offer.
fn scan(root: &Path, extra: Option<&String>) -> Result<(Vec<Update>, Vec<Novelty>), String> {
    let (verifier, policy) = trust(extra)?;
    let registry = Registry::load(&root.join("installed.toml")).map_err(|e| format!("{e:?}"))?;
    let (mut updates, mut offered) = (Vec::new(), Vec::new());
    let mut reached = false;
    for url in addresses(root, None)? {
        let catalog = match fetch_source(root, &url, &verifier, &policy)
            .and_then(|(c, _)| read_catalog(&c))
        {
            Ok(c) => c,
            Err(message) => {
                eprintln!("{message}");
                continue;
            }
        };
        reached = true;
        for entry in catalog.entries {
            offered.push(Novelty {
                name: entry.name.as_str().to_owned(),
                version: entry.version.to_string(),
            });
            let have = registry
                .get(entry.name.as_str())
                .and_then(|i| fs_store_core::Version::parse(&i.version));
            if let Some(have) = have.filter(|h| entry.version > *h) {
                updates.push(Update {
                    name: entry.name.as_str().to_owned(),
                    from: have.to_string(),
                    to: entry.version.to_string(),
                });
            }
        }
    }
    if reached {
        Ok((updates, offered))
    } else {
        Err("no source could be read".into())
    }
}

/// `fs-store background`: check, remember, tell, maybe install.
pub(crate) fn background(root: &Path, extra: Option<&String>) -> Result<(), String> {
    let settings = Settings::load(&root.join("settings.toml")).map_err(|e| format!("{e:?}"))?;
    if settings.check == CheckEvery::Off {
        return Ok(());
    }
    let file = root.join("updates.toml");
    let before = Findings::load(&file).map_err(|e| format!("{e:?}"))?;
    let (updates, offered) = scan(root, extra)?;
    let installed: Vec<String> = Registry::load(&root.join("installed.toml"))
        .map_err(|e| format!("{e:?}"))?
        .list()
        .iter()
        .map(|i| i.name.clone())
        .collect();
    // New = offered now, not offered at the last check, not installed; never on the very first check (no flood).
    let mut new: Vec<Novelty> = before
        .new
        .iter()
        .filter(|n| !installed.contains(&n.name))
        .cloned()
        .collect();
    if !before.offered.is_empty() {
        for o in &offered {
            if !before.offered.contains(&o.name)
                && !installed.contains(&o.name)
                && !new.iter().any(|n| n.name == o.name)
            {
                new.push(o.clone());
            }
        }
    }
    let after = Findings {
        checked: now(),
        update: updates,
        new,
        offered: offered.into_iter().map(|o| o.name).collect(),
    };
    after.save(&file).map_err(|e| format!("{e:?}"))?;
    let news = after.update != before.update || after.new != before.new;
    if settings.notify {
        badge(after.update.len() + after.new.len());
        if news && (!after.update.is_empty() || !after.new.is_empty()) {
            tell(&after);
        }
    }
    if settings.install == InstallMode::Auto && !after.update.is_empty() {
        crate::updates::update_all(root, extra)?;
    }
    Ok(())
}

fn tell(f: &Findings) {
    let mut lines: Vec<String> = f
        .update
        .iter()
        .map(|u| format!("{} {} -> {}", u.name, u.from, u.to))
        .collect();
    lines.extend(f.new.iter().map(|n| format!("New: {}", n.name)));
    let _ = Command::new("notify-send")
        .args([
            "--app-name=FreeSynergy.Store",
            "--icon=system-software-update",
            "FreeSynergy.Store",
        ])
        .arg(lines.join("\n"))
        .status();
}

/// Launcher badge (Unity `LauncherEntry` over D-Bus; KDE and some GNOME setups show it). Best effort.
fn badge(count: usize) {
    let _ = Command::new("gdbus")
        .args([
            "emit",
            "--session",
            "--object-path",
            "/com/canonical/unity/launcherentry/1",
            "--signal",
            "com.canonical.Unity.LauncherEntry.Update",
            "application://net.freesynergy.Store.desktop",
        ])
        .arg(format!(
            "{{'count': <int64 {count}>, 'count-visible': <{}>}}",
            count > 0
        ))
        .output();
}

/// `fs-store settings [check <off|start|daily|hourly> | install <ask|auto> | notify <on|off>]`.
pub(crate) fn settings(root: &Path, rest: &[String]) -> Result<(), String> {
    let file = root.join("settings.toml");
    let mut s = Settings::load(&file).map_err(|e| format!("{e:?}"))?;
    match rest {
        [] => {}
        [k, v] if k == "check" => {
            s.check = match v.as_str() {
                "off" => CheckEvery::Off,
                "start" => CheckEvery::Start,
                "daily" => CheckEvery::Daily,
                "hourly" => CheckEvery::Hourly,
                _ => return Err("check wants off, start, daily or hourly".into()),
            }
        }
        [k, v] if k == "install" => {
            s.install = match v.as_str() {
                "ask" => InstallMode::Ask,
                "auto" => InstallMode::Auto,
                _ => return Err("install wants ask or auto".into()),
            }
        }
        [k, v] if k == "notify" => s.notify = matches!(v.as_str(), "on" | "yes"),
        _ => return Err("usage: fs-store settings [check off|start|daily|hourly | install ask|auto | notify on|off]".into()),
    }
    if !rest.is_empty() {
        s.save(&file).map_err(|e| format!("{e:?}"))?;
    }
    println!(
        "check {:?}, install {:?}, notify {}",
        s.check, s.install, s.notify
    );
    Ok(())
}
