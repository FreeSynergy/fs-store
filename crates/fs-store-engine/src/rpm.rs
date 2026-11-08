//! Pattern: Strategy (`InstallStrategy` for `rpm`, through `dnf`), with a small Strategy for running commands
//! (`Commander`) so tests need neither root nor dnf.
//! The person confirms root themselves: `pkexec` asks for it, we never see a password.

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use fs_store_core::{Format, InstallError, InstallStrategy, VerifiedPackage, Version};

/// Runs a program with arguments; `Err` carries what went wrong (exit status and stderr).
pub trait Commander {
    /// # Errors
    /// Text describing the failure.
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String>;
}

/// The real thing: starts the process and waits.
#[derive(Debug, Default)]
pub struct SystemCommander;

impl Commander for SystemCommander {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String> {
        let out = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("{program}: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!(
                "{program} {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
}

/// Installs a verified rpm with `pkexec dnf install`. dnf works in one transaction, so a failed install has
/// changed nothing and `restore` has nothing to undo; going back to an older rpm is the person's decision
/// (`dnf downgrade`), the store does not do it silently.
pub struct Rpm<C: Commander> {
    dir: PathBuf,
    files: Vec<Vec<u8>>,
    commander: C,
}

impl<C: Commander> Rpm<C> {
    /// `dir` is a private directory for the file handed to dnf (created with mode 0700).
    #[must_use]
    pub fn new(dir: &Path, files: Vec<Vec<u8>>, commander: C) -> Self {
        Self {
            dir: dir.to_owned(),
            files,
            commander,
        }
    }
}

impl<C: Commander> InstallStrategy for Rpm<C> {
    fn format(&self) -> Format {
        Format::Rpm
    }

    fn install(&mut self, package: &VerifiedPackage) -> Result<(), InstallError> {
        let p = package.package();
        let io = |e: std::io::Error| InstallError(e.to_string());
        fs::create_dir_all(&self.dir).map_err(io)?;
        fs::set_permissions(
            &self.dir,
            std::os::unix::fs::PermissionsExt::from_mode(0o700),
        )
        .map_err(io)?;
        // Names nobody can guess, created exclusively and never through a link: nothing to swap after the check.
        let mut paths = Vec::new();
        let mut written = Ok(());
        for (n, bytes) in self.files.iter().enumerate() {
            let file = self.dir.join(format!(
                "{}-{}-{}-{n}.rpm",
                p.name.as_str(),
                p.version,
                std::process::id()
            ));
            let _ = fs::remove_file(&file);
            let one = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(0x2_0000) // O_NOFOLLOW
                .open(&file)
                .and_then(|mut f| f.write_all(bytes));
            paths.push(file);
            if let Err(e) = one {
                written = Err(e);
                break;
            }
        }
        let result = written.map_err(|e| e.to_string()).and_then(|()| {
            let names: Vec<String> = paths
                .iter()
                .map(|f| f.to_string_lossy().into_owned())
                .collect();
            // A local rpm needs no fresh repository metadata (measured 2026-11-08: a refresh of all repositories is what
            // makes `dnf install` slow); dnf still downloads metadata when there is none.
            let mut args = vec!["dnf", "install", "-y", "--setopt=metadata_expire=never"];
            args.extend(names.iter().map(String::as_str));
            self.commander.run("pkexec", &args)
        });
        for f in &paths {
            let _ = fs::remove_file(f);
        }
        result.map_err(InstallError)
    }

    fn restore(&mut self, _name: &str, _previous: Option<Version>) -> Result<(), InstallError> {
        Ok(())
    }
}
