//! Pattern: Strategy (Windows: `%LOCALAPPDATA%`, `%APPDATA%` and the program folder).

use std::path::{Path, PathBuf};

use crate::{Paths, Vars, display_name, process_vars};

/// Paths on Windows: `%LOCALAPPDATA%\FreeSynergy\Store`, `...\bin`, the Start menu folder `FreeSynergy`.
pub struct WindowsPaths {
    vars: Vars,
}

/// Makes a path comparable: no `\\?\` prefix, one kind of slash, lower case (Windows ignores case).
fn comparable(path: &Path) -> PathBuf {
    let text = path.to_string_lossy().replace('/', "\\").to_lowercase();
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

impl WindowsPaths {
    /// Reads the environment of this process.
    #[must_use]
    pub fn new() -> Self {
        Self::with_vars(process_vars())
    }

    /// Reads `vars` instead of the environment (tests).
    #[must_use]
    pub fn with_vars(vars: Vars) -> Self {
        Self { vars }
    }

    fn var(&self, name: &str) -> PathBuf {
        PathBuf::from((self.vars)(name).unwrap_or_default())
    }

    fn or_below(&self, over: &str, base: &str, below: &str) -> PathBuf {
        (self.vars)(over).map_or_else(|| self.var(base).join(below), PathBuf::from)
    }
}

impl Default for WindowsPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl Paths for WindowsPaths {
    fn store_root(&self) -> PathBuf {
        self.or_below("FS_STORE_ROOT", "LOCALAPPDATA", r"FreeSynergy\Store")
    }

    fn bin_dir(&self) -> PathBuf {
        self.or_below("FS_STORE_BIN", "LOCALAPPDATA", r"FreeSynergy\bin")
    }

    fn menu_dir(&self) -> PathBuf {
        self.or_below(
            "FS_STORE_APPS",
            "APPDATA",
            r"Microsoft\Windows\Start Menu\Programs\FreeSynergy",
        )
    }

    fn config_dir(&self) -> PathBuf {
        self.var("APPDATA")
    }

    fn native_program_dirs(&self, package: &str) -> Vec<PathBuf> {
        vec![self.var("ProgramFiles").join(display_name(package))]
    }

    fn own_program_dir(&self, package: &str) -> Option<PathBuf> {
        self.native_program_dirs(package).pop()
    }

    fn program_file_name(&self, command: &str) -> String {
        format!("{command}.exe")
    }

    fn command_files(&self, command: &str) -> Vec<String> {
        ["exe", "cmd", "bat"]
            .iter()
            .map(|e| format!("{command}.{e}"))
            .collect()
    }

    fn command_name(&self, file: &str) -> String {
        let path = Path::new(file);
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            path.file_stem()
                .map_or_else(|| file.to_owned(), |s| s.to_string_lossy().into_owned())
        } else {
            file.to_owned()
        }
    }

    fn is_below(&self, path: &Path, dir: &Path) -> bool {
        comparable(path).starts_with(comparable(dir))
    }
}
