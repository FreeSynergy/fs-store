//! Pattern: Strategy (Windows: an executable is told by its extension, privacy comes from the folder's access list, and
//! elevation is a group of the access token).

use std::fs;
use std::io::{self, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::Permissions;

/// `FILE_FLAG_OPEN_REPARSE_POINT`: open the link itself, never what it points at.
const OPEN_REPARSE_POINT: u32 = 0x0020_0000;
/// The "High" mandatory level: a token of an administrator that was allowed to act as one.
const HIGH_LEVEL: &str = "S-1-16-12288";

/// Rights as Windows knows them.
pub(super) struct WindowsPermissions;

impl Permissions for WindowsPermissions {
    fn is_executable(&self, path: &Path) -> bool {
        let known = ["exe", "cmd", "bat", "com"];
        fs::metadata(path).is_ok_and(|m| m.is_file())
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| known.contains(&e.to_ascii_lowercase().as_str()))
    }

    fn create_private_dir(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        let user = std::env::var("USERNAME").map_err(io::Error::other)?;
        let domain = std::env::var("USERDOMAIN").unwrap_or_default();
        let who = if domain.is_empty() {
            user
        } else {
            format!("{domain}\\{user}")
        };
        // No inherited rights, full control for this person alone.
        let status = Command::new("icacls")
            .arg(dir)
            .args(["/inheritance:r", "/grant:r"])
            .arg(format!("{who}:(OI)(CI)F"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!("icacls ended with {status}")))
        }
    }

    fn create_private_file(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(OPEN_REPARSE_POINT)
            .open(path)
            .and_then(|mut f| f.write_all(bytes))
    }

    fn is_elevated(&self) -> bool {
        Command::new("whoami")
            .arg("/groups")
            .output()
            .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains(HIGH_LEVEL))
    }
}
