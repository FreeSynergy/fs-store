//! The places follow the environment: the three `FS_STORE_*` variables win on every system.
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use fs_store_platform::{Paths, Vars, current, display_name};

fn vars(pairs: &'static [(&'static str, &'static str)]) -> Vars {
    Box::new(move |name| {
        pairs
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| OsString::from(v))
    })
}

fn own(vars: Vars) -> Box<dyn Paths> {
    #[cfg(unix)]
    return Box::new(fs_store_platform::LinuxPaths::with_vars(vars));
    #[cfg(windows)]
    return Box::new(fs_store_platform::WindowsPaths::with_vars(vars));
}

#[test]
fn the_overrides_win() {
    let paths = own(vars(&[
        ("FS_STORE_ROOT", "/r"),
        ("FS_STORE_BIN", "/b"),
        ("FS_STORE_APPS", "/a"),
    ]));
    assert_eq!(paths.store_root(), PathBuf::from("/r"));
    assert_eq!(paths.bin_dir(), PathBuf::from("/b"));
    assert_eq!(paths.menu_dir(), PathBuf::from("/a"));
}

#[cfg(unix)]
#[test]
fn linux_works_below_home() {
    let paths = own(vars(&[("HOME", "/home/u")]));
    assert_eq!(
        paths.store_root(),
        Path::new("/home/u/.local/share/fs-store")
    );
    assert_eq!(paths.bin_dir(), Path::new("/home/u/.local/bin"));
    assert_eq!(
        paths.menu_dir(),
        Path::new("/home/u/.local/share/applications")
    );
    assert_eq!(
        paths.native_program_dirs("fs-tasks"),
        [PathBuf::from("/usr/bin")]
    );
}

#[cfg(windows)]
#[test]
fn windows_works_below_the_profile() {
    let paths = own(vars(&[
        ("LOCALAPPDATA", r"C:\Users\u\AppData\Local"),
        ("APPDATA", r"C:\Users\u\AppData\Roaming"),
        ("ProgramFiles", r"C:\Program Files"),
    ]));
    assert_eq!(
        paths.store_root(),
        Path::new(r"C:\Users\u\AppData\Local\FreeSynergy\Store")
    );
    assert_eq!(
        paths.bin_dir(),
        Path::new(r"C:\Users\u\AppData\Local\FreeSynergy\bin")
    );
    assert!(
        paths
            .menu_dir()
            .ends_with(r"Start Menu\Programs\FreeSynergy")
    );
    assert_eq!(
        paths.native_program_dirs("fs-tasks"),
        [PathBuf::from(r"C:\Program Files\FreeSynergy.Tasks")]
    );
    // Windows does not tell `C:\A` from `c:\a`, and a prefix of letters is not a folder.
    assert!(paths.is_below(Path::new(r"C:\Data\Pkg\x.exe"), Path::new(r"c:\data\pkg")));
    assert!(!paths.is_below(Path::new(r"C:\Data\Pkg2\x.exe"), Path::new(r"c:\data\pkg")));
}

#[test]
fn below_is_a_folder_boundary_and_the_display_name_is_readable() {
    let paths = &current().paths;
    let dir = std::env::temp_dir().join("pkg");
    assert!(paths.is_below(&dir.join("x"), &dir));
    assert!(paths.is_below(&dir, &dir));
    assert!(!paths.is_below(&std::env::temp_dir().join("pkg2"), &dir));
    assert_eq!(display_name("fs-tasks"), "FreeSynergy.Tasks");
    assert_eq!(display_name("store"), "FreeSynergy.Store");
    let _ = Path::new("");
}

#[test]
fn the_places_of_this_machine_are_not_empty() {
    let paths = &current().paths;
    for p in [paths.store_root(), paths.bin_dir(), paths.menu_dir()] {
        assert!(p.components().count() > 1, "{}", p.display());
    }
}
