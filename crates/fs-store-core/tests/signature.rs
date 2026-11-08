//! Tests of the signature gate: no valid signature from the trusted key, no `VerifiedPackage`.
mod common;
use common::*;
use fs_store_core::*;

#[test]
fn only_valid_signature_of_trusted_key_admits() {
    let policy = SignaturePolicy::default();
    assert!(policy.admit(pkg((1, 0, 0)), good()).is_ok());
    for bad in [
        SignatureOutcome::Invalid,
        SignatureOutcome::Missing,
        SignatureOutcome::UnknownKey,
        SignatureOutcome::Valid {
            fingerprint: "DEADBEEF".into(),
        },
    ] {
        assert_eq!(policy.admit(pkg((1, 0, 0)), bad.clone()), Err(bad));
    }
}

#[test]
fn missing_signature_never_passes_through_a_verifier() {
    let v = FixedVerifier(SignatureOutcome::Missing);
    let outcome = v.verify(b"data", None);
    assert!(
        SignaturePolicy::default()
            .admit(pkg((1, 0, 0)), outcome)
            .is_err()
    );
}

#[test]
fn built_in_fingerprint_is_the_documented_one() {
    assert_eq!(
        TRUSTED_FINGERPRINT,
        "16A4A3EE563AEF51FCF0C55DED7B57C4A1904342"
    );
}
