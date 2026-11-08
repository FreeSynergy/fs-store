//! Pattern: Adapter (rPGP behind `SignatureVerifier`).

use std::fmt::Write;
use std::io::Cursor;

use fs_store_core::{SignatureOutcome, SignatureVerifier};
use pgp::composed::{Deserializable, DetachedSignature, SignedPublicKey};
use pgp::packet::Signature;
use pgp::types::KeyDetails;

/// Why a key could not be trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyError {
    /// Not a readable public key.
    Unreadable,
    /// The key carries a revocation.
    Revoked,
}

/// Checks detached signatures against a fixed set of trusted public keys.
#[derive(Debug)]
pub struct PgpVerifier {
    keys: Vec<pgp::composed::SignedPublicKey>,
}

const BUILTIN_KEY: &str = include_str!("../keys/release-signing-key.asc");

impl PgpVerifier {
    /// A verifier trusting only the built-in FreeSynergy release key.
    ///
    /// # Panics
    /// Never in a correct build: the file is part of the crate and tested.
    #[must_use]
    pub fn builtin() -> Self {
        Self::from_armored(&[BUILTIN_KEY]).expect("built-in release key is valid")
    }

    /// The built-in key plus one more armored public key (a test key of a local source, chosen by the person).
    ///
    /// # Errors
    /// `Unreadable` or `Revoked` for the extra key.
    pub fn builtin_and(extra: &str) -> Result<Self, KeyError> {
        Self::from_armored(&[BUILTIN_KEY, extra])
    }

    /// A verifier trusting the given armored public keys.
    ///
    /// # Errors
    /// `Unreadable` or `Revoked` for the first key that is not acceptable.
    pub fn from_armored(keys: &[&str]) -> Result<Self, KeyError> {
        let mut parsed = Vec::new();
        for armored in keys {
            let (key, _) =
                SignedPublicKey::from_string(armored).map_err(|_| KeyError::Unreadable)?;
            if key.verify_bindings().is_err() {
                return Err(KeyError::Unreadable);
            }
            if !key.details.revocation_signatures.is_empty() {
                return Err(KeyError::Revoked);
            }
            parsed.push(key);
        }
        Ok(Self { keys: parsed })
    }

    /// Fingerprints of the trusted keys, upper-case hex.
    #[must_use]
    pub fn fingerprints(&self) -> Vec<String> {
        self.keys.iter().map(hex_fingerprint).collect()
    }
}

fn hex_fingerprint(key: &SignedPublicKey) -> String {
    key.fingerprint()
        .as_bytes()
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02X}");
            s
        })
}

fn checks(key: &SignedPublicKey, sig: &Signature, data: &[u8]) -> bool {
    sig.verify(&key.primary_key, Cursor::new(data)).is_ok()
        || key
            .public_subkeys
            .iter()
            .any(|s| sig.verify(s, Cursor::new(data)).is_ok())
}

impl SignatureVerifier for PgpVerifier {
    fn verify(&self, data: &[u8], signature: Option<&[u8]>) -> SignatureOutcome {
        let Some(bytes) = signature else {
            return SignatureOutcome::Missing;
        };
        let Ok((sig, _)) = DetachedSignature::from_armor_single(Cursor::new(bytes)) else {
            return SignatureOutcome::Invalid;
        };
        self.keys
            .iter()
            .find(|k| checks(k, &sig.signature, data))
            .map_or(SignatureOutcome::Invalid, |k| SignatureOutcome::Valid {
                fingerprint: hex_fingerprint(k),
            })
    }
}
