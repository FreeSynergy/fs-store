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

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::catalog::{NamespaceIndex, RawCatalogEntry};
use crate::i18n::{FtlContent, FtlKind, I18nSnippets};
use crate::inventory::NamespaceMap;
use crate::package::{
    AppPackage, BundlePackage, ContainerPackage, ExternalPackage, IconSetPackage, LanguagePackage,
    Package, PackageData, PackageHelp, RepoPackage, TaskPackage, ThemePackage, WidgetPackage,
};
use crate::package::bundle::BundleComponent;
use crate::release::PackageRelease;
use crate::source::StoreSource;

// ── StoreReader ───────────────────────────────────────────────────────────────

/// Fetches raw data from a FreeSynergy Store source.
///
/// All methods are async and return `anyhow::Result`. Callers (the Inventory)
/// are responsible for caching.
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

    // ── Catalog loading ───────────────────────────────────────────────────────

    /// Load all namespaces and return a fully populated [`NamespaceMap`].
    ///
    /// Packages that fail to parse are skipped with a warning so one broken
    /// entry does not prevent the rest of the catalog from loading.
    pub async fn load_all(&self) -> Result<NamespaceMap> {
        let mut map = NamespaceMap::default();

        map.apps = self.load_namespace("packages/apps").await?;
        map.containers = self.load_namespace("packages/containers").await?;
        map.themes = self.load_namespace("packages/themes").await?;
        map.widgets = self.load_namespace("packages/widgets").await?;
        map.tasks = self.load_namespace("packages/tasks").await?;
        map.languages = self.load_namespace("packages/i18n").await?;
        map.icons = self.load_namespace("packages/icons").await?;
        map.bundles = self.load_namespace("packages/bundles").await?;
        map.externals = self.load_namespace("packages/external").await?;
        map.repos = self.load_namespace("packages/repos").await?;

        info!(
            "StoreReader: loaded {} packages across all namespaces",
            map.total_count()
        );

        Ok(map)
    }

    /// Load all packages from one namespace directory.
    ///
    /// Reads `{ns_path}/catalog.toml` as the namespace index, then fetches
    /// each package's individual `catalog.toml` listed in `[[packages]]`.
    ///
    /// Packages that fail to parse are skipped with a warning.
    pub async fn load_namespace(&self, ns_path: &str) -> Result<Vec<Arc<dyn Package>>> {
        let index_path = format!("{ns_path}/catalog.toml");
        let index: NamespaceIndex = self
            .fetch_toml(&index_path)
            .await
            .with_context(|| format!("loading namespace index '{index_path}'"))?;

        debug!(
            "StoreReader: namespace '{}' — {} package(s)",
            index.catalog.id,
            index.packages.len()
        );

        let mut packages: Vec<Arc<dyn Package>> = Vec::with_capacity(index.packages.len());

        for pkg_ref in &index.packages {
            // pkg_ref.catalog is relative to the namespace dir, e.g. "kanidm/catalog.toml"
            let catalog_path = format!("{ns_path}/{}", pkg_ref.catalog);
            match self.load_package(&catalog_path).await {
                Ok(pkg) => packages.push(pkg),
                Err(e) => warn!(
                    "StoreReader: skipping '{}' in '{}': {e:#}",
                    pkg_ref.id, ns_path
                ),
            }
        }

        Ok(packages)
    }

    /// Load and convert one package `catalog.toml` into a domain object.
    pub async fn load_package(&self, catalog_path: &str) -> Result<Arc<dyn Package>> {
        let entry: RawCatalogEntry = self
            .fetch_toml(catalog_path)
            .await
            .with_context(|| format!("parsing package catalog '{catalog_path}'"))?;

        catalog_entry_to_package(entry, catalog_path)
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

    // ── Raw content ───────────────────────────────────────────────────────────

    /// Fetch any Store-root-relative file as raw text.
    ///
    /// Use this for content files (compose templates, theme CSS, i18n snippets)
    /// that are not catalog TOML files. The path is relative to the Store root,
    /// e.g. `"packages/containers/forgejo/compose.yml"`.
    pub async fn fetch_raw(&self, rel_path: &str) -> Result<String> {
        self.fetch_text(rel_path).await
    }

    // ── Icons ─────────────────────────────────────────────────────────────────

    /// Fetch a package icon as raw SVG bytes.
    ///
    /// `icon_path` is the Store-relative path from `PackageData::icon_path`,
    /// e.g. `"packages/containers/forgejo/icon.svg"`.
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

// ── Catalog entry → domain object ────────────────────────────────────────────

/// Convert a raw catalog entry into the appropriate typed domain object.
///
/// `catalog_path` is the Store-root-relative path, e.g.
/// `"packages/apps/kanidm/catalog.toml"`. It is used to derive the package
/// directory for `icon_path` and `help.base_path`.
fn catalog_entry_to_package(
    entry: RawCatalogEntry,
    catalog_path: &str,
) -> Result<Arc<dyn Package>> {
    let pkg_dir = catalog_path.trim_end_matches("/catalog.toml");

    let icon_path = entry
        .package
        .icon
        .as_ref()
        .map(|icon| format!("{pkg_dir}/{icon}"));

    let release = PackageRelease::catalog_only(&entry.package.version);

    let data = PackageData {
        id: entry.package.id.clone(),
        name: entry.package.name.clone(),
        summary: entry.package.summary.clone(),
        description: entry.package.description.clone(),
        icon_path,
        tags: entry.package.tags.clone(),
        releases: vec![release],
        help: PackageHelp {
            base_path: pkg_dir.to_owned(),
            available_locales: vec!["en".to_owned()],
        },
    };

    let pkg: Arc<dyn Package> = match entry.package.package_type.as_str() {
        "app" => Arc::new(AppPackage {
            data,
            repo: entry
                .package
                .origin
                .as_ref()
                .and_then(|o| o.git.clone()),
        }),

        "container" => Arc::new(ContainerPackage {
            data,
            ports: vec![],
            variables: vec![],
            features: vec![],
        }),

        "widget" => Arc::new(WidgetPackage { data }),

        "bundle" => {
            let components = entry
                .bundle
                .map(|b| {
                    b.components
                        .into_iter()
                        .map(|c| BundleComponent {
                            id: c.id,
                            version: None,
                            required: true,
                        })
                        .collect()
                })
                .unwrap_or_default();
            Arc::new(BundlePackage { data, components })
        }

        "theme" => Arc::new(ThemePackage { data }),

        "language" => {
            let (locale, rtl) = entry
                .language
                .map(|l| {
                    let loc = l
                        .locale
                        .unwrap_or_else(|| entry.package.id.clone());
                    let rtl = l
                        .direction
                        .as_deref()
                        .map(|d| d == "rtl")
                        .unwrap_or(false);
                    (loc, rtl)
                })
                .unwrap_or_else(|| (entry.package.id.clone(), false));
            Arc::new(LanguagePackage { data, locale, rtl })
        }

        "icon_set" => Arc::new(IconSetPackage {
            data,
            icon_count: None,
            upstream_url: entry
                .package
                .origin
                .as_ref()
                .and_then(|o| o.website.clone()),
        }),

        "repo" => Arc::new(RepoPackage {
            data,
            url: entry.repo.as_ref().and_then(|r| r.url.clone()),
            branch: entry
                .repo
                .as_ref()
                .and_then(|r| r.branch.clone())
                .unwrap_or_else(|| "main".to_owned()),
            verified: entry.repo.map(|r| r.verified).unwrap_or(false),
        }),

        "external" => Arc::new(ExternalPackage {
            data,
            website: entry.package.origin.as_ref().and_then(|o| o.website.clone()),
            download: None,
            docs: entry.package.origin.as_ref().and_then(|o| o.docs.clone()),
        }),

        "task" => Arc::new(TaskPackage {
            data,
            listens: vec![],
            emits: vec![],
        }),

        "bootstrap" => {
            // Bootstrap is special (BootableInstaller, not Package).
            // If it appears in a regular namespace, treat as external.
            Arc::new(ExternalPackage {
                data,
                website: entry.package.origin.as_ref().and_then(|o| o.website.clone()),
                download: None,
                docs: None,
            })
        }

        other => anyhow::bail!(
            "unknown package type '{other}' in '{catalog_path}'"
        ),
    };

    Ok(pkg)
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
