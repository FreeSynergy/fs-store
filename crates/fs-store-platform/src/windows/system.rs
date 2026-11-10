//! Pattern: Strategy (Windows: the NSIS `-setup.exe` is the package; what is installed is read from the registry, and a
//! setup is run through a UAC prompt, which the person confirms).

use std::path::{Path, PathBuf};
use std::process::Command;

use fs_store_core::Version;

use super::shell::quote;
use crate::{Commander, Elevation, System, SystemPackages, display_name};

const UNINSTALL: &str = r"HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall";

/// The packages made by the setup of `fs-release`.
pub(super) struct WindowsSystem {
    program_files: PathBuf,
}

impl WindowsSystem {
    pub(super) fn new() -> Self {
        Self {
            program_files: PathBuf::from(std::env::var_os("ProgramFiles").unwrap_or_default()),
        }
    }

    fn dir_of(&self, name: &str) -> PathBuf {
        self.program_files.join(display_name(name))
    }
}

/// The value after `DisplayVersion    REG_SZ` in the answer of `reg query`.
fn version_in(text: &str) -> Option<Version> {
    text.lines().find_map(|l| {
        let mut words = l.split_whitespace();
        (words.next() == Some("DisplayVersion") && words.next() == Some("REG_SZ"))
            .then(|| words.next().and_then(Version::parse))
            .flatten()
    })
}

impl SystemPackages for WindowsSystem {
    fn system(&self) -> System {
        System::Windows
    }

    fn installed_version(&self, name: &str) -> Option<Version> {
        let key = format!(r"{UNINSTALL}\{}", display_name(name));
        // A 32-bit setup writes into the 32-bit view of the registry; ask for both.
        ["/reg:64", "/reg:32"].iter().find_map(|view| {
            let out = Command::new("reg")
                .args(["query", &key, "/v", "DisplayVersion", view])
                .output()
                .ok()?;
            out.status
                .success()
                .then(|| version_in(&String::from_utf8_lossy(&out.stdout)))
                .flatten()
        })
    }

    fn owns_executable(&self, path: &Path) -> bool {
        let text = path.to_string_lossy().to_lowercase();
        let base = self.program_files.to_string_lossy().to_lowercase();
        !base.is_empty()
            && text
                .strip_prefix(&base)
                .is_some_and(|rest| rest.trim_start_matches('\\').starts_with("freesynergy."))
    }

    fn remove(&self, name: &str, commander: &mut dyn Commander) -> Result<(), String> {
        if fs_store_core::PackageName::new(name).is_none_or(|n| n.as_str() != name) {
            return Err(format!("`{name}` is not a plain package name"));
        }
        let dir = self.dir_of(name);
        let uninstaller = dir.join("uninstall.exe");
        // `_?=<dir>` makes the NSIS uninstaller wait until it is done instead of copying itself and returning at once.
        let place = format!("_?={}", dir.display());
        let (program, args) = Uac.wrap(&uninstaller.to_string_lossy(), &["/S", &place]);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        commander.run(&program, &args)?;
        // Run in place, it cannot delete itself nor the folder it sits in: that is left for us to do (the folder is the one
        // made from the plain name above, below the program folder).
        let folder = dir.to_string_lossy();
        let (program, args) = Uac.wrap("cmd.exe", &["/C", "rmdir", "/S", "/Q", &folder]);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        commander.run(&program, &args)
    }
}

/// Administrator rights through a UAC prompt (`Start-Process -Verb RunAs`); the exit code of the program is passed on.
pub(super) struct Uac;

impl Elevation for Uac {
    fn wrap(&self, program: &str, args: &[&str]) -> (String, Vec<String>) {
        let list = args.iter().map(|a| quote(a)).collect::<Vec<_>>().join(",");
        let with_args = if list.is_empty() {
            String::new()
        } else {
            format!(" -ArgumentList {list}")
        };
        let script = format!(
            "$p = Start-Process -FilePath {}{with_args} -Verb RunAs -Wait -PassThru; exit $p.ExitCode",
            quote(program)
        );
        (
            "powershell".to_owned(),
            vec![
                "-NoProfile".to_owned(),
                "-NonInteractive".to_owned(),
                "-Command".to_owned(),
                script,
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_is_read_from_the_answer_of_reg() {
        let text =
            "\r\nHKEY_LOCAL_MACHINE\\Software\\X\r\n    DisplayVersion    REG_SZ    1.2.3\r\n";
        assert_eq!(version_in(text), Version::parse("1.2.3"));
        assert_eq!(version_in("nothing"), None);
    }

    #[test]
    fn elevation_quotes_every_argument() {
        let (program, args) = Uac.wrap("C:\\a b\\setup.exe", &["/S", "it's"]);
        assert_eq!(program, "powershell");
        assert!(args[3].contains("'C:\\a b\\setup.exe' -ArgumentList '/S','it''s'"));
    }
}
