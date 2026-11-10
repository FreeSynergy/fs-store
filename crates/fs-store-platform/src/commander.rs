//! Pattern: Strategy (running a command: the real one, or a fake in tests).

use std::process::Command;

/// Runs a program with arguments; `Err` carries what went wrong (exit status and stderr).
pub trait Commander {
    /// # Errors
    /// Text describing the failure.
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String>;
}

/// The real thing: starts the process and waits.
#[derive(Debug, Default, Clone)]
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
