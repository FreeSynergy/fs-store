//! Pattern: Strategy (`RunningCheck` through the platform's `Processes`: `/proc` on Linux, the process table on Windows).

use std::path::{Path, PathBuf};

use fs_store_core::RunningCheck;
use fs_store_platform::current;

/// A package is running when some process of this person executes a file below `root/packages/<name>/` (or, for a
/// package of the system, in the program folder the setup made). The calling process itself is ignored, so the store can
/// update itself: its own old files stay on disk (the switch is a pointer, nothing is overwritten) and the new version
/// starts at the next launch.
#[derive(Debug)]
pub struct ProcRunningCheck {
    root: PathBuf,
    ignore: u32,
}

impl ProcRunningCheck {
    /// Looks for processes started from packages under `root`, ignoring this process.
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self::ignoring(root, std::process::id())
    }

    /// Like [`Self::new`], ignoring the process `pid` instead.
    #[must_use]
    pub fn ignoring(root: &Path, pid: u32) -> Self {
        Self {
            root: root.to_owned(),
            ignore: pid,
        }
    }
}

impl RunningCheck for ProcRunningCheck {
    fn is_running(&self, name: &str) -> bool {
        let platform = current();
        let base = self.root.join("packages").join(name);
        platform.processes.mine().iter().any(|p| {
            p.pid != self.ignore
                && p.exe
                    .as_ref()
                    .is_some_and(|exe| platform.paths.is_below(exe, &base))
        })
    }
}
