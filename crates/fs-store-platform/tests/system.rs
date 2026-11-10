//! The package manager of the system says nothing for a package that is not there, and refuses names that are not plain.
use fs_store_platform::{Commander, System, current};

struct Fake(Vec<String>);
impl Commander for Fake {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<(), String> {
        self.0.push(format!("{program} {}", args.join(" ")));
        Ok(())
    }
}

#[test]
fn an_unknown_package_has_no_version_and_the_system_matches_the_build() {
    let packages = &current().packages;
    assert_eq!(packages.installed_version("no-such-package-fs-test"), None);
    #[cfg(windows)]
    assert_eq!(packages.system(), System::Windows);
    #[cfg(unix)]
    assert_ne!(packages.system(), System::Windows);
}

#[test]
fn removing_refuses_names_that_are_not_plain() {
    let packages = &current().packages;
    let mut fake = Fake(Vec::new());
    if packages.system() != System::Other {
        assert!(packages.remove("../x", &mut fake).is_err());
        assert!(packages.remove("-rf", &mut fake).is_err());
        assert_eq!(fake.0.len(), 0);
    }
}

#[test]
fn elevation_runs_the_program_through_the_system_prompt() {
    let (program, args) = current().elevation.wrap("tool", &["a", "b"]);
    assert_ne!(program.len(), 0);
    assert!(args.iter().any(|a| a.contains("tool")));
}
