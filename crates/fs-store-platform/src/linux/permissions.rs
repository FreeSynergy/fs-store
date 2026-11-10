//! Pattern: Strategy (Linux: mode bits, `O_NOFOLLOW`, the user id).

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::Permissions;

const O_NOFOLLOW: i32 = 0x2_0000;

/// Rights as Unix knows them.
pub(super) struct UnixPermissions;

impl Permissions for UnixPermissions {
    fn is_executable(&self, path: &Path) -> bool {
        fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }

    fn create_private_dir(&self, dir: &Path) -> io::Result<()> {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
    }

    fn create_private_file(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .and_then(|mut f| f.write_all(bytes))
    }

    fn is_elevated(&self) -> bool {
        fs::metadata("/proc/self").is_ok_and(|m| m.uid() == 0)
    }
}
