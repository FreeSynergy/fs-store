//! Pattern: Strategy (what a command does with an offer that this store version cannot install). A package the person NAMED (`install X`,
//! `update X`) that cannot be installed is a failure with a non-zero exit, never a quiet success; only the sweep over everything
//! (`update --all`, which the daily timer runs) skips such a package with a note, because it did not ask for that one.

/// What to do with the offer.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Nothing is locked: go on.
    Proceed,
    /// Leave it, tell the person, no failure.
    Skip(String),
    /// Leave it and fail the command (exit code not zero).
    Fail(String),
}

/// `locked` is the reason of the offer (`CatalogEntry::locked`); `sweep` is true for `update --all`.
pub(crate) fn outcome(name: &str, locked: Option<&str>, sweep: bool) -> Outcome {
    match locked {
        None => Outcome::Proceed,
        Some(why) if sweep => Outcome::Skip(format!(
            "{name} is locked in this store version ({why}); not changed"
        )),
        Some(why) => Outcome::Fail(format!(
            "{name} cannot be installed by this store version: {why}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_named_locked_package_fails_and_only_the_sweep_skips_it() {
        let why = Some("not supported by this store version");
        assert!(matches!(outcome("tasks", why, false), Outcome::Fail(m) if m.contains("tasks")));
        assert!(matches!(outcome("tasks", why, true), Outcome::Skip(_)));
        // counter-proof: an unlocked package goes on in both modes
        assert_eq!(outcome("tasks", None, false), Outcome::Proceed);
        assert_eq!(outcome("tasks", None, true), Outcome::Proceed);
    }
}
