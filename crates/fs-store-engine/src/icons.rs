//! Pattern: none (one function). Keeps the icon of a package where the menu entries can find it.

use std::fmt::Write;
use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// The icons are small; anything larger is not an icon.
const MAX_ICON: usize = 1024 * 1024;

/// Stores `bytes` as `root/icons/<name>.png` when they are a PNG of at most 1 MiB with the given SHA-256.
/// Returns whether it was stored; a wrong hash or a file that is not a PNG is refused, never stored.
///
/// # Errors
/// `io::Error` when the disk refuses.
pub fn store_icon(root: &Path, name: &str, bytes: &[u8], sha256: &str) -> io::Result<bool> {
    let plain = !name.is_empty() && !name.contains(['/', '\\']) && !name.starts_with('.');
    let hash = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
    if !plain
        || bytes.len() > MAX_ICON
        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || hash != sha256.to_ascii_lowercase()
    {
        return Ok(false);
    }
    let dir = root.join("icons");
    fs::create_dir_all(&dir)?;
    let tmp = dir.join(format!(".{name}.png.new"));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dir.join(format!("{name}.png")))?;
    Ok(true)
}
