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
/// Valid when the signature bytes are exactly `good`.
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

fn tarball(content: &str) -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(content.len() as u64);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", content.as_bytes())
        .unwrap();
    b.into_inner().unwrap().finish().unwrap();
    raw
}

fn catalog(sha: &str) -> String {
    format!(
        r#"<components version="1.0"><component type="desktop-application"><id>tasks</id>
<releases><release version="1.2.3"><artifacts><artifact type="binary"><location>https://x/t.tar.gz</location>
<checksum type="sha256">{sha}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/t.asc</value></custom></component></components>"#
    )
}

fn run(
    cat_sig: &[u8],
    rel_sig: &[u8],
    sha_override: Option<&str>,
) -> (Result<Version, InstallFailure>, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let tgz = tarball("hello");
    let sha = sha_override.map_or_else(|| format!("{:x}", Sha256::digest(&tgz)), str::to_owned);
    let f = Map(HashMap::from([
        ("https://x/t.tar.gz".into(), tgz),
        ("https://x/t.asc".into(), rel_sig.to_vec()),
    ]));
    let i = Installer {
        root: tmp.path().join("root"),
        verifier: &Fake,
        fetch: &f,
        running: &Idle,
        prefer: Format::TarGz,
        policy: SignaturePolicy::default(),
    };
    let r = i.install(catalog(&sha).as_bytes(), cat_sig, "tasks", None);
    (r, tmp)
}

#[test]
fn good_chain_installs() {
    let (r, tmp) = run(b"good", b"good", None);
    assert_eq!(r, Ok(Version::new(1, 2, 3)));
    assert_eq!(
        fs::read_to_string(tmp.path().join("root/current/tasks/bin/prog")).unwrap(),
        "hello"
    );
}

#[test]
fn unsigned_catalog_is_not_believed() {
    let (r, tmp) = run(b"bad", b"good", None);
    assert_eq!(r, Err(InstallFailure::CatalogSignature));
    assert!(!tmp.path().join("root").exists());
}

#[test]
fn bad_release_signature_installs_nothing() {
    let (r, tmp) = run(b"good", b"bad", None);
    assert_eq!(r, Err(InstallFailure::ReleaseSignature));
    assert!(!tmp.path().join("root").exists());
}

#[test]
fn checksum_mismatch_installs_nothing() {
    let (r, tmp) = run(b"good", b"good", Some("00"));
    assert_eq!(r, Err(InstallFailure::Checksum));
    assert!(!tmp.path().join("root").exists());
}

#[test]
fn an_older_catalog_than_the_installed_version_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let tgz = tarball("hello");
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
    // the catalog offers 1.2.3; 2.0.0 is installed
    let r = i.install(
        catalog(&sha).as_bytes(),
        b"good",
        "tasks",
        Some(Version::new(2, 0, 0)),
    );
    assert_eq!(r, Err(InstallFailure::Downgrade));
    assert!(!tmp.path().join("root").exists());
}
