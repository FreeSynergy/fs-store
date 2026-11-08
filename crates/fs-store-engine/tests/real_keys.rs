//! The whole chain with real OpenPGP: a test key signs catalog and release, nothing is faked but the key's origin.
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;

use fs_store_core::{Format, RunningCheck, SignaturePolicy, Version};
use fs_store_engine::service::{Fetch, InstallFailure, Installer};
use fs_store_pgp::PgpVerifier;
use pgp::composed::{
    ArmorOptions, DetachedSignature, KeyType, SecretKeyParamsBuilder, SignedSecretKey,
};
use pgp::crypto::hash::HashAlgorithm;
use pgp::types::{KeyDetails, KeyVersion, Password};
use sha2::{Digest, Sha256};

struct Map(HashMap<String, Vec<u8>>);
impl Fetch for Map {
    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        self.0.get(url).cloned().ok_or_else(|| format!("404 {url}"))
    }
}
struct Idle;
impl RunningCheck for Idle {
    fn is_running(&self, _: &str) -> bool {
        false
    }
}

fn key() -> SignedSecretKey {
    SecretKeyParamsBuilder::default()
        .version(KeyVersion::V6)
        .key_type(KeyType::Ed25519)
        .can_certify(true)
        .can_sign(true)
        .primary_user_id("<local@test>".to_owned())
        .build()
        .unwrap()
        .generate(rand::thread_rng())
        .unwrap()
}
fn sign(k: &SignedSecretKey, d: &[u8]) -> Vec<u8> {
    DetachedSignature::sign_binary_data(
        rand::thread_rng(),
        &k.primary_key,
        &Password::empty(),
        HashAlgorithm::Sha512,
        d,
    )
    .unwrap()
    .to_armored_bytes(ArmorOptions::default())
    .unwrap()
}
fn fp(k: &SignedSecretKey) -> String {
    k.fingerprint()
        .as_bytes()
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02X}");
            s
        })
}
fn tarball() -> Vec<u8> {
    let mut raw = Vec::new();
    let enc = flate2::write::GzEncoder::new(&mut raw, flate2::Compression::fast());
    let mut b = tar::Builder::new(enc);
    let mut h = tar::Header::new_gnu();
    h.set_size(2);
    h.set_mode(0o755);
    h.set_cksum();
    b.append_data(&mut h, "bin/prog", &b"hi"[..]).unwrap();
    b.into_inner().unwrap().finish().unwrap();
    raw
}

#[test]
fn local_source_with_an_explicitly_trusted_test_key_installs_a_pre_release() {
    let k = key();
    let tgz = tarball();
    let catalog = format!(
        r#"<components version="1.0"><component type="desktop-application"><id>tasks</id>
<releases><release version="0.6.1-rc.2"><artifacts><artifact type="binary"><location>https://x/t.tar.gz</location>
<checksum type="sha256">{:x}</checksum></artifact></artifacts></release></releases>
<custom><value key="fs:signature">https://x/t.asc</value></custom></component></components>"#,
        Sha256::digest(&tgz)
    );
    let fetch = Map(HashMap::from([
        ("https://x/t.tar.gz".into(), tgz.clone()),
        ("https://x/t.asc".into(), sign(&k, &tgz)),
    ]));
    let public = k
        .to_public_key()
        .to_armored_string(ArmorOptions::default())
        .unwrap();
    let verifier = PgpVerifier::from_armored(&[&public]).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let make = |policy| Installer {
        root: tmp.path().join("root"),
        verifier: &verifier,
        fetch: &fetch,
        running: &Idle,
        prefer: Format::TarGz,
        policy,
    };
    let sig = sign(&k, catalog.as_bytes());
    // Without the explicit opt-in the test key is nobody.
    assert_eq!(
        make(SignaturePolicy::default()).install(catalog.as_bytes(), &sig, "tasks", None),
        Err(InstallFailure::CatalogSignature)
    );
    assert!(!tmp.path().join("root").exists());
    let v = make(SignaturePolicy::default().also_trusting(&fp(&k)))
        .install(catalog.as_bytes(), &sig, "tasks", None)
        .unwrap();
    assert_eq!(v, Version::parse("0.6.1-rc.2").unwrap());
    assert_eq!(
        fs::read_to_string(tmp.path().join("root/current/tasks/bin/prog")).unwrap(),
        "hi"
    );
}
