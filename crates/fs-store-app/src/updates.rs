//! Pattern: Strategy / Port (settings, findings and the background check sit behind a trait, so the model is
//! tested with a fake and no process is started in tests).

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use fs_store_sources::{Findings, Registry, Settings};

/// A program of the register and its switch for automatic updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// Name.
    pub name: String,
    /// Whether it is updated by itself when the install mode is "Automatically".
    pub auto_update: bool,
}

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
    /// True if the daily update timer is switched on. Default: no timer.
    fn timer_enabled(&self) -> bool {
        false
    }
    /// Switches the daily update timer on or off (as its own process); returns what it printed.
    ///
    /// # Errors
    /// What the process said, or why it could not start.
    fn timer_set(&self, _on: bool) -> Result<String, String> {
        Err("no timer".to_owned())
    }
    /// The programs of the register, with their switch for automatic updates. Default: none.
    ///
    /// # Errors
    /// A developer text when the register cannot be read.
    fn programs(&self) -> Result<Vec<Program>, String> {
        Ok(Vec::new())
    }
    /// Switches automatic updates of `name` on or off and saves the register.
    ///
    /// # Errors
    /// A developer text when `name` is unknown or the register cannot be written.
    fn set_auto_update(&self, _name: &str, _on: bool) -> Result<(), String> {
        Err("no register".to_owned())
    }
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

    /// `FS_STORE_ROOT`, or the store's folder of this system, like the commands.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::new(fs_store_platform::current().paths.store_root())
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
        let exe = fs_store_engine::own_exe().map_err(|e| e.to_string())?;
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

    fn timer_enabled(&self) -> bool {
        fs_store_platform::current().background.timer_enabled()
    }

    fn timer_set(&self, on: bool) -> Result<String, String> {
        let exe = fs_store_engine::own_exe().map_err(|e| e.to_string())?;
        let out = Command::new(exe)
            .args(["timer", if on { "enable" } else { "disable" }])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        let said = |bytes: &[u8]| String::from_utf8_lossy(bytes).trim().to_owned();
        if out.status.success() {
            Ok(said(&out.stdout))
        } else {
            let err = said(&out.stderr);
            Err(if err.is_empty() {
                said(&out.stdout)
            } else {
                err
            })
        }
    }

    fn programs(&self) -> Result<Vec<Program>, String> {
        let registry =
            Registry::load(&self.root.join("installed.toml")).map_err(|e| format!("{e:?}"))?;
        Ok(registry
            .list()
            .iter()
            .map(|e| Program {
                name: e.name.clone(),
                auto_update: e.auto_update,
            })
            .collect())
    }

    fn set_auto_update(&self, name: &str, on: bool) -> Result<(), String> {
        let path = self.root.join("installed.toml");
        let mut registry = Registry::load(&path).map_err(|e| format!("{e:?}"))?;
        if !registry.set_auto_update(name, on) {
            return Err(format!("{name} is not in the register"));
        }
        registry.save().map_err(|e| format!("{e:?}"))
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
