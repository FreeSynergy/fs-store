//! Pattern: Strategy / Port (starting another program sits behind a trait, so the model is tested with a fake).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Starts programs as processes of their own.
pub trait Launcher: Send + Sync {
    /// True if `command` can be started (found on the PATH or in the store's command folder).
    fn exists(&self, command: &str) -> bool;
    /// Starts `command` so that it outlives the store.
    ///
    /// # Errors
    /// A developer text when it could not be started.
    fn start(&self, command: &str) -> Result<(), String>;
}

/// The real launcher: `setsid -f` when available (own session, survives the store and its terminal), else a
/// plain child with nothing attached that nobody waits for.
#[derive(Debug, Clone)]
pub struct ProcessLauncher {
    extra: PathBuf,
}

impl ProcessLauncher {
    /// Also looks in `extra` (the folder the store links commands into), for it may not be on the PATH.
    #[must_use]
    pub fn new(extra: PathBuf) -> Self {
        Self { extra }
    }

    fn find(&self, command: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH").unwrap_or_default();
        std::env::split_paths(&path)
            .chain(std::iter::once(self.extra.clone()))
            .map(|dir| dir.join(command))
            .find(|p| is_file(p))
    }
}

fn is_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_file())
}

fn detached(mut command: Command) -> Command {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

impl Launcher for ProcessLauncher {
    fn exists(&self, command: &str) -> bool {
        !command.contains('/') && self.find(command).is_some()
    }

    fn start(&self, command: &str) -> Result<(), String> {
        let path = self.find(command).ok_or_else(|| "not found".to_owned())?;
        let mut setsid = Command::new("setsid");
        setsid.arg("-f").arg(&path);
        match detached(setsid).status() {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("setsid ended with {status}")),
            Err(_) => {
                let mut child = detached(Command::new(&path))
                    .spawn()
                    .map_err(|e| e.to_string())?;
                // Reap the child when it ends, without holding anything up.
                std::thread::spawn(move || child.wait().map(|_| ()));
                Ok(())
            }
        }
    }
}
