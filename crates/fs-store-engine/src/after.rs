//! Pattern: Facade over the follow-up steps of an install and of a removal (commands on the PATH, menu entries). Both
//! the window and the commands call these two functions, so the order is one and nothing is swallowed: every problem
//! comes back as a sentence, the caller says it. None of these problems undoes the install or removal that happened.

use std::path::Path;

/// What the follow-up of an install did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Settled {
    /// Facet commands made (for the register).
    pub made: Vec<String>,
    /// Program commands linked.
    pub linked: Vec<String>,
    /// What went wrong or was left alone, one sentence each.
    pub problems: Vec<String>,
}

/// Facet commands and menu entries first, then the program commands. A package without `current/<name>` (installed by
/// rpm or deb: the package manager put the program in `/usr/bin`) gets no program links.
#[must_use]
pub fn settle_install(
    root: &Path,
    name: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
) -> Settled {
    let mut out = Settled::default();
    match crate::link_facets(root, name, facets, bin_dir, apps_dir) {
        Ok(made) => out.made = made,
        Err(e) => out.problems.push(format!(
            "{name}: facet commands and menu entries not made: {e}"
        )),
    }
    if root.join("current").join(name).exists() {
        match crate::link_programs(root, name, bin_dir) {
            Ok((linked, skipped)) => {
                out.linked = linked;
                out.problems.extend(skipped.into_iter().map(|s| {
                    format!("{name}: `{s}` is in the way (another file of that name), not linked")
                }));
            }
            Err(e) => out
                .problems
                .push(format!("{name}: commands not linked: {e}")),
        }
    }
    out
}

/// Takes the commands and menu entries of `name` away (only ours). Returns the problems; the caller still forgets the
/// package in the register, because the files are gone.
#[must_use]
pub fn settle_remove(
    root: &Path,
    name: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
) -> Vec<String> {
    let mut problems = Vec::new();
    if let Err(e) = crate::unlink_programs(root, name, bin_dir) {
        problems.push(format!("{name}: commands not removed: {e}"));
    }
    if let Err(e) = crate::unlink_facets(root, name, facets, bin_dir, apps_dir) {
        problems.push(format!(
            "{name}: facet commands and menu entries not removed: {e}"
        ));
    }
    problems
}
