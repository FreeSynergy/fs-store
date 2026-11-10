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
/// Other openers may read, not write, rename or delete.
const FILE_SHARE_READ: u32 = 1;
/// The "High" mandatory level: a token of an administrator that was allowed to act as one.
const HIGH_LEVEL: &str = "S-1-16-12288";

/// The SID of the person running this, from `whoami /user /fo csv /nh` (`"DOMAIN\\name","S-1-5-21-..."`).
fn own_sid() -> io::Result<String> {
    let out = Command::new("whoami")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .rsplit(',')
        .next()
        .map(|s| s.trim_matches('"').to_owned())
        .filter(|s| s.starts_with("S-1-"))
        .ok_or_else(|| io::Error::other("whoami did not tell the SID"))
}

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
        // No inherited rights; full control for this person (by SID, not by a name that the environment could change), and for
        // the administrators and the system: the setup is started elevated, maybe by another account of this machine.
        let who = own_sid()?;
        let status = Command::new("icacls")
            .arg(dir)
            .args(["/inheritance:r", "/grant:r"])
            .arg(format!("*{who}:(OI)(CI)F"))
            .args(["*S-1-5-32-544:(OI)(CI)F", "*S-1-5-18:(OI)(CI)F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!("icacls ended with {status}")))
        }
    }

    fn create_private_file(&self, path: &Path, bytes: &[u8]) -> io::Result<Box<dyn Send>> {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(OPEN_REPARSE_POINT)
            .open(path)
            .and_then(|mut f| f.write_all(bytes))?;
        // Closed for writing, opened again for reading that lets others read but not write, rename or delete: an elevated
        // setup then starts the file that was written, not one a process of the same person swapped in meanwhile.
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(OPEN_REPARSE_POINT)
            .open(path)?;
        Ok(Box::new(held))
    }

    fn is_elevated(&self) -> bool {
        Command::new("whoami")
            .arg("/groups")
            .output()
            .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains(HIGH_LEVEL))
    }
}
