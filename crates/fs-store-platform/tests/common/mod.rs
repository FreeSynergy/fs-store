//! Helpers shared by the tests of the platform parts (they run on every system).
#![allow(dead_code)]

use std::path::Path;
use std::process::{Child, Command, Stdio};

/// A program file that exists on this system (a copy of a tool every system has), made executable.
pub fn make_program(dir: &Path, name: &str) -> std::path::PathBuf {
    let file = dir.join(fs_store_platform::current().paths.program_file_name(name));
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&file, b"program").unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(&file, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    file
}

/// A child that runs for a while and does nothing.
pub fn sleeper() -> Child {
    #[cfg(unix)]
    let mut command = {
        let mut c = Command::new("sleep");
        c.arg("60");
        c
    };
    #[cfg(windows)]
    let mut command = {
        let mut c = Command::new("ping");
        c.args(["-n", "60", "127.0.0.1"]);
        c
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}
