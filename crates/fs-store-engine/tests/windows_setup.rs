//! The real thing on a Windows machine: a setup.exe made like the ones of `fs-release` (NSIS, per machine) is installed through
//! the `Exe` strategy (a real `Start-Process -Verb RunAs`, `/S`), seen through the registry, and taken away through its own
//! uninstaller. Needs a setup to install: the gate builds one and names it in `FS_PROBE_SETUP`; without it the test says so and
//! passes (a developer machine must not install things).
#![cfg(windows)]

mod common;

use std::fs;
use std::path::PathBuf;

use fs_store_core::{InstallStrategy, Version};
use fs_store_engine::{Exe, SystemCommander};
use fs_store_platform::current;

#[test]
fn a_real_setup_is_installed_seen_and_removed() {
    let Some(setup) = std::env::var_os("FS_PROBE_SETUP") else {
        eprintln!("FS_PROBE_SETUP is not set: no setup to install, nothing done");
        return;
    };
    let name = "fs-probe"; // the setup was made as `FreeSynergy.Probe`, which is what the store derives from this name
    let packages = &current().packages;
    assert_eq!(
        packages.installed_version(name),
        None,
        "not installed before"
    );
    let tmp = tempfile::tempdir().unwrap();
    let mut exe = Exe::new(
        &tmp.path().join("exe"),
        vec![fs::read(setup).unwrap()],
        SystemCommander,
    );
    exe.install(&common::verified(name, Version::new(1, 2, 3)))
        .unwrap();
    assert_eq!(
        packages.installed_version(name),
        Some(Version::new(1, 2, 3))
    );
    let dir: PathBuf = current().paths.own_program_dir(name).unwrap();
    assert!(dir.join("probe.txt").is_file(), "{}", dir.display());
    assert!(packages.owns_executable(&dir.join("probe.exe")));
    assert_eq!(fs::read_dir(tmp.path().join("exe")).unwrap().count(), 0);

    packages.remove(name, &mut SystemCommander).unwrap();
    assert_eq!(packages.installed_version(name), None);
    assert!(!dir.exists(), "{} is gone", dir.display());
}
