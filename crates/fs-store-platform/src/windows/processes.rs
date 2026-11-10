//! Pattern: Strategy (Windows: the process table, through the `sysinfo` crate).

use std::io;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::{ProcessInfo, Processes};

/// `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP`.
const BACKGROUND: u32 = 0x0800_0200;

/// Process information from the process table, control through `taskkill` (without `/F`: never by force).
pub(super) struct TableProcesses;

fn table(only: Option<Pid>) -> System {
    let mut system = System::new();
    let kind = ProcessRefreshKind::nothing()
        .with_exe(UpdateKind::Always)
        .with_cmd(UpdateKind::Always)
        .with_user(UpdateKind::Always);
    let wanted = only.map(|p| [p]);
    let which = wanted
        .as_ref()
        .map_or(ProcessesToUpdate::All, |p| ProcessesToUpdate::Some(p));
    system.refresh_processes_specifics(which, true, kind);
    system
}

impl Processes for TableProcesses {
    fn mine(&self) -> Vec<ProcessInfo> {
        let system = table(None);
        let me = system
            .process(Pid::from_u32(std::process::id()))
            .and_then(|p| p.user_id().cloned());
        system
            .processes()
            .values()
            .filter(|p| me.is_some() && p.user_id().cloned() == me)
            .map(|p| ProcessInfo {
                pid: p.pid().as_u32(),
                argv: p
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect(),
                exe: p.exe().map(Path::to_path_buf),
            })
            .collect()
    }

    fn is_alive(&self, pid: u32) -> bool {
        table(Some(Pid::from_u32(pid)))
            .process(Pid::from_u32(pid))
            .is_some()
    }

    fn request_stop(&self, pid: u32) -> Result<(), String> {
        // A program with a window gets a close request. One without a window cannot be asked: it stays, and the caller
        // finds out by waiting; nothing is ever ended by force.
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("taskkill: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "process {pid} cannot be asked to end in order (it has no window)"
            ))
        }
    }

    fn spawn_detached(&self, program: &Path, args: &[String]) -> io::Result<()> {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(BACKGROUND)
            .spawn()
            .map(drop)
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        // `explorer.exe <address>` opens the default browser; it splits its command line at commas and takes more than web
        // addresses, so only a plain http(s) address is passed.
        if !(url.starts_with("https://") || url.starts_with("http://"))
            || url.contains([',', '"', ' ', '\r', '\n'])
        {
            return Err(format!("not a plain web address: {url}"));
        }
        self.spawn_detached(Path::new("explorer.exe"), &[url.to_owned()])
            .map_err(|e| e.to_string())
    }
}
