//! Pattern: Strategy (Linux: a systemd user timer, `notify-send` and the Unity launcher badge over D-Bus).

use std::path::Path;
use std::process::Command;

use crate::{Background, Every, current};

const SERVICE: &str = "fs-store-update.service";
const TIMER: &str = "fs-store-update.timer";

/// The timer and notices of a Linux desktop.
pub(super) struct Systemd;

/// The two systemd user unit files that run `<exe> background` every `every` and a little after login.
#[must_use]
pub(super) fn unit_files(exe: &str, every: &str) -> (String, String) {
    let service = format!(
        "[Unit]\nDescription=FreeSynergy.Store: update installed programs\n\n[Service]\nType=oneshot\nExecStart={exe} background\n"
    );
    let timer = format!(
        "[Unit]\nDescription=FreeSynergy.Store: look for updates\n\n[Timer]\nOnBootSec=5min\nOnUnitActiveSec={every}\nRandomizedDelaySec=30min\nPersistent=true\n\n[Install]\nWantedBy=timers.target\n"
    );
    (service, timer)
}

fn systemctl(args: &[&str]) -> Result<String, String> {
    let out = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .output()
        .map_err(|e| format!("systemctl: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

impl Background for Systemd {
    fn timer_enable(&self, exe: &Path, every: Every) -> Result<String, String> {
        let config = current().paths.config_dir();
        if config.starts_with(".config") || config.as_os_str().is_empty() {
            return Err("HOME is not set".to_owned());
        }
        let dir = config.join("systemd/user");
        let every = if every == Every::Hourly { "1h" } else { "1d" };
        let (service, timer) = unit_files(&exe.to_string_lossy(), every);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(SERVICE), service).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(TIMER), timer).map_err(|e| e.to_string())?;
        systemctl(&["daemon-reload"])?;
        systemctl(&["enable", "--now", TIMER])
    }

    fn timer_disable(&self) -> Result<String, String> {
        let said = systemctl(&["disable", "--now", TIMER])?;
        let dir = current().paths.config_dir().join("systemd/user");
        for f in [SERVICE, TIMER] {
            let _ = std::fs::remove_file(dir.join(f));
        }
        systemctl(&["daemon-reload"]).map(|more| said + &more)
    }

    fn timer_status(&self) -> Result<String, String> {
        systemctl(&["status", TIMER, "--no-pager"])
    }

    fn timer_enabled(&self) -> bool {
        systemctl(&["is-enabled", TIMER]).is_ok()
    }

    fn notify(&self, text: &str) {
        let _ = Command::new("notify-send")
            .args([
                "--app-name=FreeSynergy.Store",
                "--icon=system-software-update",
                "FreeSynergy.Store",
            ])
            .arg(text)
            .status();
    }

    fn badge(&self, count: usize) {
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
}

#[cfg(test)]
mod tests {
    use super::unit_files;

    #[test]
    fn units_run_the_update_command_of_this_binary() {
        let (service, timer) = unit_files("/home/u/.local/bin/fs-store", "1d");
        assert!(service.contains("ExecStart=/home/u/.local/bin/fs-store background"));
        assert!(timer.contains("OnUnitActiveSec=1d") && timer.contains("WantedBy=timers.target"));
    }
}
