//! An entry that is locked (it wants a newer store, or its `fs:min-store` cannot be read) is refused by the engine even when it has a real,
//! well signed release file. Without this refusal in `plan` the window and the commands would install it.

use std::collections::HashMap;
use std::fs;

use fs_store_core::*;
use fs_store_engine::service::{Fetch, InstallFailure, Installer};
use sha2::{Digest, Sha256};

struct Map(HashMap<String, Vec<u8>>);
impl Fetch for Map {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0.get(url).cloned().ok_or_else(|| format!("404 {url}"))
    }
}
struct Fake;
impl SignatureVerifier for Fake {
    fn verify(&self, _: &[u8], sig: Option<&[u8]>) -> SignatureOutcome {
        match sig {
            Some(b"good") => SignatureOutcome::Valid {
                fingerprint: TRUSTED_FINGERPRINT.into(),
            },
            Some(_) => SignatureOutcome::Invalid,
            None => SignatureOutcome::Missing,
        }
    }
}
struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

fn tarball() -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(5);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", &b"hello"[..]).unwrap();
    b.into_inner().unwrap().finish().unwrap();
    raw
}

/// A catalog with a real release file for `tasks` and the given `fs:min-store` value.
fn catalog(sha: &str, min_store: &str) -> String {
    format!(
        r#"<components version="1.0"><component type="desktop-application"><id>tasks</id>
<releases><release version="1.2.3"><artifacts><artifact type="binary"><location>https://x/t.tar.gz</location>
<checksum type="sha256">{sha}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/t.asc</value><value key="fs:min-store">{min_store}</value></custom></component></components>"#
    )
}

fn try_plan(min_store: &str) -> (Result<(), InstallFailure>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let tgz = tarball();
    let sha = format!("{:x}", Sha256::digest(&tgz));
    let f = Map(HashMap::from([
        ("https://x/t.tar.gz".into(), tgz),
        ("https://x/t.asc".into(), b"good".to_vec()),
    ]));
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &f,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    };
    let text = catalog(&sha, min_store);
    let planned = i.plan(text.as_bytes(), b"good", "tasks", None).map(|_| ());
    let installed = i.install(text.as_bytes(), b"good", "tasks", None);
    // plan and install agree: whatever plan refuses, install refuses too
    assert_eq!(planned.is_ok(), installed.is_ok());
    (planned, tmp)
}

#[test]
fn an_entry_that_wants_a_newer_store_is_not_planned_even_with_a_real_release_file() {
    let (r, tmp) = try_plan("99.0.0");
    assert!(matches!(r, Err(InstallFailure::NotInstallable(_))), "{r:?}");
    // nothing was put on the machine
    assert!(!tmp.path().join("root/current/tasks").exists());
}

#[test]
fn an_entry_with_an_unreadable_min_store_is_not_planned_either() {
    let (r, _tmp) = try_plan("0.1");
    assert!(matches!(r, Err(InstallFailure::NotInstallable(why)) if why.contains("min-store")));
}

#[test]
fn counter_proof_an_entry_that_wants_an_older_store_is_planned_and_installed() {
    let (r, tmp) = try_plan("0.0.1");
    assert_eq!(r, Ok(()));
    assert_eq!(
        fs::read_to_string(tmp.path().join("root/current/tasks/bin/prog")).unwrap(),
        "hello"
    );
}
