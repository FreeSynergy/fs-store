//! Pattern: Query (what the question before an install says besides the steps: where it comes from, how big it is, which rights it
//! takes, and the announcement of the password of the system). The question is the one confirmation; nothing asks again after it.

use fs_store_core::Format;

use crate::words_lang::Lang;
use crate::words_msg as m;

use super::store::StoreModel;

impl StoreModel {
    /// True when the install of `name` goes through the package manager of the system (it will ask for the password).
    pub(super) fn installs_system_wide(&self, name: &str) -> bool {
        let Some(row) = self.available.iter().find(|r| r.package.name == name) else {
            return false;
        };
        matches!(
            self.backend.system_format(),
            f @ (Format::Rpm | Format::Deb) if row.package.native.contains(&f)
        )
    }

    /// The line "Source - Size - Rights" for `name`.
    pub(super) fn facts_line(&self, name: &str) -> String {
        let Some(row) = self.available.iter().find(|r| r.package.name == name) else {
            return String::new();
        };
        let lang = Lang::current();
        let source = self
            .sources
            .iter()
            .find(|s| s.id == row.package.source)
            .map_or_else(|| row.package.source.clone(), |s| s.url.clone());
        let rights = if self.installs_system_wide(name) {
            m::rights_system(lang)
        } else {
            m::rights_own(lang)
        };
        crate::words::fill(
            m::facts_line(lang),
            &[
                ("source", &source),
                ("size", &self.size_for_sentence(name)),
                ("rights", rights),
            ],
        )
    }

    /// The announcement of the password of the system, when one of `names` is installed that way.
    pub(super) fn password_notice(&self, names: &[String]) -> Option<String> {
        names
            .iter()
            .any(|n| self.installs_system_wide(n))
            .then(|| m::password_notice(Lang::current()).to_owned())
    }
}
