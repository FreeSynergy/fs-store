//! Pattern: Strategy (Windows: a scheduled task of this person made with `schtasks`, and a balloon in the notification
//! area). A scheduled task of the person needs no administrator rights. The badge on a launcher needs a packaged app,
//! which a setup.exe is not: `badge` does nothing on purpose (Null object), the notice is the balloon.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use crate::{Background, Every};

const TASK: &str = r"FreeSynergy\Store update";
/// `CREATE_NO_WINDOW`.
const NO_WINDOW: u32 = 0x0800_0000;

/// The timer and notices of a Windows desktop.
pub(super) struct Scheduler;

fn schtasks(args: &[&str]) -> Result<String, String> {
    let out = Command::new("schtasks")
        .args(args)
        .creation_flags(NO_WINDOW)
        .output()
        .map_err(|e| format!("schtasks: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

impl Background for Scheduler {
    fn timer_enable(&self, exe: &Path, every: Every) -> Result<String, String> {
        let run = format!("\"{}\" background", exe.display());
        let every = if every == Every::Hourly {
            "HOURLY"
        } else {
            "DAILY"
        };
        schtasks(&["/Create", "/TN", TASK, "/TR", &run, "/SC", every, "/F"])
    }

    fn timer_disable(&self) -> Result<String, String> {
        schtasks(&["/Delete", "/TN", TASK, "/F"])
    }

    fn timer_status(&self) -> Result<String, String> {
        schtasks(&["/Query", "/TN", TASK, "/V", "/FO", "LIST"])
    }

    fn timer_enabled(&self) -> bool {
        schtasks(&["/Query", "/TN", TASK]).is_ok()
    }

    fn notify(&self, text: &str) {
        let script = "Add-Type -AssemblyName System.Windows.Forms, System.Drawing; \
             $n = New-Object System.Windows.Forms.NotifyIcon; \
             $n.Icon = [System.Drawing.SystemIcons]::Information; $n.Visible = $true; \
             $n.ShowBalloonTip(10000, 'FreeSynergy.Store', $env:FS_TEXT, 'Info'); \
             Start-Sleep -Seconds 12; $n.Dispose()";
        let _ = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-Command",
                script,
            ])
            .env("FS_TEXT", text)
            .creation_flags(NO_WINDOW)
            .spawn();
    }

    fn badge(&self, _count: usize) {}
}
