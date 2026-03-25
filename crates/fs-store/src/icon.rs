// icon.rs — IconSource trait and all concrete icon source types.
//
// Icons are always stored locally after first fetch (offline-first).
// Multiple sources can be queried in priority order.

use async_trait::async_trait;

// ── IconSource trait ──────────────────────────────────────────────────────────

/// A source from which package icons can be retrieved.
///
/// Implementations cover the Store's built-in icons, community icon sets
/// (Homarr, Simple Icons), AI-generated icons, and user-drawn icons.
///
/// All implementations must be `Send + Sync` for use across async tasks.
#[async_trait]
pub trait IconSource: Send + Sync {
    /// Display name of this source, e.g. `"FreeSynergy Store"`.
    fn name(&self) -> &str;

    /// Fetch the icon for `id` as raw SVG bytes.
    ///
    /// `id` is the icon identifier within this source (usually the package id).
    async fn fetch(&self, id: &str) -> anyhow::Result<Vec<u8>>;

    /// `true` when this source can generate icons on demand (e.g. AI sources).
    fn can_generate(&self) -> bool {
        false
    }

    /// `true` when this source supports user-drawn custom icons.
    fn can_draw(&self) -> bool {
        false
    }
}

// ── Concrete sources ──────────────────────────────────────────────────────────

/// Icons bundled in the FreeSynergy/Store repository (`shared/icons/`).
///
/// This is the primary source for all packages that declare an `icon_path`.
pub struct StoreIconSource {
    /// Base URL or local path of the Store root.
    pub store_base: String,
}

#[async_trait]
impl IconSource for StoreIconSource {
    fn name(&self) -> &str {
        "FreeSynergy Store"
    }

    async fn fetch(&self, id: &str) -> anyhow::Result<Vec<u8>> {
        // Resolve to shared/icons/{id}.svg relative to the Store root.
        let path = format!(
            "{}/shared/icons/{}.svg",
            self.store_base.trim_end_matches('/'),
            id
        );
        let bytes = reqwest::get(&path).await?.bytes().await?;
        Ok(bytes.to_vec())
    }
}
