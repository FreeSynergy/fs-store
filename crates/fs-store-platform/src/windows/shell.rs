//! Pattern: none (small helpers to run PowerShell and read what it answers; one place for the quoting rules).

use std::process::{Command, Stdio};

/// Runs a PowerShell script. Values go in through environment variables (`$env:NAME`), never into the script text, so
/// no path or title can change what the script does.
pub(super) fn powershell(script: &str, env: &[(&str, &str)]) -> Result<String, String> {
    let mut command = Command::new("powershell");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdin(Stdio::null());
    for (key, value) in env {
        command.env(key, value);
    }
    let out = command.output().map_err(|e| format!("powershell: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(format!(
            "powershell {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Quotes text as a PowerShell single-quoted string.
pub(super) fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}
