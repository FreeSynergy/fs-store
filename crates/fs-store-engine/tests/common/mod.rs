#![allow(dead_code)]
use std::fs;
use std::path::Path;

use fs_store_core::*;

pub fn archive(dir: &Path, file: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(format!("{file}.tar.gz"));
    let enc = flate2::write::GzEncoder::new(
        fs::File::create(&path).unwrap(),
        flate2::Compression::fast(),
    );
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(content.len() as u64);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", content.as_bytes())
        .unwrap();
    b.into_inner().unwrap().finish().unwrap();
    path
}

pub fn verified(name: &str, v: Version) -> VerifiedPackage {
    let p = Package {
        name: PackageName::new(name).unwrap(),
        kind: Kind::Program,
        version: v,
        format: Format::TarGz,
        source: SourceId(REFERENCE_ID.to_owned()),
    };
    SignaturePolicy::default()
        .admit(
            p,
            SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
        )
        .unwrap()
}

pub struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

/// Copies the stand-in server (`examples/serve_helper`) to `dir/<name>` (with the system's executable suffix) and returns it.
pub fn server_copy(dir: &Path, name: &str) -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap();
    let examples = exe.parent().unwrap().parent().unwrap().join("examples");
    let suffix = std::env::consts::EXE_SUFFIX;
    let helper = examples.join(format!("serve_helper{suffix}"));
    let copy = dir.join(format!("{name}{suffix}"));
    // A hard link, not a copy: a file that is open for writing in a parallel test cannot be started ("text file busy").
    if fs::hard_link(&helper, &copy).is_err() {
        fs::copy(&helper, &copy).unwrap_or_else(|e| panic!("{}: {e}", helper.display()));
    }
    copy
}

/// A program file that this system can start, with the system's executable name, in `dir`.
pub fn make_program(dir: &Path, name: &str) -> std::path::PathBuf {
    let file = dir.join(fs_store_platform::current().paths.program_file_name(name));
    fs::create_dir_all(dir).unwrap();
    fs::write(&file, b"program").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&file, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    file
}

/// Starts the command; a program file that a parallel test has just copied may still be open for writing in that test's
/// thread ("text file busy", error 26), which passes in a moment.
pub fn spawn(command: &mut std::process::Command) -> std::process::Child {
    for _ in 0..50 {
        match command.spawn() {
            Err(e) if e.raw_os_error() == Some(26) => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            other => return other.unwrap(),
        }
    }
    command.spawn().unwrap()
}
