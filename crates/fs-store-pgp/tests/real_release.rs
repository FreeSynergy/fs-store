//! Probe against a real release (sprint step 5). Runs only when `FS_PROBE_FILE` and `FS_PROBE_SIG` are set.
use fs_store_core::{SignatureOutcome, SignatureVerifier, TRUSTED_FINGERPRINT};
use fs_store_pgp::PgpVerifier;

#[test]
fn real_release_file_verifies_with_the_built_in_key() {
    let (Some(file), Some(sig)) = (
        std::env::var_os("FS_PROBE_FILE"),
        std::env::var_os("FS_PROBE_SIG"),
    ) else {
        return;
    };
    let data = std::fs::read(file).unwrap();
    let sig = std::fs::read(sig).unwrap();
    let v = PgpVerifier::builtin();
    assert_eq!(
        v.verify(&data, Some(&sig)),
        SignatureOutcome::Valid {
            fingerprint: TRUSTED_FINGERPRINT.into()
        }
    );
    let mut bad = data;
    bad[100] ^= 1;
    assert_eq!(v.verify(&bad, Some(&sig)), SignatureOutcome::Invalid);
}
