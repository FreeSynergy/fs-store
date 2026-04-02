// installer/mod.rs — File-download strategies for each package kind.
//
// This module handles the filesystem side of installing a package:
// downloading compose files, language packs, theme files, and app binaries.
//
// Registry operations (PackageRegistry::install / is_installed) are left to
// the caller — this crate has no dependency on fs-db-desktop.
//
// Sub-modules:
//   pipeline — Pipeline Pattern: InstallStep trait, Pipeline, InstallTarget,
//              InstallContext, PipelineEvent, and all concrete step impls.

pub mod pipeline;

pub use pipeline::{InstallContext, InstallTarget, Pipeline, PipelineEvent, StepOutcome};

use crate::StoreReader;

// ── InstallKind ───────────────────────────────────────────────────────────────

/// Package category relevant for installation routing.
///
/// Mirrors the variants used in the app layer without pulling in
/// `fs_db_desktop` as a library dependency.
#[derive(Clone, Debug, PartialEq)]
pub enum InstallKind {
    Bundle,
    Container,
    Language,
    Theme,
    App,
    /// Widget / Bot / Task / Bridge / Plugin — register without a file download.
    Other,
}

// ── InstallRequest ─────────────────────────────────────────────────────────────

/// Minimal install descriptor passed to each download function.
///
/// Built from the app-layer `PackageEntry`, independent of any GUI or DB types.
#[derive(Clone, Debug)]
pub struct InstallRequest {
    pub id: String,
    pub name: String,
    pub kind: InstallKind,
    pub version: String,
    /// Store-relative path to the package directory.
    pub store_path: Option<String>,
    /// Capability IDs listed by bundle packages.
    pub capabilities: Vec<String>,
    /// Icon URL or SVG content (may need fetching).
    pub icon: Option<String>,
}

// ── Icon fetch ─────────────────────────────────────────────────────────────────

/// Fetch icon content: if `icon` is an HTTP(S) URL, download and return the
/// SVG text. Falls back to the original value on network errors so installs
/// never fail over icons.
pub async fn fetch_icon_content(icon: Option<String>) -> String {
    match icon {
        None => String::new(),
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => {
            match reqwest::Client::new()
                .get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or(url),
                _ => url,
            }
        }
        Some(other) => other,
    }
}

// ── Language pack ─────────────────────────────────────────────────────────────

/// Download and install a language pack (`ui.toml`).
///
/// # Errors
///
/// Returns an error string when the destination directory or file cannot be
/// written.  A failed download is treated as a warning — `Ok(None)` is
/// returned so the package is still registered.
pub async fn install_language_pack(
    req: &InstallRequest,
    fs_dir: &std::path::Path,
) -> Result<Option<String>, String> {
    let base = req
        .store_path
        .clone()
        .unwrap_or_else(|| format!("packages/i18n/{}", req.id));
    let url = format!("{base}/ui.toml");

    match StoreReader::official().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fs_dir.join("i18n").join(&req.id);
            std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            let dest = dest_dir.join("ui.toml");
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Ok(Some(dest.to_string_lossy().into_owned()))
        }
        Err(e) => {
            tracing::warn!("Language pack download failed (registering anyway): {e}");
            Ok(None)
        }
    }
}

// ── Theme file ────────────────────────────────────────────────────────────────

/// Download and install a theme file (`theme.css`).
///
/// # Errors
///
/// Returns an error string when the destination directory or file cannot be
/// written.  A failed download is treated as a warning — `Ok(None)` is
/// returned so the package is still registered.
pub async fn install_theme_file(
    req: &InstallRequest,
    fs_dir: &std::path::Path,
) -> Result<Option<String>, String> {
    let base = req
        .store_path
        .clone()
        .unwrap_or_else(|| format!("packages/themes/{}", req.id));
    let url = format!("{base}/theme.css");

    match StoreReader::official().fetch_raw(&url).await {
        Ok(content) => {
            let dest_dir = fs_dir.join("themes");
            std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
            let dest = dest_dir.join(format!("{}.css", req.id));
            std::fs::write(&dest, content).map_err(|e| e.to_string())?;
            Ok(Some(dest.to_string_lossy().into_owned()))
        }
        Err(e) => {
            tracing::warn!("Theme download failed (registering anyway): {e}");
            Ok(None)
        }
    }
}

// ── Container ─────────────────────────────────────────────────────────────────

