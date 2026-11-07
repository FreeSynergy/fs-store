//! Pattern: Adapter (rPGP behind `SignatureVerifier`).

use fs_launcher_core::{SignatureOutcome, SignatureVerifier};

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

impl PgpVerifier {
    /// A verifier trusting only the built-in FreeSynergy release key.
    ///
    /// # Panics
    /// Never in a correct build: the file is part of the crate and tested.
    #[must_use]
    pub fn builtin() -> Self {
        todo!()
    }

    /// A verifier trusting the given armored public keys.
    ///
    /// # Errors
    /// `Unreadable` or `Revoked` for the first key that is not acceptable.
    pub fn from_armored(keys: &[&str]) -> Result<Self, KeyError> {
        let _ = keys;
        todo!()
    }
}

impl SignatureVerifier for PgpVerifier {
    fn verify(&self, data: &[u8], signature: Option<&[u8]>) -> SignatureOutcome {
        let _ = (data, signature, &self.keys);
        todo!()
    }
}
