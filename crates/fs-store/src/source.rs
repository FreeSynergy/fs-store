// source.rs — StoreSource: where catalog data is fetched from.
//
// Design: Open/Closed Principle.
// New source kinds (IPFS, OCI registry) = new variant, no existing code changed.

use std::path::PathBuf;

/// Identifies where a Store catalog can be found.
///
/// `StoreReader` accepts a `StoreSource` and resolves every relative catalog
/// path against it. The `Local` variant is used in development and CI; `Http`
/// is used in production.
#[derive(Debug, Clone)]
pub enum StoreSource {
    /// Local directory — the path is the Store root.
    ///
    /// Used during development when the Store repo is checked out locally.
    Local(PathBuf),

    /// HTTP base URL — all catalog paths are appended.
    ///
    /// In production this points to the raw GitHub content URL:
    /// `https://raw.githubusercontent.com/FreeSynergy/Store/main`
    Http(String),
}

impl StoreSource {
    /// The official FreeSynergy Store over HTTP.
    ///
    /// Override via the `FS_STORE_URL` environment variable.
    pub fn official() -> Self {
        let url = std::env::var("FS_STORE_URL").unwrap_or_else(|_| {
            "https://raw.githubusercontent.com/FreeSynergy/Store/main".to_owned()
        });
        Self::Http(url)
    }

    /// Resolve a Store-root-relative path to its full location string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fs_store::source::StoreSource;
    /// let s = StoreSource::Http("https://example.com/store".to_owned());
    /// assert_eq!(s.resolve("catalog.toml"), "https://example.com/store/catalog.toml");
    /// ```
    pub fn resolve(&self, rel_path: &str) -> String {
        match self {
            Self::Local(root) => root.join(rel_path).to_string_lossy().into_owned(),
            Self::Http(base) => format!(
                "{}/{}",
                base.trim_end_matches('/'),
                rel_path.trim_start_matches('/')
            ),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_resolve() {
        let s = StoreSource::Local("/srv/store".into());
        assert_eq!(s.resolve("catalog.toml"), "/srv/store/catalog.toml");
        assert_eq!(
            s.resolve("packages/containers/forgejo/catalog.toml"),
            "/srv/store/packages/containers/forgejo/catalog.toml"
        );
    }

    #[test]
    fn http_resolve_no_double_slash() {
        let s = StoreSource::Http("https://example.com/store/".to_owned());
        let url = s.resolve("catalog.toml");
        assert!(!url.contains("//catalog"), "double slash: {url}");
        assert_eq!(url, "https://example.com/store/catalog.toml");
    }
}