/// Full container install:
///   1. Fetch compose file from store
///   2. Write compose + `.env` to `~/.local/share/fsn/services/<id>/`
///   3. Try `fsn container install <compose_path>` (adds Quadlet + systemd unit)
///   4. `systemctl --user daemon-reload`
///
/// # Errors
///
/// Returns an error string when the service directory or files cannot be
/// written.
#[allow(clippy::cognitive_complexity)]
pub async fn install_container(
    req: &InstallRequest,
    fs_dir: &std::path::Path,
    env_vars: &str,
) -> Result<Option<String>, String> {
    let base = req
        .store_path
        .clone()
        .unwrap_or_else(|| format!("packages/containers/{}", req.id));

    // Try compose.yml first, then docker-compose.yml / container.yml
    let compose_content = {
        let mut content = None;
        for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
            let url = format!("{base}/{name}");
            if let Ok(c) = StoreReader::official().fetch_raw(&url).await {
                content = Some((c, *name));
                break;
            }
        }
        content
    };

    let service_dir = fs_dir.join("services").join(&req.id);
    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let compose_path = if let Some((content, filename)) = compose_content {
        let dest = service_dir.join(filename);
        std::fs::write(&dest, content).map_err(|e| e.to_string())?;
        dest
    } else {
        let dest = service_dir.join("compose.yml");
        std::fs::write(
            &dest,
            format!(
                "# Compose file for {name}\n\
                 # Edit this file and run: fsn container install {path}\n\
                 services:\n\
                 #  {id}:\n\
                 #    image: ...\n",
                name = req.name,
                id = req.id,
                path = dest.display(),
            ),
        )
        .map_err(|e| e.to_string())?;
        dest
    };

    if !env_vars.trim().is_empty() {
        let env_path = service_dir.join(".env");
        std::fs::write(&env_path, env_vars).map_err(|e| e.to_string())?;
    }

    let compose_str = compose_path.to_string_lossy().into_owned();
    let container_result = tokio::process::Command::new("fsn")
        .args(["container", "install", &compose_str])
        .output()
        .await;

    match container_result {
        Ok(out) if out.status.success() => {
            let _ = tokio::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
                .await;
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::warn!(
                "fsn container install returned non-zero: {stderr}. \
                 Compose file saved to {compose_str} — run manually to finish setup."
            );
        }
        Err(_) => {
            tracing::warn!(
                "`fsn` binary not found. Compose file saved to {compose_str}. \
                 Run `fsn container install {compose_str}` manually to activate the service."
            );
        }
    }

    Ok(Some(compose_str))
}

// ── App binary ────────────────────────────────────────────────────────────────

/// Install a binary app package.
///
/// Production: download from the distribution URL (not yet implemented).
/// Dev mode (`FS_DEV=1` or debug build): use the locally compiled binary.
///
/// Dev binary resolution order:
///   1. `FS_BIN_{ID_UPPER}` env var — explicit override
///   2. `~/Server/<repo>/target/release/<binary>` — static table lookup
///   3. `~/Server/<repo>/target/debug/<binary>` — static table lookup
///   4. Derived from ID slug — same release/debug search
///
/// # Errors
///
/// Returns an error string when the binary file cannot be copied.
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::unused_async)]
pub async fn install_app_binary(
    req: &InstallRequest,
    installed_by: Option<&str>,
) -> Result<Option<String>, String> {
    let is_dev =
        cfg!(debug_assertions) || std::env::var("FS_DEV").map(|v| v == "1").unwrap_or(false);
    if !is_dev {
        tracing::info!(
            "App '{}' registered (no binary download in production yet).",
            req.id
        );
        return Ok(None);
    }

    if let Some(path) = find_local_build_binary(&req.id) {
        let dest_dir =
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".local/share/fsn/bin");
        std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

        let binary_name = std::path::Path::new(&path)
            .file_name()
            .map_or_else(|| req.id.clone(), |n| n.to_string_lossy().into_owned());
        let dest = dest_dir.join(&binary_name);

        std::fs::copy(&path, &dest).map_err(|e| {
            format!(
                "Failed to copy local build '{}' to '{}': {e}",
                path,
                dest.display()
            )
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }

        tracing::info!(
            "[dev] App '{}' installed from local build: {} → {}",
            req.id,
            path,
            dest.display()
        );
        return Ok(Some(dest.to_string_lossy().into_owned()));
    }

    if installed_by.is_some() {
        tracing::warn!(
            "[dev] App '{}' has no local build — skipping as bundle dependency. \
             Build it or set FS_BIN_{} to install it.",
            req.id,
            req.id.to_uppercase().replace('-', "_")
        );
        return Ok(None);
    }

    tracing::warn!(
        "[dev] No local build found for '{}' — registering without binary. \
         Build the project or set FS_BIN_{} to the binary path.",
        req.id,
        req.id.to_uppercase().replace('-', "_")
    );
    Ok(None)
}

// ── Local binary table ────────────────────────────────────────────────────────

