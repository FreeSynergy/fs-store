//! The last step of a job that installed: "end server". Pattern: Template method step of `Installer::commit_observed`.
//!
//! The servers of the program were ended before the files were replaced (a running program is never replaced).
//! This step looks again after the install: a server that runs now (one a window started in between, or one that was
//! not found before) is the old version and is ended too (`Control.stop`, SIGTERM for an old one). The new version is
//! not started here: the window of the program starts it when needed (`connect_matching`).

use std::time::Duration;

use fs_store_core::Version;

use crate::service::InstallFailure;
use crate::{end_servers, find_servers};

/// How long one server gets to end.
pub(crate) const END_WAIT: Duration = Duration::from_secs(15);

/// Ends every server of `name` that still runs after a good install. A failure is never silent: the result becomes
/// `ServerNotEnded` (installed, but ...). A result that is already an error is left alone.
pub(crate) fn end_old_servers(
    name: &str,
    wait: Duration,
    result: &mut Result<Version, InstallFailure>,
) {
    let Ok(version) = *result else { return };
    let servers = find_servers(name, std::process::id());
    if servers.is_empty() {
        return;
    }
    if let Err(reason) = end_servers(&servers, wait) {
        *result = Err(InstallFailure::ServerNotEnded { version, reason });
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use super::*;

    #[test]
    fn nothing_running_changes_nothing() {
        let mut result = Ok(Version::new(1, 0, 0));
        end_old_servers("fs-ending-none", Duration::from_millis(200), &mut result);
        assert_eq!(result, Ok(Version::new(1, 0, 0)));
    }

    #[test]
    fn an_error_is_left_alone() {
        let mut result = Err(InstallFailure::Checksum);
        end_old_servers("fs-ending-none", Duration::from_millis(200), &mut result);
        assert_eq!(result, Err(InstallFailure::Checksum));
    }

    #[test]
    fn a_server_that_will_not_end_is_told_never_silent() {
        // `sh serve` shows up as the command line `fs-ending-stubborn serve`; the script ignores SIGTERM.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("serve"),
            "trap '' TERM\nwhile :; do sleep 1; done\n",
        )
        .unwrap();
        let mut child = Command::new("sh")
            .arg0("fs-ending-stubborn")
            .arg("serve")
            .current_dir(dir.path())
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let mut result = Ok(Version::new(1, 2, 3));
        end_old_servers(
            "fs-ending-stubborn",
            Duration::from_millis(300),
            &mut result,
        );
        child.kill().unwrap(); // the test's own cleanup; the store never kills
        child.wait().unwrap();
        match result {
            Err(InstallFailure::ServerNotEnded { version, reason }) => {
                assert_eq!(version, Version::new(1, 2, 3));
                assert!(reason.contains("did not end in time"), "{reason}");
            }
            other => panic!("expected ServerNotEnded, got {other:?}"),
        }
    }
}
