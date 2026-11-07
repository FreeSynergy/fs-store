//! Tests of the OpenPGP check with test keys made here; no real private key is used or stored.
use std::fmt::Write;

use fs_launcher_core::{SignatureOutcome, SignaturePolicy, SignatureVerifier, TRUSTED_FINGERPRINT};
use fs_launcher_pgp::{KeyError, PgpVerifier};
use pgp::composed::{
    ArmorOptions, DetachedSignature, KeyType, SecretKeyParamsBuilder, SignedSecretKey,
};
use pgp::crypto::hash::HashAlgorithm;
use pgp::types::{KeyDetails, KeyVersion, Password};

fn test_key(name: &str) -> SignedSecretKey {
    SecretKeyParamsBuilder::default()
        .version(KeyVersion::V6)
        .key_type(KeyType::Ed25519)
        .can_certify(true)
        .can_sign(true)
        .primary_user_id(format!("<{name}@test>"))
        .build()
        .unwrap()
        .generate(rand::thread_rng())
        .unwrap()
}

fn sign(key: &SignedSecretKey, data: &[u8]) -> Vec<u8> {
    DetachedSignature::sign_binary_data(
        rand::thread_rng(),
        &key.primary_key,
        &Password::empty(),
        HashAlgorithm::Sha512,
        data,
    )
    .unwrap()
    .to_armored_bytes(ArmorOptions::default())
    .unwrap()
}

fn armored_public(key: &SignedSecretKey) -> String {
    key.to_public_key()
        .to_armored_string(ArmorOptions::default())
        .unwrap()
}

fn fingerprint(key: &SignedSecretKey) -> String {
    key.fingerprint()
        .as_bytes()
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02X}");
            s
        })
}

#[test]
fn valid_signature_of_the_trusted_test_key_is_accepted() {
    let key = test_key("release");
    let v = PgpVerifier::from_armored(&[&armored_public(&key)]).unwrap();
    let outcome = v.verify(b"payload", Some(&sign(&key, b"payload")));
    assert_eq!(
        outcome,
        SignatureOutcome::Valid {
            fingerprint: fingerprint(&key)
        }
    );
    let policy = SignaturePolicy::with_fingerprint(&fingerprint(&key));
    assert!(policy.is_satisfied_by(&outcome));
}

#[test]
fn signature_of_another_key_is_rejected() {
    let (trusted, other) = (test_key("a"), test_key("b"));
    let v = PgpVerifier::from_armored(&[&armored_public(&trusted)]).unwrap();
    let outcome = v.verify(b"payload", Some(&sign(&other, b"payload")));
    assert!(!matches!(outcome, SignatureOutcome::Valid { .. }));
}

#[test]
fn changed_file_is_rejected() {
    let key = test_key("release");
    let v = PgpVerifier::from_armored(&[&armored_public(&key)]).unwrap();
    let outcome = v.verify(b"payload!", Some(&sign(&key, b"payload")));
    assert_eq!(outcome, SignatureOutcome::Invalid);
}

#[test]
fn missing_and_garbage_signatures_are_rejected() {
    let v = PgpVerifier::builtin();
    assert_eq!(v.verify(b"x", None), SignatureOutcome::Missing);
    assert_eq!(
        v.verify(b"x", Some(b"not a signature")),
        SignatureOutcome::Invalid
    );
}

#[test]
fn builtin_key_has_the_documented_fingerprint() {
    let armored = include_str!("../keys/release-signing-key.asc");
    let v = PgpVerifier::from_armored(&[armored]).unwrap();
    assert_eq!(v.fingerprints(), vec![TRUSTED_FINGERPRINT.to_owned()]);
    assert_eq!(PgpVerifier::builtin().fingerprints(), v.fingerprints());
}

#[test]
fn old_revoked_key_is_refused() {
    let armored = include_str!("../keys/revoked-release-key-C40F0FBA.asc");
    assert_eq!(
        PgpVerifier::from_armored(&[armored]).unwrap_err(),
        KeyError::Revoked
    );
}

#[test]
fn test_signature_is_not_valid_for_the_builtin_policy() {
    let key = test_key("release");
    let v = PgpVerifier::from_armored(&[&armored_public(&key)]).unwrap();
    let outcome = v.verify(b"p", Some(&sign(&key, b"p")));
    assert!(!SignaturePolicy::default().is_satisfied_by(&outcome));
}