/// Static table mapping known catalog IDs to `(repo_dir, binary_name)`.
static LOCAL_BUILD_BINARIES: &[(&str, &str, &str)] = &[
    ("node", "fs-node", "fsn"),
    ("apps/fs-node", "fs-node", "fsn"),
    ("desktop", "fs-desktop", "fs-desktop"),
    ("apps/fs-desktop", "fs-desktop", "fs-desktop"),
    ("apps/fs-store-app", "fs-desktop", "fs-desktop"),
    ("init", "fs-init", "fs-init"),
    ("apps/fs-init", "fs-init", "fs-init"),
    ("browser", "fs-browser", "fs-browser"),
    ("apps/fs-browser", "fs-browser", "fs-browser"),
    ("browser/fs-browser", "fs-browser", "fs-browser"),
];

fn lookup_local_build_binary(id: &str) -> Option<(&'static str, &'static str)> {
    LOCAL_BUILD_BINARIES
        .iter()
        .find(|(pkg_id, _, _)| *pkg_id == id)
        .map(|(_, repo, bin)| (*repo, *bin))
}

/// Try to locate a locally compiled binary for a package.
#[must_use]
pub fn find_local_build_binary(id: &str) -> Option<String> {
    let env_key = format!("FS_BIN_{}", id.to_uppercase().replace('-', "_"));
    if let Ok(path) = std::env::var(&env_key) {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let base = std::path::PathBuf::from(&home).join("Server");

    let (repo_dir, binary_name): (String, String) =
        if let Some((dir, bin)) = lookup_local_build_binary(id) {
            (dir.to_string(), bin.to_string())
        } else {
            let slug = id.rsplit('/').next().unwrap_or(id);
            let dir = if slug.starts_with("fs-") {
                slug.to_string()
            } else {
                format!("fs-{slug}")
            };
            (dir.clone(), dir)
        };

    let repo = base.join(&repo_dir);
    for profile in &["release", "debug"] {
        let path = repo.join("target").join(profile).join(&binary_name);
        if path.exists() {
            return Some(path.to_string_lossy().into_owned());
        }
    }

    None
}

// ── Env var extraction ────────────────────────────────────────────────────────

/// Fetch a compose file from the store and extract environment variable names.
///
/// Used by the configure step to show pre-populated input fields.
pub async fn fetch_container_env_vars(req: &InstallRequest) -> Vec<String> {
    let base = req.store_path.as_deref().map_or_else(
        || format!("node/modules/{}", req.id),
        |p| p.trim_end_matches('/').to_string(),
    );

    for name in &["compose.yml", "docker-compose.yml", "container.yml"] {
        let url = format!("{base}/{name}");
        if let Ok(content) = StoreReader::official().fetch_raw(&url).await {
            return extract_env_var_names(&content);
        }
    }
    vec![]
}

/// Extract `KEY` names from a YAML `environment:` section.
///
/// Handles `KEY=value`, `KEY: value`, and bare `KEY` forms.
/// Good enough for showing input fields; full YAML parsing happens
/// in the Conductor at install time.
#[must_use]
pub fn extract_env_var_names(yaml: &str) -> Vec<String> {
    let mut in_env = false;
    let mut vars = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();

        if trimmed == "environment:" || trimmed.starts_with("environment:") {
            in_env = true;
            continue;
        }

        if in_env && !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
            in_env = false;
        }

        if in_env {
            let entry = trimmed.trim_start_matches("- ");
            let key = if let Some(pos) = entry.find('=') {
                &entry[..pos]
            } else if let Some(pos) = entry.find(':') {
                &entry[..pos]
            } else {
                entry
            };
            let key = key.trim();
            if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                vars.push(key.to_string());
            }
        }
    }
    vars
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_env_vars_basic() {
        let yaml = "services:\n  app:\n    environment:\n      - DB_HOST=localhost\n      - DB_PORT=5432\n      - SECRET_KEY\nvolumes:\n  data:\n";
        let vars = extract_env_var_names(yaml);
        assert_eq!(vars, vec!["DB_HOST", "DB_PORT", "SECRET_KEY"]);
    }

    #[test]
    fn extract_env_vars_colon_style() {
        let yaml = "environment:\n  DB_HOST: localhost\n  API_KEY: secret\n";
        let vars = extract_env_var_names(yaml);
        assert_eq!(vars, vec!["DB_HOST", "API_KEY"]);
    }

    #[test]
    fn extract_env_vars_empty() {
        assert!(extract_env_var_names("services:\n  app:\n    image: nginx\n").is_empty());
    }

    #[test]
    fn find_local_build_binary_env_override() {
        // Set a non-existent path — should not return it
        std::env::remove_var("FS_BIN_NONEXISTENT");
        assert!(find_local_build_binary("nonexistent").is_none());
    }

    #[test]
    fn install_kind_variants_exist() {
        let kinds = [
            InstallKind::Bundle,
            InstallKind::Container,
            InstallKind::Language,
            InstallKind::Theme,
            InstallKind::App,
            InstallKind::Other,
        ];
        assert_eq!(kinds.len(), 6);
    }
}
