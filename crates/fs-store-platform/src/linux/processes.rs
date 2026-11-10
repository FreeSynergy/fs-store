//! Pattern: Strategy (Linux: `/proc`).

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::{ProcessInfo, Processes};

/// Process information from `/proc`, control through `kill`.
pub(super) struct ProcProcesses;

impl Processes for ProcProcesses {
    fn mine(&self) -> Vec<ProcessInfo> {
        let Ok(me) = fs::metadata("/proc/self").map(|m| m.uid()) else {
            return Vec::new();
        };
        let Ok(procs) = fs::read_dir("/proc") else {
            return Vec::new();
        };
        procs
            .flatten()
            .filter_map(|p| {
                let pid = p.file_name().to_str()?.parse::<u32>().ok()?;
                if fs::metadata(p.path()).ok()?.uid() != me {
                    return None;
                }
                let raw = fs::read(p.path().join("cmdline")).unwrap_or_default();
                let argv = raw
                    .split(|b| *b == 0)
                    .filter(|s| !s.is_empty())
                    .map(|s| String::from_utf8_lossy(s).into_owned())
                    .collect();
                Some(ProcessInfo {
                    pid,
                    argv,
                    exe: fs::read_link(p.path().join("exe")).ok(),
                })
            })
            .collect()
    }

    fn is_alive(&self, pid: u32) -> bool {
        // A zombie is gone for our purpose: its state is `Z`.
        fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|s| {
            s.rsplit(')')
                .next()
                .is_some_and(|r| !r.trim_start().starts_with('Z'))
        })
    }

    fn request_stop(&self, pid: u32) -> Result<(), String> {
        Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(drop)
            .map_err(|e| format!("kill: {e}"))
    }

    fn spawn_detached(&self, program: &Path, args: &[String]) -> io::Result<()> {
        // `setsid -f`: an own session, so the program outlives the store and its terminal. Without `setsid`, a plain child
        // in its own process group that nobody waits for.
        // `setsid -f` answers success even when the program is not there; a path that does not exist is said so here.
        if program.components().count() > 1 && !program.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} does not exist", program.display()),
            ));
        }
        let mut setsid = Command::new("setsid");
        setsid.arg("-f").arg(program).args(args);
        match detached(setsid).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => return Err(io::Error::other(format!("setsid ended with {status}"))),
            Err(_) => {}
        }
        let mut plain = Command::new(program);
        plain.args(args).process_group(0);
        let mut child = detached(plain).spawn()?;
        // Reap the child when it ends, without holding anything up.
        std::thread::spawn(move || child.wait().map(|_| ()));
        Ok(())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let opener = std::env::split_paths(&path)
            .map(|dir| dir.join("xdg-open"))
            .find(|p| fs::metadata(p).is_ok_and(|m| m.is_file()))
            .ok_or_else(|| "xdg-open not found".to_owned())?;
        self.spawn_detached(&opener, &[url.to_owned()])
            .map_err(|e| e.to_string())
    }
}

fn detached(mut command: Command) -> Command {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}
