#![allow(dead_code)]
//! What the cancel tests share: a fetch that reads in pieces and looks at the token, a fake signature check, tarballs of n files, the
//! installer, the names in a folder.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use fs_store_core::*;
use fs_store_engine::service::{Fetch, Installer};
use sha2::{Digest, Sha256};

pub struct Map(pub HashMap<String, Vec<u8>>, pub Mutex<Vec<Progress>>);
impl Fetch for Map {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0.get(url).cloned().ok_or_else(|| format!("404 {url}"))
    }
    /// Reads in four pieces, looks at the token between them and reports each (as a real download does).
    fn get_controlled(
        &self,
        url: &str,
        cancel: &CancelToken,
        progress: &(dyn Fn(Progress) + Sync),
    ) -> Result<Vec<u8>, String> {
        let all = self.get(url)?;
        let total = all.len() as u64;
        for i in 1..=4u64 {
            if cancel.is_cancelled() {
                return Err(CANCELLED.to_owned());
            }
            let p = Progress {
                done: total * i / 4,
                total: Some(total),
            };
            self.1.lock().unwrap().push(p);
            progress(p);
        }
        Ok(all)
    }
}
pub struct Fake;
impl SignatureVerifier for Fake {
    fn verify(&self, _: &[u8], sig: Option<&[u8]>) -> SignatureOutcome {
        match sig {
            Some(b"good") => SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
            _ => SignatureOutcome::Invalid,
        }
    }
}
pub struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

/// A tar.gz of `n` files.
pub fn tarball(n: usize, content: &str) -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    for i in 0..n {
        let mut h = tar::Header::new_gnu();
        h.set_size(content.len() as u64);
        h.set_mode(0o755);
        h.set_cksum();
        b.append_data(&mut h, format!("bin/prog{i}"), content.as_bytes())
            .unwrap();
    }
    b.into_inner().unwrap().finish().unwrap();
    raw
}

pub fn setup(version: &str, files: usize) -> (Map, Vec<u8>, Vec<u8>) {
    let tar = tarball(files, version);
    let sha = format!("{:x}", Sha256::digest(&tar));
    let catalog = format!(
        r#"<components version="1.0"><component type="desktop-application"><id>prog</id>
<releases><release version="{version}"><artifacts><artifact type="binary"><location>https://x/prog.tar.gz</location>
<checksum type="sha256">{sha}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/prog.asc</value></custom></component></components>"#
    )
    .into_bytes();
    let fetch = Map(
        HashMap::from([
            ("https://x/prog.tar.gz".into(), tar),
            ("https://x/prog.asc".into(), b"good".to_vec()),
        ]),
        Mutex::new(Vec::new()),
    );
    (fetch, catalog, b"good".to_vec())
}

pub fn installer<'a>(root: &Path, fetch: &'a Map) -> Installer<'a> {
    Installer {
        root: root.to_owned(),
        verifier: &Fake,
        fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    }
}

pub fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Installs `version` normally, as the state to come back to.
pub fn install(root: &Path, version: &str) {
    let (fetch, catalog, sig) = setup(version, 3);
    let i = installer(root, &fetch);
    i.install(&catalog, &sig, "prog", None).unwrap();
}
