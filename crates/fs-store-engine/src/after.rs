//! Pattern: Facade over the follow-up steps of an install and of a removal (commands on the PATH, menu entries). Both
//! the window and the commands call these two functions, so the order is one and nothing is swallowed: every problem
//! comes back as a sentence, the caller says it. None of these problems undoes the install or removal that happened.

use std::path::Path;

use crate::facets::Form;
use crate::own_files::package_dir;

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

/// Facet commands and menu entries first, then the program commands, for the form of `version` that is active now: an
/// unpacked package (`packages/<name>/<version>` exists) is reached through `current/<name>`; an rpm or deb (the package
/// manager put the program in `/usr/bin`) gets links to that program, no program links, and what the store unpacked
/// earlier for the package is taken away.
#[must_use]
pub fn settle_install(
    root: &Path,
    name: &str,
    version: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
) -> Settled {
    settle_install_with(
        root,
        name,
        version,
        facets,
        bin_dir,
        apps_dir,
        Path::new("/usr/bin"),
    )
}

/// [`settle_install`] with the folder of the package manager given (tests use their own).
#[must_use]
pub fn settle_install_with(
    root: &Path,
    name: &str,
    version: &str,
    facets: &[String],
    bin_dir: &Path,
    apps_dir: &Path,
    system: &Path,
) -> Settled {
    let mut out = Settled::default();
    let unpacked = package_dir(root, name, version).is_dir();
    let form = if unpacked {
        Form::Unpacked
    } else {
        out.problems
            .extend(crate::retire::retire_unpacked(root, name, bin_dir));
        Form::System(system)
    };
    match crate::link_facets_for(root, name, facets, bin_dir, apps_dir, form, system) {
        Ok(made) => out.made = made,
        Err(e) => out.problems.push(format!(
            "{name}: facet commands and menu entries not made: {e}"
        )),
    }
    if unpacked && root.join("current").join(name).exists() {
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
