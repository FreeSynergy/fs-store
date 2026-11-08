//! Key generation: the root keys (for the person, to be kept offline) and the three role keys, plus a signed first `root.json`.

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::Path;
use std::time::Duration;

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::Ed25519KeyPair;
use jiff::Timestamp;
use tough::editor::signed::SignedRole;
use tough::key_source::{KeySource, LocalKeySource};
use tough::schema::{KeyHolder, RoleKeys, RoleType, Root};

use crate::{PublishError, block_on, fail};

/// Writes `root-1.pk8`..`root-N.pk8`, `targets.pk8`, `snapshot.pk8`, `timestamp.pk8` (Ed25519, PKCS#8 DER) and a first
/// `root.json` (version 1, valid for `root_validity`, `threshold` of the `root_keys` needed to change it, signed by all
/// root keys) into `dir`. Keep the `root-*.pk8` files offline; they are the person's, never CI secrets and never committed.
///
/// # Errors
/// `PublishError` on a bad threshold or when a file cannot be written.
pub fn generate_keys(
    dir: &Path,
    root_keys: usize,
    threshold: u64,
    root_validity: Duration,
) -> Result<(), PublishError> {
    let threshold = NonZeroU64::new(threshold)
        .filter(|t| t.get() <= root_keys as u64)
        .ok_or_else(|| PublishError("threshold must be 1..=number of root keys".to_owned()))?;
    std::fs::create_dir_all(dir).map_err(fail("create dir"))?;
    let rng = SystemRandom::new();
    let mut names: Vec<String> = (1..=root_keys).map(|i| format!("root-{i}")).collect();
    names.extend(["targets", "snapshot", "timestamp"].map(str::to_owned));
    for name in &names {
        let doc = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| PublishError(format!("generate {name}: {e}")))?;
        write_secret(&dir.join(format!("{name}.pk8")), doc.as_ref())?;
    }
    let sources = |wanted: &dyn Fn(&str) -> bool| -> Vec<Box<dyn KeySource>> {
        names
            .iter()
            .filter(|n| wanted(n))
            .map(|n| {
                Box::new(LocalKeySource {
                    path: dir.join(format!("{n}.pk8")),
                }) as Box<dyn KeySource>
            })
            .collect()
    };
    let root_sources = sources(&|n| n.starts_with("root-"));
    let all_sources = sources(&|_| true);
    let expires = Timestamp::now()
        .checked_add(root_validity)
        .map_err(fail("expiry"))?;
    block_on(async {
        let mut key_map = HashMap::new();
        let mut role_ids: HashMap<RoleType, Vec<_>> = HashMap::new();
        for (source, name) in all_sources.iter().zip(&names) {
            let key = source
                .as_sign()
                .await
                .map_err(|e| PublishError(format!("{name}: {e}")))?
                .tuf_key();
            let id = key.key_id().map_err(fail("key id"))?;
            key_map.insert(id.clone(), key);
            let role = match name.as_str() {
                "targets" => RoleType::Targets,
                "snapshot" => RoleType::Snapshot,
                "timestamp" => RoleType::Timestamp,
                _ => RoleType::Root,
            };
            role_ids.entry(role).or_default().push(id);
        }
        let roles = role_ids
            .into_iter()
            .map(|(role, keyids)| {
                let threshold = if role == RoleType::Root {
                    threshold
                } else {
                    NonZeroU64::MIN
                };
                (
                    role,
                    RoleKeys {
                        keyids,
                        threshold,
                        _extra: HashMap::new(),
                    },
                )
            })
            .collect();
        let root = Root {
            spec_version: "1.0.0".to_owned(),
            consistent_snapshot: false,
            version: NonZeroU64::MIN,
            expires,
            keys: key_map,
            roles,
            _extra: HashMap::new(),
        };
        let signed = SignedRole::new(root.clone(), &KeyHolder::Root(root), &root_sources, &rng)
            .await
            .map_err(fail("sign root"))?;
        std::fs::write(dir.join("root.json"), signed.buffer()).map_err(fail("write root.json"))
    })?
}

/// Writes a private key readable by the owner only.
fn write_secret(path: &Path, bytes: &[u8]) -> Result<(), PublishError> {
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    options
        .open(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(fail("write key"))
}
