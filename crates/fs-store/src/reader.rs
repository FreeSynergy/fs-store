// reader.rs — StoreReader: fetches catalog data from a StoreSource.
//
// StoreReader is the only component that talks to the Store repository.
// It fetches TOML catalogs, FTL help files, TOML i18n snippets, and icons.
// The Inventory calls StoreReader to populate itself.
//
// Design: Strategy Pattern via StoreSource.
//   StoreSource::Local  → reads from the filesystem (dev / CI)
//   StoreSource::Http   → fetches from the network (production)
//   New sources         → new StoreSource variant, no reader code touched.

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::i18n::{FtlContent, FtlKind, I18nSnippets};
use crate::source::StoreSource;

// ── StoreReader ───────────────────────────────────────────────────────────────

/// Fetches raw data from a FreeSynergy Store source.
///
/// All methods are async and return `anyhow::Result`. Callers (the Inventory)
/// are responsible for parsing and caching.
pub struct StoreReader {
    source: StoreSource,
    http: reqwest::Client,
}

impl StoreReader {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a reader for the given source.
    pub fn new(source: StoreSource) -> Self {
        Self {
            source,
            http: reqwest::Client::new(),
        }
    }

    /// Create a reader pointed at the official FreeSynergy Store.
    pub fn official() -> Self {
        let source = StoreSource::official();
        info!("StoreReader: official store");
        Self::new(source)
    }

    // ── TOML ──────────────────────────────────────────────────────────────────

    /// Fetch and parse a TOML file at a Store-root-relative path.
    pub async fn fetch_toml<T>(&self, rel_path: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let text = self.fetch_text(rel_path).await?;
        toml::from_str(&text).with_context(|| format!("parsing TOML '{rel_path}'"))
    }

    // ── FTL ───────────────────────────────────────────────────────────────────

    /// Fetch a FTL file for a package, with locale fallback to `"en"`.
    ///
    /// # Arguments
    ///
    /// * `catalog_dir` — Store-relative package directory,
    ///   e.g. `"packages/containers/forgejo"`
    /// * `locale` — BCP-47 locale code, e.g. `"de"`
    /// * `kind` — which FTL file to fetch
    ///
    /// Falls back to `"en"` if the requested locale is not available.
    pub async fn fetch_ftl(
        &self,
        catalog_dir: &str,
        locale: &str,
        kind: FtlKind,
    ) -> Result<FtlContent> {
        let filename = kind.filename();
        let primary = format!("{catalog_dir}/{locale}/{filename}");

        match self.fetch_text(&primary).await {
            Ok(content) => Ok(FtlContent {
                package_id: package_id_from_dir(catalog_dir),
                locale: locale.to_owned(),
                kind,
                content,
            }),
            Err(_) if locale != "en" => {
                debug!("StoreReader: '{locale}/{filename}' not found, falling back to 'en'");
                let fallback = format!("{catalog_dir}/en/{filename}");
                let content = self.fetch_text(&fallback).await.with_context(|| {
                    format!("FTL fallback 'en/{filename}' not found for {catalog_dir}")
                })?;
                Ok(FtlContent {
                    package_id: package_id_from_dir(catalog_dir),
                    locale: "en".to_owned(),
                    kind,
                    content,
                })
            }
            Err(e) => Err(e),
        }
    }

    // ── i18n snippets ─────────────────────────────────────────────────────────

    /// Fetch a TOML i18n snippet file for the given locale.
    ///
    /// Snippet files live at `packages/i18n/{locale}/store.toml` (or similar).
    /// The raw TOML text is returned inside [`I18nSnippets::content`].
    pub async fn fetch_i18n_snippets(&self, locale: &str) -> Result<I18nSnippets> {
        let path = format!("packages/i18n/{locale}/store.toml");
        let content = self.fetch_text(&path).await?;
        Ok(I18nSnippets {
            locale: locale.to_owned(),
            path,
            content: Some(content),
        })
    }

    // ── Icons ─────────────────────────────────────────────────────────────────

    /// Fetch a package icon as raw SVG bytes.
    ///
    /// `icon_path` is the Store-relative path from `PackageData::icon_path`,
    /// e.g. `"packages/containers/forgejo/forgejo.svg"`.
    pub async fn fetch_icon(&self, icon_path: &str) -> Result<Vec<u8>> {
        let text = self.fetch_text(icon_path).await?;
        Ok(text.into_bytes())
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    async fn fetch_text(&self, rel_path: &str) -> Result<String> {
        match &self.source {
            StoreSource::Local(root) => {
                let path = root.join(rel_path);
                debug!("StoreReader: reading {}", path.display());
                std::fs::read_to_string(&path)
                    .with_context(|| format!("reading '{}'", path.display()))
            }
            StoreSource::Http(base) => {
                let url = format!(
                    "{}/{}",
                    base.trim_end_matches('/'),
                    rel_path.trim_start_matches('/')
                );
                debug!("StoreReader: fetching {url}");
                self.http
                    .get(&url)
                    .send()
                    .await
                    .with_context(|| format!("GET {url}"))?
                    .error_for_status()
                    .with_context(|| format!("HTTP error for {url}"))?
                    .text()
                    .await
                    .with_context(|| format!("reading response from {url}"))
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract the package id from a catalog directory path.
///
/// `"packages/containers/forgejo"` → `"forgejo"`
fn package_id_from_dir(catalog_dir: &str) -> String {
    catalog_dir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(catalog_dir)
        .to_owned()
}
