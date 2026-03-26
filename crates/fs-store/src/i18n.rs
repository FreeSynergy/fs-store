// i18n.rs — i18n support for the Store.
//
// FreeSynergy uses two formats for translations:
//
//   TOML snippets  — short UI strings (labels, placeholders, messages).
//                    Stored as `{locale}.toml` files in each package's
//                    i18n directory and in the top-level Store i18n dir.
//                    Loaded into the fs-i18n SnippetPlugin system.
//
//   FTL files      — long-form texts (help pages, field descriptions,
//                    package descriptions). Stored as `{locale}/description.ftl`,
//                    `{locale}/overview.ftl`, `{locale}/fields.ftl` per package.
//                    Loaded by StoreReader on demand.
//
// The Store ships i18n data in its own repository (FreeSynergy/Store).
// `StoreReader` fetches both formats from there.

use serde::{Deserialize, Serialize};

// ── I18nSnippets ──────────────────────────────────────────────────────────────

/// A TOML-based i18n snippet bundle for one locale.
///
/// Snippets are key-value pairs used for short UI strings. They are loaded
/// into the `fs-i18n` runtime via `SnippetPlugin`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct I18nSnippets {
    /// BCP-47 locale code, e.g. `"de"`, `"en"`.
    pub locale: String,

    /// Store-relative path to the TOML file, e.g. `"packages/i18n/de/store.toml"`.
    pub path: String,

    /// Raw TOML content, if already fetched.
    #[serde(skip)]
    pub content: Option<String>,
}

// ── FtlContent ────────────────────────────────────────────────────────────────

/// A fetched FTL file for one package and locale.
///
/// FTL files contain long-form translated text: description, overview,
/// field help. They are fetched on demand by `StoreReader` and cached locally
/// after install.
#[derive(Debug, Clone)]
pub struct FtlContent {
    /// Package id this FTL belongs to.
    pub package_id: String,

    /// BCP-47 locale, e.g. `"de"`. Falls back to `"en"` if not available.
    pub locale: String,

    /// FTL kind: `"description"`, `"overview"`, or `"fields"`.
    pub kind: FtlKind,

    /// Raw FTL source text.
    pub content: String,
}

/// Which FTL file within a package's locale directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FtlKind {
    /// Store detail view description (`description.ftl`). Always required.
    Description,

    /// Manager help page overview (`overview.ftl`). Required for installable packages.
    Overview,

    /// Per-field help texts (`fields.ftl`). Required when the package has variables.
    Fields,
}

impl FtlKind {
    /// The filename for this FTL kind.
    #[must_use]
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Description => "description.ftl",
            Self::Overview => "overview.ftl",
            Self::Fields => "fields.ftl",
        }
    }
}

impl std::fmt::Display for FtlKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.filename())
    }
}
