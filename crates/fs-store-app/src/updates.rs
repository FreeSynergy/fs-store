//! Pattern: Strategy / Port (settings, findings and the background check sit behind a trait, so the model is
//! tested with a fake and no process is started in tests).

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use fs_store_sources::{Findings, Settings};

/// What the window needs for updates: the person's settings, what the last check found, and a way to check.
pub trait UpdateService: Send + Sync {
    /// The settings (defaults when there is no file).
    ///
    /// # Errors
    /// A developer text when the file cannot be read.
    fn settings(&self) -> Result<Settings, String>;
    /// Saves the settings.
    ///
    /// # Errors
    /// A developer text when writing fails.
    fn save_settings(&self, settings: &Settings) -> Result<(), String>;
    /// What the last check found (nothing when never checked).
    ///
    /// # Errors
    /// A developer text when the file cannot be read.
    fn findings(&self) -> Result<Findings, String>;
    /// Saves the findings.
    ///
    /// # Errors
    /// A developer text when writing fails.
    fn save_findings(&self, findings: &Findings) -> Result<(), String>;
    /// Runs the background check and waits for its end. Blocks: call it from a worker thread only.
    ///
    /// # Errors
    /// A developer text when it could not run or ended with an error.
    fn check(&self) -> Result<(), String>;
    /// Now, in Unix seconds.
    fn now(&self) -> u64;
}

/// The real service: files below the root, and the check is `fs-store background` as a child process.
#[derive(Debug, Clone)]
pub struct FileUpdates {
    root: PathBuf,
}

impl FileUpdates {
    /// Works below `root` (`settings.toml`, `updates.toml`).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// `FS_STORE_ROOT`, or `~/.local/share/fs-store`, like the commands.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::new(std::env::var_os("FS_STORE_ROOT").map_or_else(
            || {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join(".local/share/fs-store")
            },
            PathBuf::from,
        ))
    }
}

impl UpdateService for FileUpdates {
    fn settings(&self) -> Result<Settings, String> {
        Settings::load(&self.root.join("settings.toml")).map_err(|e| format!("{e:?}"))
    }

    fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        settings
            .save(&self.root.join("settings.toml"))
            .map_err(|e| format!("{e:?}"))
    }

    fn findings(&self) -> Result<Findings, String> {
        Findings::load(&self.root.join("updates.toml")).map_err(|e| format!("{e:?}"))
    }

    fn save_findings(&self, findings: &Findings) -> Result<(), String> {
        findings
            .save(&self.root.join("updates.toml"))
            .map_err(|e| format!("{e:?}"))
    }

    fn check(&self) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let status = Command::new(exe)
            .arg("background")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("fs-store background ended with {status}"))
        }
    }

    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

/// A service that has nothing: defaults, nothing found, saving goes nowhere, checking is not possible.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoUpdates;

impl UpdateService for NoUpdates {
    fn settings(&self) -> Result<Settings, String> {
        Ok(Settings {
            notify: true,
            ..Settings::default()
        })
    }
    fn save_settings(&self, _: &Settings) -> Result<(), String> {
        Ok(())
    }
    fn findings(&self) -> Result<Findings, String> {
        Ok(Findings::default())
    }
    fn save_findings(&self, _: &Findings) -> Result<(), String> {
        Ok(())
    }
    fn check(&self) -> Result<(), String> {
        Err("no checker".to_owned())
    }
    fn now(&self) -> u64 {
        0
    }
}
